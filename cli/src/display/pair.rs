use crate::{
    display::{mint_decimals, mint_ui_name, u128_ui_amount},
    pool_vault::{MainVault, MainVaultUiData, SecondaryVault, SecondaryVaultUiData},
    pubkey_str::{pubkey, pubkey_pair},
};
use gfx_ssl_v2_interface::{
    utils::{u128_from_bytes, u16_to_bps},
    Pair, PoolRegistry,
};
use serde::{self, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;

/// Scale used to record the historical USD volume swapped.
const USD_VOLUME_DECIMALS: u32 = 6;

pub struct PairAccountAndVaults {
    pub address: Pubkey,
    pub pair: Pair,
    pub mint_one_main_vault: MainVault,
    pub mint_one_secondary_vault: SecondaryVault,
    pub mint_two_main_vault: MainVault,
    pub mint_two_secondary_vault: SecondaryVault,
}

impl PairAccountAndVaults {
    pub fn from_rpc_client(
        address: Pubkey,
        pair: Pair,
        pool_registry: PoolRegistry,
        client: &RpcClient,
    ) -> anyhow::Result<Self> {
        let mint_one_main_vault =
            MainVault::from_rpc_client(pair.pool_registry, &pool_registry, pair.mints.0, client)?;
        let mint_one_secondary_vault = SecondaryVault::from_rpc_client(
            pair.pool_registry,
            &pool_registry,
            pair.mints.0,
            pair.mints.1,
            client,
        )?;
        let mint_two_main_vault =
            MainVault::from_rpc_client(pair.pool_registry, &pool_registry, pair.mints.1, client)?;
        let mint_two_secondary_vault = SecondaryVault::from_rpc_client(
            pair.pool_registry,
            &pool_registry,
            pair.mints.1,
            pair.mints.0,
            client,
        )?;
        Ok(Self {
            address,
            pair,
            mint_one_main_vault,
            mint_one_secondary_vault,
            mint_two_main_vault,
            mint_two_secondary_vault,
        })
    }
}

/// Raw data with serde traits, skipping padding and extra space fields
#[derive(Serialize, Clone)]
pub struct PairRawData {
    #[serde(with = "pubkey")]
    address: Pubkey,
    #[serde(with = "pubkey")]
    pool_registry: Pubkey,
    #[serde(with = "pubkey_pair")]
    mints: (Pubkey, Pubkey),
    #[serde(with = "pubkey_pair")]
    fee_collector: (Pubkey, Pubkey),
    normal_fee_rates: (u16, u16),
    preferred_fee_rates: (u16, u16),
    total_fees_generated_native: (u128, u128),
    total_historical_volume: u128,
    total_internally_swapped: (u128, u128),
    mint_one_main_vault: MainVault,
    mint_one_secondary_vault: SecondaryVault,
    mint_two_main_vault: MainVault,
    mint_two_secondary_vault: SecondaryVault,
}

impl From<&PairAccountAndVaults> for PairRawData {
    fn from(value: &PairAccountAndVaults) -> Self {
        let total_fees_generated_native = {
            let first = u128_from_bytes(&value.pair.total_fees_generated_native.0);
            let second = u128_from_bytes(&value.pair.total_fees_generated_native.0);
            (first, second)
        };
        let total_historical_volume = u128_from_bytes(&value.pair.total_historical_volume);
        let total_internally_swapped = {
            let first = u128_from_bytes(&value.pair.total_internally_swapped.0);
            let second = u128_from_bytes(&value.pair.total_internally_swapped.0);
            (first, second)
        };
        Self {
            address: value.address,
            pool_registry: value.pair.pool_registry,
            mints: value.pair.mints,
            fee_collector: value.pair.fee_collector,
            normal_fee_rates: value.pair.normal_fee_rates,
            preferred_fee_rates: value.pair.preferred_fee_rates,
            total_fees_generated_native,
            total_historical_volume,
            total_internally_swapped,
            mint_one_main_vault: value.mint_one_main_vault,
            mint_one_secondary_vault: value.mint_one_secondary_vault,
            mint_two_main_vault: value.mint_two_main_vault,
            mint_two_secondary_vault: value.mint_two_secondary_vault,
        }
    }
}

/// User-friendly values
#[derive(Serialize, Clone)]
pub struct PairUiData {
    #[serde(with = "pubkey")]
    address: Pubkey,
    #[serde(with = "pubkey")]
    pool_registry: Pubkey,
    total_historical_volume: Option<String>,
    mint_one: PairSideUiData,
    mint_two: PairSideUiData,
}

#[derive(Serialize, Clone)]
pub struct PairSideUiData {
    #[serde(with = "pubkey")]
    mint: Pubkey,
    mint_name: Option<String>,
    #[serde(with = "pubkey")]
    fee_collector: Pubkey,
    normal_fee_rate: String,
    preferred_fee_rate: String,
    total_fees_generated_native: Option<String>,
    total_internally_swapped: Option<String>,
    main_vault: MainVaultUiData,
    secondary_vault: SecondaryVaultUiData,
}

impl From<&PairAccountAndVaults> for PairUiData {
    fn from(value: &PairAccountAndVaults) -> Self {
        let total_fees_generated_native = {
            let first = u128_from_bytes(&value.pair.total_fees_generated_native.0);
            let second = u128_from_bytes(&value.pair.total_fees_generated_native.1);
            (first, second)
        };
        let total_historical_volume = u128_from_bytes(&value.pair.total_historical_volume);
        let total_internally_swapped = {
            let first = u128_from_bytes(&value.pair.total_internally_swapped.0);
            let second = u128_from_bytes(&value.pair.total_internally_swapped.1);
            (first, second)
        };
        let mint_dec = mint_decimals(value.pair.mints.0);
        let mint_one = PairSideUiData {
            mint: value.pair.mints.0,
            mint_name: mint_ui_name(value.pair.mints.0),
            fee_collector: value.pair.fee_collector.0,
            normal_fee_rate: u16_to_bps(value.pair.normal_fee_rates.0).to_string(),
            preferred_fee_rate: u16_to_bps(value.pair.preferred_fee_rates.0).to_string(),
            total_fees_generated_native: u128_ui_amount(total_fees_generated_native.0, mint_dec),
            total_internally_swapped: u128_ui_amount(total_internally_swapped.0, mint_dec),
            main_vault: MainVaultUiData::from(&value.mint_one_main_vault),
            secondary_vault: SecondaryVaultUiData::from(&value.mint_one_secondary_vault),
        };
        let mint_dec = mint_decimals(value.pair.mints.1);
        let mint_two = PairSideUiData {
            mint: value.pair.mints.1,
            mint_name: mint_ui_name(value.pair.mints.1),
            fee_collector: value.pair.fee_collector.1,
            normal_fee_rate: u16_to_bps(value.pair.normal_fee_rates.1).to_string(),
            preferred_fee_rate: u16_to_bps(value.pair.preferred_fee_rates.1).to_string(),
            total_fees_generated_native: u128_ui_amount(total_fees_generated_native.1, mint_dec),
            total_internally_swapped: u128_ui_amount(total_internally_swapped.1, mint_dec),
            main_vault: MainVaultUiData::from(&value.mint_two_main_vault),
            secondary_vault: SecondaryVaultUiData::from(&value.mint_two_secondary_vault),
        };
        Self {
            address: value.address,
            pool_registry: value.pair.pool_registry,
            total_historical_volume: u128_ui_amount(
                total_historical_volume,
                Some(USD_VOLUME_DECIMALS),
            ),
            mint_one,
            mint_two,
        }
    }
}
