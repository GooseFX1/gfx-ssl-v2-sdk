mod metrics;

use std::{cell::RefCell, collections::HashMap, fmt::Debug};

use anchor_lang::{
    prelude::{Clock, UpgradeableLoaderState},
    AccountDeserialize, InstructionData, ToAccountMetas,
};
use anchor_spl::associated_token::get_associated_token_address;
use anyhow::Error;
use fehler::{throw, throws};
use gfx_ssl_v2_jupiter::{error::GfxJupiterIntegrationError::*, tuple::Tuple};
use gfx_ssl_v2_sdk::state::{EventEmitter, OraclePriceHistory, Pair, PoolRegistry, SSLPool};
use jupiter_amm_interface::{
    AccountMap, Amm, KeyedAccount, Quote, QuoteParams, SwapAndAccountMetas, SwapParams,
};
pub use metrics::*;
use rust_decimal::Decimal;
use solana_bpf_simulator::{SBPFMessageExecutor, WorkingSlot, FEATURES};
use solana_sdk::{
    account::{Account, AccountSharedData, ReadableAccount, WritableAccount},
    account_utils::StateMut,
    bpf_loader_upgradeable,
    message::{LegacyMessage, Message, SanitizedMessage},
    native_loader,
    program_option::COption,
    program_pack::Pack,
    pubkey,
    pubkey::Pubkey,
    system_program,
    sysvar::clock,
};
use spl_token::{native_mint, state::AccountState};
use tracing::error;

#[derive(Debug, Clone)]
struct AccountWithKey {
    key: Pubkey,
    account: AccountSharedData,
}

impl From<(Pubkey, AccountSharedData)> for AccountWithKey {
    fn from(value: (Pubkey, AccountSharedData)) -> Self {
        Self {
            key: value.0,
            account: value.1,
        }
    }
}

impl From<(Pubkey, Account)> for AccountWithKey {
    fn from(value: (Pubkey, Account)) -> Self {
        Self {
            key: value.0,
            account: value.1.into(),
        }
    }
}

/// Struct that implements the `jupiter_core::amm::Amm` trait.
#[derive(Debug, Clone)]
pub struct GfxAmm {
    pair: Pubkey,

    pub log: bool,

    mints: Tuple<2, Pubkey>,

    pool_registry: Pubkey,
    fee_rates: Tuple<2, u16>,
    fee_destination: Tuple<2, Pubkey>,
    ssl_signers: Tuple<2, Pubkey>,
    main_vaults: Tuple<2, Pubkey>,
    secondary_vaults: Tuple<2, Pubkey>,
    fee_vaults: Tuple<2, Pubkey>,
    oracles: Tuple<2, Pubkey>,
    event_emitter: Pubkey,

    price_histories: Tuple<2, Pubkey>, // this will get updated once pool_registry is updated
    program_data_address: Pubkey,

    accounts: HashMap<Pubkey, Option<AccountSharedData>>,

    // cached fake accounts
    user_wallet: AccountWithKey,
    user_atas: Tuple<2, AccountWithKey>,
}

impl GfxAmm {
    #[throws(Error)]
    fn ready(&self) {
        if self.program_data_address == Pubkey::default()
            || self.price_histories.any(|k| k == &Pubkey::default())
            || self.oracles.any(|k| k == &Pubkey::default())
        {
            throw!(RequiredAccountUpdate);
        }
    }
}

impl Amm for GfxAmm {
    fn from_keyed_account(pair: &KeyedAccount) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let mut accounts = HashMap::new();

        accounts.insert(clock::ID, None); // Can be lifted if a slot is passed in
        accounts.insert(gfx_ssl_v2_sdk::ID, None);
        accounts.insert(spl_token::ID, None);
        accounts.insert(EventEmitter::address(), None);

        let pair_pubkey = pair.key;
        accounts.insert(pair.key, Some(pair.account.clone().into()));

        let data = &pair.account.data;
        let pair: Pair = Pair::try_deserialize(&mut data.as_slice())
            .map_err(|_| DeserializeFailure(pair_pubkey, "Pair".to_string()))?;

        accounts.insert(pair.pool_registry, None);

        let mints = pair.mints;
        let ssl_signers = (
            SSLPool::signer_address(pair.pool_registry, mints.0),
            SSLPool::signer_address(pair.pool_registry, mints.1),
        );

        let main_vaults = (
            get_associated_token_address(&ssl_signers.0, &mints.0),
            get_associated_token_address(&ssl_signers.1, &mints.1),
        );
        accounts.insert(main_vaults.0, None);
        accounts.insert(main_vaults.1, None);

        let secondary_vaults = (
            get_associated_token_address(&ssl_signers.0, &mints.1),
            get_associated_token_address(&ssl_signers.1, &mints.0),
        );
        accounts.insert(secondary_vaults.0, None);
        accounts.insert(secondary_vaults.1, None);

