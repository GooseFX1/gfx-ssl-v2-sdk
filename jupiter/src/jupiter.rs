use std::{cell::RefCell, collections::HashMap, fmt::Debug};

use anchor_lang::{
    prelude::{Clock, UpgradeableLoaderState},
    AccountDeserialize, AccountSerialize, InstructionData, ToAccountMetas,
};
use anchor_spl::associated_token::get_associated_token_address;
use anyhow::Error;
use fehler::{throw, throws};
use gfx_ssl_v2_sdk::state::{OraclePriceHistory, Pair, PoolRegistry, SSLPool};
use jupiter_amm_interface::{
    AccountMap, Amm, KeyedAccount, Quote, QuoteParams, Swap, SwapAndAccountMetas, SwapParams,
};
use once_cell::sync::Lazy;
use rust_decimal::Decimal;
use solana_bpf_simulator::{SBFExecutor, WrappedSlot, FEATURES};
use solana_sdk::{
    account::{Account, AccountSharedData, ReadableAccount},
    account_utils::StateMut,
    message::{LegacyMessage, Message, SanitizedMessage},
    native_loader, pubkey,
    pubkey::Pubkey,
    sysvar::clock,
};

use crate::{error::GfxJupiterIntegrationError::*, swap_account_metas::get_account_metas_for_swap};

static BPF_LOADER: Lazy<AccountSharedData> = Lazy::new(|| {
    Account {
        owner: native_loader::ID,
        executable: true,
        rent_epoch: 0,
        data: b"solana_bpf_loader_upgradeable_program".to_vec(),
        lamports: 1,
    }
    .into()
});

/// Struct that implements the `jupiter_core::amm::Amm` trait.
#[derive(Debug, Clone)]
pub struct GfxAmm {
    pair: Pubkey,

    pool_registry: Pubkey,
    mints: (Pubkey, Pubkey),
    fee_rates: (u16, u16),
    fee_destination: (Pubkey, Pubkey),
    price_histories: Vec<Pubkey>,
    has_program_data: bool,
    main_vaults: (Pubkey, Pubkey),
    secondary_vaults: (Pubkey, Pubkey),
    oracles: (Pubkey, Pubkey),

    accounts: HashMap<Pubkey, Option<AccountSharedData>>,
}

impl Amm for GfxAmm {
    fn from_keyed_account(pair: &KeyedAccount) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let mut accounts = HashMap::new();

        accounts.insert(clock::ID, None); // Can be lifted if a slot is passed in
        accounts.insert(gfx_ssl_v2_sdk::ID, None);

        let pair_pubkey = pair.key;
        accounts.insert(pair.key, None);

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

        let (_, fee_destination_a, _) = pair
            .find_fee_attrs(mints.0, mints.1)
            .map_err(|_| CannotResolveFeeDestination)?;
        let (_, fee_destination_b, _) = pair
            .find_fee_attrs(mints.1, mints.0)
            .map_err(|_| CannotResolveFeeDestination)?;

