use std::{
    cell::RefCell,
    collections::{hash_map, HashMap},
    fmt::Debug,
};

use anchor_lang::{
    prelude::{Clock, UpgradeableLoaderState},
    AccountDeserialize, InstructionData, ToAccountMetas,
};
use anchor_spl::associated_token::get_associated_token_address;
use anyhow::Error;
use bytemuck::bytes_of;
use fehler::{throw, throws};
use gfx_ssl_v2_sdk::state::{BollingerBand, OraclePriceHistory, Pair, PoolRegistry, SSLPool};
use jupiter_amm_interface::{
    AccountMap, Amm, KeyedAccount, Quote, QuoteParams, Swap, SwapAndAccountMetas, SwapParams,
};
use once_cell::sync::Lazy;
use rust_decimal::Decimal;
use solana_bpf_simulator::{SBFExecutor, WrappedSlot, FEATURES};
use solana_sdk::{
    account::{Account, AccountSharedData, ReadableAccount},
    account_utils::StateMut,
    bpf_loader_upgradeable,
    message::{LegacyMessage, Message, SanitizedMessage},
    native_loader,
    pubkey::Pubkey,
    system_program,
    sysvar::clock,
};

use crate::{
    error::GfxJupiterIntegrationError::*, swap_account_metas::get_account_metas_for_swap,
    tuple::Tuple,
};

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

static SYSTEM_PROGRAM: Lazy<AccountSharedData> = Lazy::new(|| {
    Account {
        owner: native_loader::ID,
        executable: true,
        rent_epoch: 96,
        data: b"solana_system_program".to_vec(),
        lamports: 1,
    }
    .into()
});

// Principle: For Tuple<2> the each element is when the corresponding mint is the input
// For Tuple<4>, element 0, 2 is when the first mint is the input
/// Struct that implements the `jupiter_core::amm::Amm` trait.
#[derive(Debug, Clone)]
pub struct GfxAmm {
    pair: Pubkey,

    mints: Tuple<2, Pubkey>,

    pool_registry: Pubkey,
    fee_rates: Tuple<2, u16>,
    fee_destination: Tuple<2, Pubkey>,
    main_vaults: Tuple<2, Pubkey>,
    secondary_vaults: Tuple<2, Pubkey>,
    oracles: Tuple<4, Pubkey>,

    price_histories: Tuple<4, Pubkey>, // this will get updated once pool_registry is updated
    mean_windows: Tuple<2, usize>, // this will get updated once pool_registry is updated. Note: swap uses output tokens math_params
    std_windows: Tuple<2, usize>, // this will get updated once pool_registry is updated. Note: swap uses output tokens math_params
    bbands: Tuple<4, BollingerBand<f64>>, // this will get updated once two price history is updated
    has_program_data: bool,

    accounts: HashMap<Pubkey, Option<AccountSharedData>>,
}