        let fee_vaults = (
            get_associated_token_address(&pair.pool_registry, &mints.0),
            get_associated_token_address(&pair.pool_registry, &mints.1),
        );
        accounts.insert(fee_vaults.0, None);
        accounts.insert(fee_vaults.1, None);

        let (_, fee_destination_a, _) = pair
            .find_fee_attrs(mints.0, mints.1)
            .map_err(|_| CannotResolveFeeDestination)?;
        let (_, fee_destination_b, _) = pair
            .find_fee_attrs(mints.1, mints.0)
            .map_err(|_| CannotResolveFeeDestination)?;
        accounts.insert(fee_destination_a, None);
        accounts.insert(fee_destination_b, None);

        let user_wallet = pubkey!("GFXFAKEWA11ET111111111111111111111111111111");
        let mut user_ata_a = (
            get_associated_token_address(&user_wallet, &mints.0),
            Account {
                owner: spl_token::ID,
                lamports: 0,
                rent_epoch: 0,
                executable: false,
                data: vec![0; spl_token::state::Account::LEN],
            },
        );
        spl_token::state::Account {
            mint: mints.0,
            owner: user_wallet,
            amount: 0,
            state: AccountState::Initialized,
            close_authority: COption::None,
            delegate: COption::None,
            delegated_amount: 0,
            is_native: (mints.0 == native_mint::ID).then_some(0).into(),
        }
        .pack_into_slice(&mut user_ata_a.1.data);

        let mut user_ata_b = (
            get_associated_token_address(&user_wallet, &mints.1),
            Account {
                owner: spl_token::ID,
                lamports: 0,
                rent_epoch: 0,
                executable: false,
                data: vec![0; spl_token::state::Account::LEN],
            },
        );
        spl_token::state::Account {
            mint: mints.1,
            owner: user_wallet,
            amount: 0,
            state: AccountState::Initialized,
            close_authority: COption::None,
            delegate: COption::None,
            delegated_amount: 0,
            is_native: (mints.1 == native_mint::ID).then_some(0).into(),
        }
        .pack_into_slice(&mut user_ata_b.1.data);