        Ok(Self {
            pair: pair_pubkey,
            pool_registry: pair.pool_registry,
            has_program_data: false,
            price_histories: vec![],
            fee_destination: (fee_destination_a, fee_destination_b),
            mints,
            fee_rates: pair.fee_rates,
            accounts,

            main_vaults,
            secondary_vaults,
            oracles: Default::default(),
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
        vec![self.mints.0, self.mints.1]
    }

    /// Returns pubkeys of all the accounts required
    /// for providing accurate quotes and swap instructions.
    fn get_accounts_to_update(&self) -> Vec<Pubkey> {
        self.accounts.keys().copied().collect()
    }

    /// Update the account state contained in self.
    fn update(&mut self, account_map: &AccountMap) -> Result<(), Error> {
        for (pubkey, account) in account_map {
            let acc = match self.accounts.get_mut(pubkey) {
                Some(a) => a,
                None => continue,
            };
            *acc = Some(account.clone().into());

            if self.price_histories.is_empty() && pubkey == &self.pool_registry {
                let pool_registry = PoolRegistry::try_deserialize(&mut account.data.as_slice())
                    .map_err(|_| DeserializeFailure(*pubkey, "PoolRegistry".to_string()))?;

                for mint in [self.mints.0, self.mints.1] {
                    let ssl: &SSLPool = pool_registry
                        .find_pool(mint)
                        .map_err(|_| PoolNotFound(mint))?;
                    self.accounts.insert(ssl.oracle_price_histories[0], None);
                    self.price_histories.push(ssl.oracle_price_histories[0]);
                }
            } else if !self.price_histories.is_empty() && pubkey == &self.price_histories[0]
            // && self.oracles.0 == Pubkey::default()
            {
                let mut history = OraclePriceHistory::try_deserialize(&mut account.data.as_slice())
                    .map_err(|_| DeserializeFailure(*pubkey, "OraclePriceHistory".to_string()))?;

                for h in &mut history.price_history {
                    h.price.inv = (1. / Into::<f64>::into(h.price)) as f32;
                }

                let mut data = vec![];
                history.try_serialize(&mut data).unwrap();
                *acc = Some(
                    Account {
                        lamports: account.lamports,
                        owner: account.owner,
                        data,
                        executable: account.executable,
                        rent_epoch: account.rent_epoch,
                    }
                    .into(),
                );

                if !self.accounts.contains_key(&history.oracle_address) {
                    self.accounts.insert(history.oracle_address, None);
                }
                self.oracles.0 = history.oracle_address;
            } else if !self.price_histories.is_empty() && pubkey == &self.price_histories[1]
            // && self.oracles.1 == Pubkey::default()
            {
                let mut history = OraclePriceHistory::try_deserialize(&mut account.data.as_slice())
                    .map_err(|_| DeserializeFailure(*pubkey, "OraclePriceHistory".to_string()))?;

                for h in &mut history.price_history {
                    h.price.inv = (1. / Into::<f64>::into(h.price)) as f32;
                }

                let mut data = vec![];
                history.try_serialize(&mut data).unwrap();
                *acc = Some(
                    Account {
                        lamports: account.lamports,
                        owner: account.owner,
                        data,
                        executable: account.executable,
                        rent_epoch: account.rent_epoch,
                    }
                    .into(),
                );

                if !self.accounts.contains_key(&history.oracle_address) {
                    self.accounts.insert(history.oracle_address, None);
                }
                self.oracles.1 = history.oracle_address;
            } else if !self.has_program_data && pubkey == &gfx_ssl_v2_sdk::ID {
                let state: UpgradeableLoaderState =
                    account.state().expect("SSL Program is not upgradable?");
                let programdata_address = match state {
                    UpgradeableLoaderState::Program {
                        programdata_address,
                    } => programdata_address,
                    _ => unreachable!("SSL Program is not upgradable?"),
                };
                self.accounts.insert(programdata_address, None);
                self.has_program_data = true;
            }
        }
        Ok(())
    }

    /// Get a GooseFX SSL swap quote
    #[throws(Error)]
    fn quote(&self, quote_params: &QuoteParams) -> Quote {
        thread_local! {
            pub static EXECUTOR: RefCell<SBFExecutor> = RefCell::new(SBFExecutor::new(FEATURES).unwrap());
        }

        if !self.has_program_data
            || self.price_histories.is_empty()
            || self.oracles.0 == Pubkey::default()
            || self.oracles.1 == Pubkey::default()
        {
            throw!(RequiredAccountUpdate);
        }

        let (price_histories, main_vaults, secondary_vaults, oracles) =
            if quote_params.input_mint == self.mints.0 {
                (
                    (self.price_histories[0], self.price_histories[1]),
                    (self.main_vaults.0, self.main_vaults.1),
                    (self.secondary_vaults.0, self.secondary_vaults.1),
                    (self.oracles.0, self.oracles.1),
                )
            } else {
                (
                    (self.price_histories[1], self.price_histories[0]),
                    (self.main_vaults.1, self.main_vaults.0),
                    (self.secondary_vaults.1, self.secondary_vaults.0),
                    (self.oracles.1, self.oracles.0),
                )
            };
        let metas = gfx_ssl_v2_sdk::anchor::accounts::Quote {
            pair: self.pair,
            pool_registry: self.pool_registry,

            ssl_in_main_vault: main_vaults.0,
            ssl_in_secondary_vault: secondary_vaults.0,
            input_token_price_history: price_histories.0,
            input_token_oracle: oracles.0,

            ssl_out_main_vault: main_vaults.1,
            ssl_out_secondary_vault: secondary_vaults.1,
            output_token_price_history: price_histories.1,
            output_token_oracle: oracles.1,
        }
        .to_account_metas(None);

        let ix = solana_sdk::instruction::Instruction {
            program_id: gfx_ssl_v2_sdk::ID,
            accounts: metas,
            data: gfx_ssl_v2_sdk::anchor::instruction::Quote {
                amount_in: quote_params.amount,
            }
            .data(),
        };

        let message = Message::new(&[ix], None);
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
                if key == &pubkey!("BPFLoaderUpgradeab1e11111111111111111111111") {
                    return Some(BPF_LOADER.clone());
                }

                self.accounts.get(key).cloned().flatten()
            });