impl GfxAmm {
    #[throws(Error)]
    fn ready(&self) {
        if !self.has_program_data {
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
            price_histories: Tuple::default(),
            mean_windows: Tuple::default(),
            std_windows: Tuple::default(),
            bbands: Tuple::default(),
            has_program_data: false,
            fee_destination: [fee_destination_a, fee_destination_b].into(),
            mints: mints.into(),
            fee_rates: pair.fee_rates.into(),
            accounts,

            main_vaults: main_vaults.into(),
            secondary_vaults: secondary_vaults.into(),
            oracles: Tuple::default(),
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
    fn update(&mut self, account_map: &AccountMap) -> Result<(), Error> {
        for (pubkey, account) in account_map {
            let acc = match self.accounts.get_mut(pubkey) {
                Some(a) => a,
                None => continue,
            };
            *acc = Some(account.clone().into());

            if pubkey == &self.pool_registry {
                let pool_registry = PoolRegistry::try_deserialize(&mut account.data.as_slice())
                    .map_err(|_| DeserializeFailure(*pubkey, "PoolRegistry".to_string()))?;

                self.mints
                    .iter()
                    .enumerate()
                    .zip(&mut *self.mean_windows)
                    .zip(&mut *self.std_windows)
                    .try_for_each(|(((idx, &mint), mean_window), std_window)| {
                        let ssl: &SSLPool = pool_registry
                            .find_pool(mint)
                            .map_err(|_| PoolNotFound(mint))?;
                        const NORACLES: usize = 2;
                        for (oracle_idx, &h) in ssl.oracle_price_histories[..NORACLES]
                            .into_iter()
                            .enumerate()
                        {
                            if h == Pubkey::default() {
                                continue;
                            }
                            if let hash_map::Entry::Vacant(e) = self.accounts.entry(h) {
                                e.insert(None);
                            }

                            // 0 - 0 2
                            // 1 - 1 3
                            self.price_histories[idx + oracle_idx * NORACLES] = h;
                        }

                        *mean_window = ssl.math_params.mean_window as usize;
                        *std_window = ssl.math_params.std_window as usize;

                        Result::<_, Error>::Ok(())
                    })?;
            } else if self.price_histories.contains(&pubkey) {
                let history = OraclePriceHistory::try_deserialize(&mut account.data())
                    .map_err(|_| DeserializeFailure(*pubkey, "OraclePriceHistory".to_string()))?;

                if let hash_map::Entry::Vacant(e) = self.accounts.entry(history.oracle_address) {
                    e.insert(None);
                }

                let idx = self
                    .price_histories
                    .iter()
                    .position(|e| e == pubkey)
                    .unwrap(); // the index of the updated price history
                self.oracles[idx] = history.oracle_address;

                let win_idx = idx % self.mints.len(); // the index of the window. 0, 2 is the oracle for the first mint, the window is 0. Same for 1, 3.
                let counterparts = [
                    (idx + 1) % self.price_histories.len(),
                    (idx + 3) % self.price_histories.len(),
                ]; // The counter part of 0 (first mint, main oracle) is (1, 3), same for others.

                for cidx in counterparts {
                    if let Some(Some(account)) = self.accounts.get(&self.price_histories[cidx]) {
                        let history_ = OraclePriceHistory::try_deserialize(&mut account.data())
                            .map_err(|_| {
                                DeserializeFailure(*pubkey, "OraclePriceHistory".to_string())
                            })?;

                        // bband when idx is the input.
                        // e.g. bbands[0] = bband(0, 1) then bband(1, 0)
                        // e.g. bbands[1] = bband(1, 2) then bband(2, 1)
                        // e.g. bbands[2] = bband(2, 3) then bband(3, 2)
                        // e.g. bbands[3] = bband(3, 0) then bband(0, 3)

                        // output is history_
                        self.bbands[idx] = history_
                            .bollinger_band(
                                self.mean_windows[1 - win_idx],
                                self.std_windows[1 - win_idx],
                                &history,
                            )
                            .unwrap();

                        // output is history
                        self.bbands[cidx] = history
                            .bollinger_band(
                                self.mean_windows[win_idx],
                                self.std_windows[win_idx],
                                &history_,
                            )
                            .unwrap();
                    }
                }
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

        self.ready()?;

        let (price_histories, main_vaults, secondary_vaults, oracles, bband) =
            if quote_params.input_mint == self.mints[0] {
                (
                    self.price_histories,
                    self.main_vaults,
                    self.secondary_vaults,
                    self.oracles,
                    self.bbands.pick([0, 2]),
                )
            } else {
                (
                    self.price_histories.pick([1, 0, 3, 2]),
                    self.main_vaults.reverse(),
                    self.secondary_vaults.reverse(),
                    self.oracles.pick([1, 0, 3, 2]),
                    self.bbands.pick([1, 3]),
                )
            };

        let metas = gfx_ssl_v2_sdk::anchor::accounts::Quote {
            pair: self.pair,
            pool_registry: self.pool_registry,

            ssl_in_main_vault: main_vaults[0],
            ssl_in_secondary_vault: secondary_vaults[0],
            input_token_price_history: price_histories[0],
            input_token_oracle: oracles[0],
            backup_input_token_price_history: price_histories[2],
            backup_input_token_oracle: oracles[2],

            ssl_out_main_vault: main_vaults[1],
            ssl_out_secondary_vault: secondary_vaults[1],
            output_token_price_history: price_histories[1],
            output_token_oracle: oracles[1],
            backup_output_token_price_history: price_histories[3],
            backup_output_token_oracle: oracles[3],
        }
        .to_account_metas(None);

        let mut bband_data = bytes_of(&bband[0]).to_vec();
        bband_data.extend_from_slice(bytes_of(&bband[1]));
        let ix = solana_sdk::instruction::Instruction {
            program_id: gfx_ssl_v2_sdk::ID,
            accounts: metas,
            data: gfx_ssl_v2_sdk::anchor::instruction::Quote {
                amount_in: quote_params.amount,
                bband: Some(bband_data),
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

            let mut loader = sbf.loader(|&key| {
                if key == system_program::ID {
                    return Some(SYSTEM_PROGRAM.clone());
                }

                if key == bpf_loader_upgradeable::ID {
                    return Some(BPF_LOADER.clone());
                }

                self.accounts.get(&key).cloned().flatten()
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

            let fee_pct = if quote_params.input_mint == self.mints[0] {
                self.fee_rates[0]
            } else {
                self.fee_rates[1]
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
        self.ready()?;

        let (mints, price_histories, fee_destination) = if swap_params.source_mint == self.mints[0]
        {
            (self.mints, self.price_histories, self.fee_destination[1])
        } else {
            (
                self.mints.reverse(),
                self.price_histories.pick([1, 0, 3, 2]),
                self.fee_destination[0],
            )
        };

        SwapAndAccountMetas {
            swap: Swap::GooseFX,
            account_metas: get_account_metas_for_swap(
                self.pool_registry,
                swap_params.token_transfer_authority,
                mints[0],
                mints[1],
                price_histories[0],
                price_histories[1],
                price_histories[2],
                price_histories[3],
                fee_destination,
            ),
        }
    }

    /// Clone this object in a [Box].
    fn clone_amm(&self) -> Box<dyn Amm + Send + Sync> {
        Box::new(self.clone())
    }
}
