pub mod liquidity_account;
pub mod pretty_printer;
pub mod math_params;
pub mod ssl_pool;
pub mod pool_registry;
pub mod pair;
pub mod oracle_price_history;

use chrono::{DateTime, NaiveDateTime, Utc};
use serde::Serialize;
use crate::pubkey_str::pubkey::Pubkey;
use gfx_ssl_v2_interface::utils::token_amount;
use serde_json::Value;
use solana_sdk::pubkey;

pub const MAINNET_POOL_REGISTRY: Pubkey = pubkey!("F451mjRqGEu1azbj46v4FuMEt1CacaPHQKUHzuTqKp4R");

pub const USDC_MINT: Pubkey = pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
pub const USDC_DECIMALS: u32 = 6;
pub const USDT_MINT: Pubkey = pubkey!("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB");
pub const USDT_DECIMALS: u32 = 6;
pub const BONK_MINT: Pubkey = pubkey!("DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263");
pub const BONK_DECIMALS: u32 = 5;
pub const MSOL_MINT: Pubkey = pubkey!("mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So");
pub const MSOL_DECIMALS: u32 = 9;
pub const SOL_MINT: Pubkey = pubkey!("So11111111111111111111111111111111111111112");
pub const SOL_DECIMALS: u32 = 9;
pub const JITOSOL_MINT: Pubkey = pubkey!("J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn");
pub const JITOSOL_DECIMALS: u32 = 9;

pub fn mint_ui_name(mint: Pubkey) -> Option<String> {
    match mint {
        USDC_MINT => Some(String::from("USDC")),
        USDT_MINT => Some(String::from("USDT")),
        BONK_MINT => Some(String::from("BONK")),
        MSOL_MINT => Some(String::from("MSOL")),
        SOL_MINT => Some(String::from("SOL")),
        JITOSOL_MINT => Some(String::from("JITOSOL")),
        _ => None,
    }
}

pub fn mint_decimals(mint: Pubkey) -> Option<u32> {
    match mint {
        USDC_MINT => Some(USDC_DECIMALS),
        USDT_MINT => Some(USDT_DECIMALS),
        BONK_MINT => Some(BONK_DECIMALS),
        MSOL_MINT => Some(MSOL_DECIMALS),
        SOL_MINT => Some(SOL_DECIMALS),
        JITOSOL_MINT => Some(JITOSOL_DECIMALS),
        _ => None,
    }
}

pub fn ui_amount(raw_amount: u64, mint_decimals: Option<u32>) -> Option<String> {
    mint_decimals
        .map(|decimals| token_amount::to_ui(raw_amount, decimals).to_string())
}

pub fn u128_ui_amount(raw_amount: u128, mint_decimals: Option<u32>) -> Option<String> {
    mint_decimals
        .map(|decimals| token_amount::u128_to_ui(raw_amount, decimals).to_string())
}

pub fn ui_timestamp(raw_timestamp: i64) -> String {
    DateTime::<Utc>::from_utc(
        NaiveDateTime::from_timestamp_opt(raw_timestamp, 0).unwrap(),
        Utc,
    ).to_string()
}

pub trait CliDisplay {
    fn to_json(&self) -> serde_json::Value;

    fn to_json_str_pretty(&self) -> String;

    fn cli_pretty_print(&self) -> String;
}

impl<T: Serialize> CliDisplay for T {
    fn to_json(&self) -> Value {
        serde_json::to_value(&self).unwrap()
    }

    fn to_json_str_pretty(&self) -> String {
        serde_json::to_string_pretty(&self).unwrap()
    }

    fn cli_pretty_print(&self) -> String {
        pretty_printer::cli_pretty_print(&self)
    }
}

/// Display either raw or UI data, JSON formatted or not.
pub fn cli_display<'a, T, Raw: Serialize + From<&'a T>, Ui: Serialize + From<&'a T>>(
    values: &'a [T],
    raw: bool,
    json: bool,
) -> Result<(), serde_json::Error> {
    if raw {
        let values = values
        .iter()
        .map(|act| Raw::from(act))
        .collect::<Vec<_>>();
        if json {
            println!("{}", serde_json::to_string_pretty(&values)?);
        } else {
            values.iter().for_each(|v| println!("{}", v.cli_pretty_print()));
        }
    } else {
        let values = values
        .iter()
        .map(|act| Ui::from(act))
        .collect::<Vec<_>>();
        if json {
            println!("{}", serde_json::to_string_pretty(&values)?);
        } else {
            values.iter().for_each(|v| println!("{}", v.cli_pretty_print()));
        }
    };
    Ok(())
}