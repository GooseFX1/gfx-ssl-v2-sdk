use crate::{
    display,
    display::{ui_amount, ui_timestamp},
    pubkey_str::pubkey,
};
use gfx_ssl_v2_interface::LiquidityAccount;
use serde::{Deserialize, Serialize};
use solana_sdk::pubkey::Pubkey;

/// Raw data with serde traits, skipping padding and extra space fields
#[derive(Serialize, Deserialize, Clone)]
pub struct LiquidityAccountRawData {
    #[serde(with = "pubkey")]
    address: Pubkey,
    #[serde(with = "pubkey")]
    pool_registry: Pubkey,
    #[serde(with = "pubkey")]
    mint: Pubkey,
    #[serde(with = "pubkey")]
    owner: Pubkey,
    amount_deposited: u64,
    last_observed_tap: u64,
    last_claimed: i64,
    total_earned: u64,
    created_at: i64,
}

impl From<&(Pubkey, LiquidityAccount)> for LiquidityAccountRawData {
    fn from((address, act): &(Pubkey, LiquidityAccount)) -> Self {
        Self {
            address: *address,
            pool_registry: act.pool_registry,
            mint: act.mint,
            owner: act.owner,
            amount_deposited: act.amount_deposited,
            last_observed_tap: act.last_observed_tap,
            last_claimed: act.last_claimed,
            total_earned: act.total_earned,
            created_at: act.created_at,
        }
    }
}

/// User-friendly values
#[derive(Serialize, Deserialize, Clone)]
pub struct LiquidityAccountUiData {
    #[serde(with = "pubkey")]
    address: Pubkey,
    #[serde(with = "pubkey")]
    pool_registry: Pubkey,
    #[serde(with = "pubkey")]
    mint: Pubkey,
    #[serde(with = "pubkey")]
    owner: Pubkey,
    mint_name: Option<String>,
    amount_deposited: Option<String>,
    last_observed_tap: Option<String>,
    total_earned: Option<String>,
    last_claimed: String,
    created_at: String,
}

impl From<&(Pubkey, LiquidityAccount)> for LiquidityAccountUiData {
    fn from((address, act): &(Pubkey, LiquidityAccount)) -> Self {
        let mint_name = display::mint_ui_name(act.mint);
        let mint_decimals = display::mint_decimals(act.mint);
        let amount_deposited = ui_amount(act.amount_deposited, mint_decimals);
        let total_earned = ui_amount(act.total_earned, mint_decimals);
        let last_claimed = ui_timestamp(act.last_claimed);
        let created_at = ui_timestamp(act.created_at);
        let last_observed_tap = ui_amount(act.last_observed_tap, mint_decimals);
        Self {
            address: *address,
            pool_registry: act.pool_registry,
            mint: act.mint,
            owner: act.owner,
            mint_name,
            amount_deposited,
            total_earned,
            last_claimed,
            created_at,
            last_observed_tap,
        }
    }
}