            let loaded_transaction = loader.load_transaction_account(&message)?;
            let loaded_programs = loader.load_programs(&WrappedSlot(slot), [&message])?;

            sbf.record_log();
            let result = sbf.process(slot, &message, loaded_transaction, &loaded_programs);
            let logs = sbf.logger();
            let _ = result?;

            let line = logs
                .get_recorded_content()
                .iter()
                .find(|line| line.starts_with("Program log: QuoteResult: "))
                .ok_or(MissingQuoteLine)?;
            let mut iter = line
                .trim_start_matches("Program log: QuoteResult: ")
                .split(" ");
            let output: u64 = iter.next().ok_or(MissingQuoteLine)?.parse()?;
            let fee: u64 = iter.next().ok_or(MissingQuoteLine)?.parse()?;

            let fee_pct = if quote_params.input_mint == self.mints.0 {
                self.fee_rates.0
            } else {
                self.fee_rates.1
            };
            let fee_pct = Decimal::new(fee_pct.into(), 4);

            let quote = Quote {
                not_enough_liquidity: false,
                min_in_amount: None,
                min_out_amount: None,
                in_amount: quote_params.amount,
                out_amount: output,
                fee_amount: fee,
                fee_mint: quote_params.output_mint,
                fee_pct,
            };

            Result::<_, Error>::Ok(quote)
        })?
    }

    /// Get account metas for a GFX swap instruction,
    /// and marker denoting a [SwapLeg::Swap], and a [Swap::GooseFX].
    #[throws(Error)]
    fn get_swap_and_account_metas(&self, swap_params: &SwapParams) -> SwapAndAccountMetas {
        // We need these accounts to be updated in order to create swap account metas
        if !self.has_program_data
            || self.price_histories.is_empty()
            || self.oracles.0 == Pubkey::default()
            || self.oracles.1 == Pubkey::default()
        {
            throw!(RequiredAccountUpdate);
        }

        let (mint_in, mint_out, input_token_price_history, output_token_price_history) =
            if swap_params.source_mint == self.mints.0 {
                (
                    self.mints.0,
                    self.mints.1,
                    self.price_histories[0],
                    self.price_histories[1],
                )
            } else {
                (
                    self.mints.1,
                    self.mints.0,
                    self.price_histories[1],
                    self.price_histories[0],
                )
            };
        let fee_destination = if swap_params.source_mint == self.mints.1 {
            self.fee_destination.0
        } else {
            self.fee_destination.1
        };

        SwapAndAccountMetas {
            swap: Swap::GooseFX,
            account_metas: get_account_metas_for_swap(
                self.pool_registry,
                swap_params.token_transfer_authority,
                mint_in,
                mint_out,
                input_token_price_history,
                output_token_price_history,
                fee_destination,
            ),
        }
    }

    /// Clone this object in a [Box].
    fn clone_amm(&self) -> Box<dyn Amm + Send + Sync> {
        Box::new(self.clone())
    }
}
