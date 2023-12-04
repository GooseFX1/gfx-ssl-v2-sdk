use crate::pubkey_str::{pubkey, pubkey_array};
use serde::{self, Serialize};
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use gfx_ssl_v2_interface::ssl_pool::MAX_NUM_ORACLES_PER_MINT;
use gfx_ssl_v2_interface::{AssetType, PoolRegistry, SSLPool, SSLPoolStatus};
use crate::display::math_params::{SSLMathParamsRawData, SSLMathParamsUiData};
use crate::display::{mint_ui_name, ui_amount};
use crate::pool_vault::{MainVault, MainVaultUiData, SecondaryVault, SecondaryVaultUiData};

pub struct SSLPoolData {
    pub pool: SSLPool,
    pub main_vault: Option<MainVault>,
    pub secondary_vaults: Vec<SecondaryVault>,
}

impl SSLPoolData {
    pub fn from_rpc_client(
        pool: SSLPool,
        pool_registry_address: Pubkey,
        pool_registry: PoolRegistry,
        client: &RpcClient,
    ) -> Self {
        let main_vault = MainVault::from_rpc_client(
            pool_registry_address,
            &pool_registry,
            pool.mint,
            client,
        ).ok();
        let other_pools: Vec<SSLPool> = pool_registry
            .entries
            .into_iter()
            .filter(|other_pool| *other_pool != SSLPool::default())
            .filter(|other_pool| other_pool.mint != pool.mint)
            .collect();
        let secondary_vaults: Vec<SecondaryVault> = other_pools
            .iter()
            .flat_map(|other_pool| {
                SecondaryVault::from_rpc_client(
                    pool_registry_address,
                    &pool_registry,
                    pool.mint,
                    other_pool.mint,
                    client,
                )
            })
            .collect();
        Self {
            pool,
            main_vault,
            secondary_vaults,
        }
    }
}

/// Raw data with serde traits, skipping padding and extra space fields
#[derive(Serialize, Clone)]
pub struct SSLPoolRawData {
    pub status: u8,
    pub asset_type: u8,
    #[serde(with = "pubkey")]
    pub mint: Pubkey,
    pub mint_decimals: u8,
    pub bump: u8,
    pub total_accumulated_lp_reward: u64,
    pub total_liquidity_deposits: u64,
    #[serde(with = "pubkey_array")]
    pub oracle_price_histories: [Pubkey; MAX_NUM_ORACLES_PER_MINT],
    pub math_params: SSLMathParamsRawData,
    pub main_vault: Option<MainVault>,
    pub secondary_vaults: Vec<SecondaryVault>,
}

impl From<&SSLPoolData> for SSLPoolRawData {
    fn from(value: &SSLPoolData) -> Self {
        Self {
            status: value.pool.status,
            asset_type: value.pool.asset_type,
            mint: value.pool.mint,
            mint_decimals: value.pool.mint_decimals,
            bump: value.pool.bump,
            total_accumulated_lp_reward: value.pool.total_accumulated_lp_reward,
            total_liquidity_deposits: value.pool.total_liquidity_deposits,
            oracle_price_histories: value.pool.oracle_price_histories,
            math_params: SSLMathParamsRawData::from(&value.pool.math_params),
            main_vault: value.main_vault,
            secondary_vaults: value.secondary_vaults.clone(),
        }
    }
}

/// User-friendly values
#[derive(Serialize, Clone)]
pub struct SSLPoolUiData {
    pub status: SSLPoolStatus,
    pub asset_type: AssetType,
    #[serde(with = "pubkey")]
    pub mint: Pubkey,
    pub mint_name: Option<String>,
    pub mint_decimals: u8,
    pub total_accumulated_lp_reward: Option<String>,
    pub total_liquidity_deposits: Option<String>,
    #[serde(with = "pubkey_array")]
    pub oracle_price_histories: [Pubkey; MAX_NUM_ORACLES_PER_MINT],
    pub math_params: SSLMathParamsUiData,
    pub main_vault: Option<MainVaultUiData>,
    pub secondary_vaults: Vec<SecondaryVaultUiData>,
}

impl From<&SSLPoolData> for SSLPoolUiData {
    fn from(value: &SSLPoolData) -> Self {
        let total_accumulated_lp_reward = ui_amount(
            value.pool.total_accumulated_lp_reward,
            Some(value.pool.mint_decimals as u32),
        );
        let total_liquidity_deposits = ui_amount(
            value.pool.total_liquidity_deposits,
            Some(value.pool.mint_decimals as u32),
        );
        Self {
            status: SSLPoolStatus::from(value.pool.status),
            asset_type: AssetType::from(value.pool.asset_type),
            mint: value.pool.mint,
            mint_name: mint_ui_name(value.pool.mint),
            mint_decimals: value.pool.mint_decimals,
            total_accumulated_lp_reward,
            total_liquidity_deposits,
            oracle_price_histories: value.pool.oracle_price_histories,
            math_params: SSLMathParamsUiData::from(&value.pool.math_params),
            main_vault: value.main_vault.map(|v| MainVaultUiData::from(&v)),
            secondary_vaults: value.secondary_vaults
                .iter()
                .map(|v| SecondaryVaultUiData::from(v))
                .collect(),
        }
    }
}
