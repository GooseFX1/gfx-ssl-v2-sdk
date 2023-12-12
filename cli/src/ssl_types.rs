use crate::pubkey_str::pubkey;
use anchor_lang::prelude::Pubkey;
use gfx_ssl_v2_interface::token_ratio_category;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Deserialize)]
pub struct CreateSSLParams {
    #[serde(with = "pubkey")]
    pub mint: Pubkey,
    pub asset_type: AssetType,
    #[serde(with = "pubkey")]
    pub oracle: Pubkey,
    pub oracle_type: OracleType,
    pub initial_deposit: u64,
    #[serde(deserialize_with = "Deserialize::deserialize")]
    pub number_of_slots_throttle: Option<u8>,
    #[serde(deserialize_with = "Deserialize::deserialize")]
    pub max_slot_price_staleness: Option<u8>,
    pub math_params: SSLMathParams,
}

#[derive(Debug, Copy, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OracleType {
    Pyth,
    Switchboard,
}

#[derive(Debug, Copy, Clone, PartialEq, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum AssetType {
    BlueChip,
    Volatile,
    Stable,
}

impl Into<gfx_ssl_v2_interface::AssetType> for AssetType {
    fn into(self) -> gfx_ssl_v2_interface::AssetType {
        match self {
            AssetType::BlueChip => gfx_ssl_v2_interface::AssetType::BlueChip,
            AssetType::Volatile => gfx_ssl_v2_interface::AssetType::Volatile,
            AssetType::Stable => gfx_ssl_v2_interface::AssetType::Stable,
        }
    }
}

/// Intended to be deserialized from a JSON file.
/// See program library for documentation on these fields.
#[derive(Copy, Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SSLMathParams {
    pub mean_window: u8,
    pub std_window: u8,
    pub fixed_price_distance: u16,
    pub minimum_price_distance: u16,
    pub max_pool_token_ratio: u16,
    pub std_weight: u16,
    pub latest_price_weight: u16,
}

impl Into<gfx_ssl_v2_interface::SSLMathParams> for SSLMathParams {
    fn into(self) -> gfx_ssl_v2_interface::SSLMathParams {
        gfx_ssl_v2_interface::SSLMathParams {
            mean_window: self.mean_window,
            std_window: self.std_window,
            fixed_price_distance: self.fixed_price_distance,
            minimum_price_distance: self.minimum_price_distance,
            std_weight: self.std_weight as u32,
            latest_price_weight: self.latest_price_weight,
            _deprecated: 0,
            _pad0: [0; 6],
            _space: [0; 32],
            _pad1: [0; 4],
        }
    }
}

/// JSON data for pair creation.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct PairInitializationParams(pub PairMintParams, pub PairMintParams);

/// JSON data for pair creation, one mint.
#[derive(Copy, Clone, Debug, Serialize, Deserialize)]
pub struct PairMintParams {
    #[serde(with = "pubkey")]
    pub mint: Pubkey,
    /// Token account of `self.mint`, external fee destination.
    #[serde(with = "pubkey")]
    pub fee_destination: Pubkey,
    /// In basis-points, max 10,000
    pub fee_bps: u16,
}

/// For Anchor instruction encoding.
#[derive(Clone, Debug, PartialEq, Deserialize)]
pub struct PoolRegistryConfig(Vec<MaxPoolTokenRatio>);

/// For Anchor instruction encoding.
#[derive(Clone, Debug, PartialEq, Deserialize)]
#[repr(C)]
pub struct MaxPoolTokenRatio {
    pub input_token: AssetType,
    pub output_token: AssetType,
    pub pool_token_ratio: u16,
}

impl Into<token_ratio_category::MaxPoolTokenRatio> for MaxPoolTokenRatio {
    fn into(self) -> token_ratio_category::MaxPoolTokenRatio {
        let input_token: gfx_ssl_v2_interface::AssetType = self.input_token.into();
        let output_token: gfx_ssl_v2_interface::AssetType = self.output_token.into();
        token_ratio_category::MaxPoolTokenRatio {
            input_token: input_token.into(),
            output_token: output_token.into(),
            pool_token_ratio: self.pool_token_ratio,
        }
    }
}

impl Into<gfx_ssl_v2_interface::PoolRegistryConfig> for PoolRegistryConfig {
    fn into(self) -> gfx_ssl_v2_interface::PoolRegistryConfig {
        gfx_ssl_v2_interface::PoolRegistryConfig {
            new_admin: None,
            new_suspend_admin: None,
            max_pool_token_ratios: self.0.into_iter().map(|r| r.into()).collect(),
        }
    }
}
