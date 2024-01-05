use std::{cell::RefCell, collections::HashMap, fmt::Debug};

use anchor_lang::{
    prelude::{Clock, UpgradeableLoaderState},
    AccountDeserialize, InstructionData,
};
use anchor_spl::associated_token::get_associated_token_address;
use anyhow::Error;
use bytemuck::bytes_of;
use fehler::{throw, throws};
use gfx_ssl_v2_sdk::state::{BollingerBand, OraclePriceHistory, Pair, PoolRegistry, SSLPool};
use jupiter_amm_interface::{
    AccountMap, Amm, KeyedAccount, Quote, QuoteParams, Swap, SwapAndAccountMetas, SwapParams,
};
use rust_decimal::Decimal;
use solana_bpf_simulator::SBPFInstructionExecutor;
use solana_program_runtime::log_collector::LogCollector;
use solana_sdk::{
    account::{AccountSharedData, ReadableAccount},
    account_utils::StateMut,
    pubkey::Pubkey,
    sysvar::clock,
};

use crate::tuple::Tuple;
use crate::{error::GfxJupiterIntegrationError::*, swap_account_metas::get_account_metas_for_swap};

type Epoch = u64; // Assuming 10 account updates for each account per s, u64 can be used for 5B years

/// Struct that implements the `jupiter_core::amm::Amm` trait.
#[derive(Debug, Clone)]
pub struct GfxAmm {
    pair: Pubkey,

    pub log: bool,

    mints: Tuple<2, Pubkey>,

    pool_registry: Pubkey,
    fee_rates: Tuple<2, u16>,
    fee_destination: Tuple<2, Pubkey>,
    oracles: Tuple<2, Pubkey>,

    price_histories: Tuple<2, Pubkey>, // this will get updated once pool_registry is updated
    mean_windows: Tuple<2, usize>,     // this will get updated once pool_registry is updated
    std_windows: Tuple<2, usize>,      // this will get updated once pool_registry is updated
    bbands: Tuple<2, BollingerBand<f64>>, // this will get updated once two price history is updated
    program_data_address: Pubkey,

    locs: HashMap<Pubkey, Tuple<2, usize>>,
    epoch: Epoch,
    accounts: HashMap<Pubkey, Option<(AccountSharedData, Epoch)>>,
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

    fn epoch(&mut self) -> Epoch {
        let ret = self.epoch;
        self.epoch += 1;
        ret
    }
}

impl Amm for GfxAmm {
    fn from_keyed_account(pair: &KeyedAccount) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let mut accounts = HashMap::new();
        let mut locs = HashMap::new();

        accounts.insert(clock::ID, None); // Can be lifted if a slot is passed in
        accounts.insert(gfx_ssl_v2_sdk::ID, None);

        let pair_pubkey = pair.key;
        accounts.insert(pair.key, Some((pair.account.clone().into(), 1)));
        locs.insert(pair.key, (0, 0).into());

        let data = &pair.account.data;
        let pair: Pair = Pair::try_deserialize(&mut data.as_slice())
            .map_err(|_| DeserializeFailure(pair_pubkey, "Pair".to_string()))?;

        accounts.insert(pair.pool_registry, None);
        locs.insert(pair.pool_registry, (1, 1).into());

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
        locs.insert(main_vaults.0, (4, 2).into());
        locs.insert(main_vaults.1, (2, 4).into());

        let secondary_vaults = (
            get_associated_token_address(&ssl_signers.0, &mints.1),
            get_associated_token_address(&ssl_signers.1, &mints.0),
        );
        accounts.insert(secondary_vaults.0, None);
        accounts.insert(secondary_vaults.1, None);
        locs.insert(secondary_vaults.0, (5, 3).into());
        locs.insert(secondary_vaults.1, (3, 5).into());

        let (_, fee_destination_a, _) = pair
            .find_fee_attrs(mints.0, mints.1)
            .map_err(|_| CannotResolveFeeDestination)?;
        let (_, fee_destination_b, _) = pair
            .find_fee_attrs(mints.1, mints.0)
            .map_err(|_| CannotResolveFeeDestination)?;

