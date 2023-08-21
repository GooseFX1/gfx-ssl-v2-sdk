use crate::pubkey_str::pubkey;
use anchor_lang::prelude::Pubkey;
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
            max_pool_token_ratio: self.max_pool_token_ratio,
            std_weight: self.std_weight as u32,
            latest_price_weight: self.latest_price_weight,
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