        Ok(Self {
            log: false,
            pair: pair_pubkey,
            pool_registry: pair.pool_registry,
            price_histories: Tuple::default(),
            program_data_address: Pubkey::default(),
            fee_destination: (fee_destination_a, fee_destination_b).into(),
            mints: mints.into(),
            ssl_signers: ssl_signers.into(),
            main_vaults: main_vaults.into(),
            secondary_vaults: secondary_vaults.into(),
            fee_vaults: fee_vaults.into(),
            fee_rates: pair.fee_rates.into(),
            accounts,
            event_emitter: EventEmitter::address(),

            oracles: Default::default(),

            user_wallet: (
                user_wallet,
                Account {
                    owner: system_program::ID,
                    lamports: 0,
                    rent_epoch: 0,
                    executable: false,
                    data: vec![],
                },
            )
                .into(),
            user_atas: (user_ata_a.into(), user_ata_b.into()).into(),
        })
    }

    /// Human-readable name for the Amm pair.
    fn label(&self) -> String {
        "GooseFX".to_string()
    }

    fn program_id(&self) -> Pubkey {
        gfx_ssl_v2_sdk::ID
    }

    /// Get a pubkey to represent the Amm as a whole.
    fn key(&self) -> Pubkey {
        self.pair
    }

    /// Returns mints offered by this Amm for swap.
    fn get_reserve_mints(&self) -> Vec<Pubkey> {
        self.mints.to_vec()
    }

    /// Returns pubkeys of all the accounts required
    /// for providing accurate quotes and swap instructions.
    fn get_accounts_to_update(&self) -> Vec<Pubkey> {
        self.accounts.keys().copied().collect()
    }

    /// Update the account state contained in self.
    #[throws(Error)]
    fn update(&mut self, account_map: &AccountMap) {
        for (pubkey, account) in account_map {
            if !self.accounts.contains_key(pubkey) {
                continue;
            };

            if pubkey == &self.pool_registry {
                let pool_registry = PoolRegistry::try_deserialize(&mut account.data.as_slice())
                    .map_err(|_| DeserializeFailure(*pubkey, "PoolRegistry".to_string()))?;

                for i in [0, 1] {
                    let mint = self.mints[i];
                    let ssl: &SSLPool = pool_registry
                        .find_pool(mint)
                        .map_err(|_| PoolNotFound(mint))?;

                    let mut remove_existing_history = None;
                    let mut insert = true;
                    if let Some(account) = self.accounts.get(pubkey).expect("No way") {
                        let pool_registry = PoolRegistry::try_deserialize(&mut account.data())
                            .map_err(|_| DeserializeFailure(*pubkey, "PoolRegistry".to_string()))?;

                        let existing_ssl: &SSLPool = pool_registry
                            .find_pool(mint)
                            .map_err(|_| PoolNotFound(mint))?;

                        if existing_ssl.oracle_price_histories[0] == ssl.oracle_price_histories[0] {
                            insert = false;
                        } else {
                            remove_existing_history = Some(existing_ssl.oracle_price_histories[0]);
                        }
                    }

                    if let Some(k) = remove_existing_history {
                        self.accounts.remove(&k);
                    }

                    if insert {
                        self.accounts.insert(ssl.oracle_price_histories[0], None);
                    }

                    self.price_histories[i] = ssl.oracle_price_histories[0];
                }
            } else if let Some(i) = self.price_histories.iter().position(|k| k == pubkey) {
                let history = OraclePriceHistory::try_deserialize(&mut account.data.as_slice())
                    .map_err(|_| DeserializeFailure(*pubkey, "OraclePriceHistory".to_string()))?;

                if !self.accounts.contains_key(&history.oracle_address) {
                    self.accounts.insert(history.oracle_address, None);
                }
                self.oracles[i] = history.oracle_address;
            } else if pubkey == &gfx_ssl_v2_sdk::ID {
                let state: UpgradeableLoaderState =
                    account.state().expect("SSL Program is not upgradable?");
                let programdata_address = match state {
                    UpgradeableLoaderState::Program {
                        programdata_address,
                    } => programdata_address,
                    _ => throw!(NotUpgradable),
                };

                // There must be a program update, the program address is guaranteed to be different
                if programdata_address != self.program_data_address {
                    self.accounts.remove(&self.program_data_address);
                    self.accounts.insert(programdata_address, None);
                }
                self.program_data_address = programdata_address;
            }

            // Update the account
            let Some(maybe_existing) = self.accounts.get_mut(pubkey) else {
                continue;
            };
            let account = account.clone();
            *maybe_existing = Some(account.into());
        }
    }

    /// Get a GooseFX SSL swap quote
    #[throws(Error)]
    fn quote(&self, quote_params: &QuoteParams) -> Quote {
        thread_local! {
            pub static EXECUTOR: RefCell<SBPFMessageExecutor> = RefCell::new(SBPFMessageExecutor::new(FEATURES).unwrap());
        }

        self.ready()?;

        let a_to_b = quote_params.input_mint == self.mints[0];

        let (
            fee_destination,
            price_histories,
            signers,
            atas,
            main_vaults,
            secondary_vaults,
            fee_vault,
            oracles,
        ) = if a_to_b {
            (
                self.fee_destination[0],
                self.price_histories,
                self.ssl_signers,
                self.user_atas
                    .iter()
                    .map(|a| a.key)
                    .collect::<Tuple<2, _>>(),
                self.main_vaults,
                self.secondary_vaults,
                self.fee_vaults[1],
                self.oracles,
            )
        } else {
            (
                self.fee_destination[1],
                self.price_histories.reverse(),
                self.ssl_signers.reverse(),
                self.user_atas.iter().map(|a| a.key).rev().collect(),
                self.main_vaults.reverse(),
                self.secondary_vaults.reverse(),
                self.fee_vaults[0],
                self.oracles.reverse(),
            )
        };

        let metas = gfx_ssl_v2_sdk::anchor::accounts::Swap {
            pair: self.pair,
            pool_registry: self.pool_registry,
            user_wallet: self.user_wallet.key,
            ssl_pool_in_signer: signers[0],
            ssl_pool_out_signer: signers[1],
            user_ata_in: atas[0],
            user_ata_out: atas[1],
            ssl_out_main_vault: main_vaults[1],
            ssl_out_secondary_vault: secondary_vaults[1],
            ssl_in_main_vault: main_vaults[0],
            ssl_in_secondary_vault: secondary_vaults[0],
            ssl_out_fee_vault: fee_vault,
            fee_destination,
            output_token_price_history: price_histories[1],
            output_token_oracle: oracles[1],
            input_token_price_history: price_histories[0],
            input_token_oracle: oracles[0],
            event_emitter: self.event_emitter,
            token_program: spl_token::ID,
        }
        .to_account_metas(None);

        let ix = solana_sdk::instruction::Instruction {
            program_id: gfx_ssl_v2_sdk::ID,
            accounts: metas,
            data: gfx_ssl_v2_sdk::anchor::instruction::Swap {
                amount_in: quote_params.amount,
                min_out: 0,
            }
            .data(),
        };

        let message = Message::new(&[ix], Some(&self.user_wallet.key));
        let message = SanitizedMessage::Legacy(LegacyMessage::new(message));

        EXECUTOR.with(|sbf| {
            let mut sbf = sbf.borrow_mut();
            let clock = self
                .accounts
                .get(&clock::id())
                .cloned()
                .ok_or(RequiredAccountUpdate)?
                .ok_or(RequiredAccountUpdate)?;
            let clock: Clock = bincode::deserialize(&clock.data())?;
            let slot = clock.slot;
            sbf.sysvar_cache_mut().set_clock(clock);

            let mut loader = sbf.loader(|key| {
                if key == &self.user_wallet.key {
                    return Some(self.user_wallet.account.clone());
                }
                if key == &self.user_atas[0].key {
                    let mut account = self.user_atas[0].account.clone();
                    if quote_params.input_mint == self.mints[0] {
                        set_spl_amount(account.data_as_mut_slice(), quote_params.amount);
                        if quote_params.input_mint == native_mint::ID {
                            account.set_lamports(quote_params.amount.saturating_add(1_000_000_000));
                        }
                    }
                    return Some(account);
                }
                if key == &self.user_atas[1].key {
                    let mut account = self.user_atas[1].account.clone();
                    if quote_params.input_mint == self.mints[1] {
                        set_spl_amount(account.data_as_mut_slice(), quote_params.amount);
                        if quote_params.input_mint == native_mint::ID {
                            account.set_lamports(quote_params.amount.saturating_add(1_000_000_000));
                        }
                    }
                    return Some(account);
                }

                if key == &self.ssl_signers[0] || key == &self.ssl_signers[1] {
                    return Some(Default::default());
                }

                if key == &bpf_loader_upgradeable::ID {
                    return Some(
                        Account {
                            owner: native_loader::ID,
                            executable: true,
                            rent_epoch: 0,
                            data: b"solana_bpf_loader_upgradable_program".to_vec(),
                            lamports: 1,
                        }
                        .into(),
                    );
                }

                let account = self.accounts.get(key).cloned().flatten();
                if account.is_none() {
                    error!("Missing account {key}");
                }

                account
            });

            let loaded_transaction = loader.load_transaction_account(&message)?;
            let loaded_programs = loader.load_programs(&WorkingSlot(slot), [&message])?;

            let result = sbf.process(slot, &message, loaded_transaction, &loaded_programs);
            // if result.is_err() {
            //     println!("111 {:?}", sbf.logger().get_recorded_content());
            // }

            let result = result?;

            let accounts: HashMap<_, _> = result.keys.into_iter().zip(result.datas).collect();

            let (fee_pct, input_amount_after, output_amount_after) = if a_to_b {
                (
                    self.fee_rates[0],
                    spl_amount(&accounts.get(&self.user_atas[0].key).unwrap().data()).unwrap(),
                    spl_amount(&accounts.get(&self.user_atas[1].key).unwrap().data()).unwrap(),
                )
            } else {
                (
                    self.fee_rates[1],
                    spl_amount(&accounts.get(&self.user_atas[1].key).unwrap().data()).unwrap(),
                    spl_amount(&accounts.get(&self.user_atas[0].key).unwrap().data()).unwrap(),
                )
            };
            let fee_amount_before = spl_amount(
                self.accounts
                    .get(&fee_destination)
                    .unwrap()
                    .as_ref()
                    .unwrap()
                    .data(),
            )
            .unwrap();
            let fee_amount_after =
                spl_amount(&accounts.get(&fee_destination).unwrap().data()).unwrap();
            let fee_pct = Decimal::new(fee_pct.into(), 4);

            let quote = Quote {
                not_enough_liquidity: false,
                min_in_amount: None,
                min_out_amount: None,
                in_amount: quote_params.amount.saturating_sub(input_amount_after),
                out_amount: output_amount_after,
                fee_amount: fee_amount_after.saturating_sub(fee_amount_before) * 2,
                fee_mint: quote_params.output_mint,
                fee_pct,
            };

            Result::<_, Error>::Ok(quote)
        })?
    }

    /// Get account metas for a GFX swap instruction,
    /// and marker denoting a [SwapLeg::Swap], and a [Swap::GooseFX].
    #[allow(unreachable_code)]
    #[throws(Error)]
    fn get_swap_and_account_metas(&self, _swap_params: &SwapParams) -> SwapAndAccountMetas {
        todo!()
    }

    /// Clone this object in a [Box].
    fn clone_amm(&self) -> Box<dyn Amm + Send + Sync> {
        Box::new(self.clone())
    }
}

fn spl_amount(bytes: &[u8]) -> Option<u64> {
    if bytes.len() < 72 {
        return None;
    }
    let mut amount_bytes = [0u8; 8];
    amount_bytes.copy_from_slice(&bytes[64..72]);
    Some(u64::from_le_bytes(amount_bytes))
}

fn set_spl_amount(bytes: &mut [u8], amount: u64) {
    if bytes.len() < 72 {
        return;
    }
    bytes[64..72].copy_from_slice(&u64::to_le_bytes(amount))
}