        Ok(Self {
            log: false,
            pair: pair_pubkey,
            pool_registry: pair.pool_registry,
            price_histories: Tuple::default(),
            mean_windows: Tuple::default(),
            std_windows: Tuple::default(),
            bbands: Tuple::default(),
            program_data_address: Pubkey::default(),
            fee_destination: (fee_destination_a, fee_destination_b).into(),
            mints: mints.into(),
            fee_rates: pair.fee_rates.into(),
            accounts,
            locs,
            epoch: 2,

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
                    if let Some((account, _)) = self.accounts.get(pubkey).expect("No way") {
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

                    if i == 0 {
                        self.locs
                            .insert(ssl.oracle_price_histories[0], (8, 6).into());
                    } else {
                        self.locs
                            .insert(ssl.oracle_price_histories[0], (6, 8).into());
                    }
                    self.price_histories[i] = ssl.oracle_price_histories[0];
                    self.mean_windows[i] = ssl.math_params.mean_window as usize;
                    self.std_windows[i] = ssl.math_params.std_window as usize;
                }
            } else if let Some(i) = self.price_histories.iter().position(|k| k == pubkey) {
                let history_i = OraclePriceHistory::try_deserialize(&mut account.data.as_slice())
                    .map_err(|_| {
                    DeserializeFailure(*pubkey, "OraclePriceHistory".to_string())
                })?;

                if !self.accounts.contains_key(&history_i.oracle_address) {
                    self.accounts.insert(history_i.oracle_address, None);
                    if i == 0 {
                        self.locs.insert(history_i.oracle_address, (9, 7).into());
                    } else {
                        self.locs.insert(history_i.oracle_address, (7, 9).into());
                    }
                }
                self.oracles[i] = history_i.oracle_address;

                let j = 1 - i;

                if let Some(Some(account_j)) = self.accounts.get(&self.price_histories[j]) {
                    let history_j = OraclePriceHistory::try_deserialize(&mut account_j.0.data())
                        .map_err(|_| {
                            DeserializeFailure(*pubkey, "OraclePriceHistory".to_string())
                        })?;

                    let bb_i = history_j
                        .bollinger_band(self.mean_windows[j], self.std_windows[j], &history_i)
                        .map_err(|_| MathError)?;

                    let bb_j = history_i
                        .bollinger_band(self.mean_windows[i], self.std_windows[i], &history_j)
                        .map_err(|_| MathError)?;

                    if i == 0 {
                        self.bbands = (bb_i, bb_j).into();
                    } else {
                        self.bbands = (bb_i, bb_j).into();
                    }
                }
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
                self.accounts.remove(&self.program_data_address);
                self.accounts.insert(programdata_address, None);
                self.program_data_address = programdata_address;
            }

            // Update the account
            let epoch = self.epoch();
            let Some(maybe_existing) = self.accounts.get_mut(pubkey) else {
                continue;
            };
            let mut account = account.clone();
            if pubkey == &self.program_data_address {
                account.data =
                    account.data[UpgradeableLoaderState::size_of_programdata_metadata()..].to_vec();
            }
            *maybe_existing = Some((account.into(), epoch));
        }
    }

    /// Get a GooseFX SSL swap quote
    #[throws(Error)]
    fn quote(&self, quote_params: &QuoteParams) -> Quote {
        fn create_vm() -> (SBPFInstructionExecutor<(usize, usize)>, Epoch) {
            // Can increase if 10k is not enough.
            let vm = SBPFInstructionExecutor::new(40, (10, 10240)).expect("Cannot create VM");

            (vm, 0)
        }

        thread_local! {
            pub static EXECUTOR: Tuple<2, RefCell<(SBPFInstructionExecutor<(usize, usize)>, Epoch)>> = (
                RefCell::new(create_vm()), RefCell::new(create_vm())
            ).into();
        }

        self.ready()?;

        let a_to_b = quote_params.input_mint == self.mints[0];

        // This can be removed if the BPF impl is fast enough
        let bband = if a_to_b {
            self.bbands[0]
        } else {
            self.bbands[1]
        };

        let ix = gfx_ssl_v2_sdk::anchor::instruction::Quote {
            amount_in: quote_params.amount,
            bband: Some(bytes_of(&bband).to_vec()),
        }
        .data();

        let (result, data, logger) = EXECUTOR.with(|exe| {
            let mut refmut = if a_to_b {
                exe[0].borrow_mut()
            } else {
                exe[1].borrow_mut()
            };
            let refmut = &mut *refmut;

            let (vm, vm_epoch) = (&mut refmut.0, &mut refmut.1);
            let mut new_epoch = *vm_epoch;

            if self.log {
                *vm.context_mut().log_collector_mut() = Some(LogCollector::new_ref());
            }
            vm.update_instruction(&ix)?;
            for (&key, maybe_account) in &self.accounts {
                let &Some((ref account, account_epoch)) = maybe_account else {
                    throw!(RequiredAccountUpdate);
                };

                if account_epoch <= *vm_epoch {
                    continue;
                }

                if key == self.program_data_address {
                    vm.update_program(&gfx_ssl_v2_sdk::ID, account, true)?;
                } else if key == clock::ID {
                    let clock: Clock = bincode::deserialize(&account.data())?;
                    vm.context_mut().sysvars_mut().set_clock(clock);
                } else if key == gfx_ssl_v2_sdk::ID {
                } else {
                    let loc = self.locs.get(&key).ok_or(RequiredAccountUpdate)?;
                    vm.update_account(
                        if a_to_b { loc[0] } else { loc[1] },
                        &key,
                        account,
                        false,
                        false,
                        false,
                    )?;
                }

                new_epoch = new_epoch.max(account_epoch);
            }

            *vm_epoch = new_epoch;

            let result = vm.execute();
            let data = vm.get_return_data().cloned();
            let logs = vm.context_mut().log_collector_mut().take();

            Result::<_, Error>::Ok((result, data, logs))
        })?;

        if let Some(logger) = logger {
            let logs = logger.borrow().get_recorded_content().to_vec();
            println!("Logs {:?}", logs);
        }

        let _ = result?;

        let Some((_, data)) = data else {
            throw!(MissingQuoteReturn)
        };

        if data.len() != 16 {
            throw!(MissingQuoteReturn);
        }

        let output: u64 = u64::from_le_bytes(data[..8].try_into().unwrap());
        let fee: u64 = u64::from_le_bytes(data[8..16].try_into().unwrap());

        let fee_pct = if a_to_b {
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
        quote
    }

    /// Get account metas for a GFX swap instruction,
    /// and marker denoting a [SwapLeg::Swap], and a [Swap::GooseFX].
    #[throws(Error)]
    fn get_swap_and_account_metas(&self, swap_params: &SwapParams) -> SwapAndAccountMetas {
        // We need these accounts to be updated in order to create swap account metas
        self.ready()?;

        let (mint_in, mint_out, input_token_price_history, output_token_price_history) =
            if swap_params.source_mint == self.mints[0] {
                (
                    self.mints[0],
                    self.mints[1],
                    self.price_histories[0],
                    self.price_histories[1],
                )
            } else {
                (
                    self.mints[1],
                    self.mints[0],
                    self.price_histories[1],
                    self.price_histories[0],
                )
            };
        let fee_destination = if swap_params.source_mint == self.mints[1] {
            self.fee_destination[0]
        } else {
            self.fee_destination[1]
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
