use rust_decimal::Decimal;
use serde::Serialize;
use solana_sdk::pubkey::Pubkey;
use gfx_ssl_v2_interface::{OracleType, HistoricalPrice, OraclePriceHistory};
use crate::display::mint_ui_name;
use crate::pubkey_str::pubkey;

#[derive(Serialize, Clone)]
pub struct HistoricalPriceRaw {
    value: i64,
    scale: u32,
    slot: u64,
}

impl From<&HistoricalPrice> for HistoricalPriceRaw {
    fn from(value: &HistoricalPrice) -> Self {
        Self {
            value: value.price.num,
            scale: value.price.scale,
            slot: value.slot,
        }
    }
}

#[derive(Serialize, Clone)]
pub struct HistoricalPriceUi (String, u64);

impl From<&HistoricalPrice> for HistoricalPriceUi {
    fn from(value: &HistoricalPrice) -> Self {
        let price: Decimal = value.price.into();
        Self(price.to_string(), value.slot)
    }
}

#[derive(Serialize, Clone)]
pub struct OraclePriceHistoryRawData {
    #[serde(with = "pubkey")]
    pub address: Pubkey,
    pub oracle_type: u8,
    pub minimum_elapsed_slots: u8,
    pub max_slot_price_staleness: u8,
    #[serde(with = "pubkey")]
    pub pool_registry: Pubkey,
    #[serde(with = "pubkey")]
    pub oracle_address: Pubkey,
    #[serde(with = "pubkey")]
    pub mint: Pubkey,
    pub num_updates: u64,
    /// Ordered the same as stored
    pub price_history: Vec<HistoricalPriceRaw>,
}

impl From<&(Pubkey, OraclePriceHistory)> for OraclePriceHistoryRawData {
    fn from((address, act): &(Pubkey, OraclePriceHistory)) -> Self {
        Self {
            address: *address,
            oracle_type: act.oracle_type,
            minimum_elapsed_slots: act.minimum_elapsed_slots,
            max_slot_price_staleness: act.max_slot_price_staleness,
            pool_registry: act.pool_registry,
            oracle_address: act.oracle_address,
            mint: act.mint,
            num_updates: act.num_updates,
            price_history: act.price_history
                .iter()
                .map(|p| HistoricalPriceRaw::from(p))
                .collect(),
        }
    }
}

#[derive(Serialize, Clone)]
pub struct OraclePriceHistoryUiData {
    #[serde(with = "pubkey")]
    pub address: Pubkey,
    pub oracle_type: OracleType,
    pub minimum_elapsed_slots: u8,
    pub max_slot_price_staleness: u8,
    #[serde(with = "pubkey")]
    pub pool_registry: Pubkey,
    #[serde(with = "pubkey")]
    pub oracle_address: Pubkey,
    #[serde(with = "pubkey")]
    pub mint: Pubkey,
    pub mint_name: Option<String>,
    pub num_updates: u64,
    /// Ordered from newest to oldest
    pub price_history: Vec<HistoricalPriceUi>,
}

impl From<&(Pubkey, OraclePriceHistory)> for OraclePriceHistoryUiData {
    fn from((address, act): &(Pubkey, OraclePriceHistory)) -> Self {
        let mut price_history: Vec<HistoricalPriceUi> = act.price_history
            .iter()
            .map(|p| HistoricalPriceUi::from(p))
            .collect();
        price_history.sort_by_key(|p| {
            p.1
        });
        price_history.reverse();
        Self {
            address: *address,
            oracle_type: OracleType::from(act.oracle_type),
            minimum_elapsed_slots: act.minimum_elapsed_slots,
            max_slot_price_staleness: act.max_slot_price_staleness,
            pool_registry: act.pool_registry,
            oracle_address: act.oracle_address,
            mint: act.mint,
            mint_name: mint_ui_name(act.mint),
            num_updates: act.num_updates,
            price_history,
        }
    }
}
