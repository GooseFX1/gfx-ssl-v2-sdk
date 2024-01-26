use crate::Pair;
use anchor_lang::prelude::Pubkey;
use solana_sdk::pubkey;

pub const MAINNET_POOL_REGISTRY: Pubkey = pubkey!("F451mjRqGEu1azbj46v4FuMEt1CacaPHQKUHzuTqKp4R");

pub const GOFX_MINT: Pubkey = pubkey!("GFX1ZjR2P15tmrSwow6FjyDYcEkoFb4p4gJCpLBjaxHD");
pub const GOFX_NAME: &'static str = "GOFX";

pub const USDC_MINT: Pubkey = pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");
pub const USDC_NAME: &'static str = "USDC";

pub const BTC_MINT: Pubkey = pubkey!("9n4nbM75f5Ui33ZbPYXn59EwSgE8CGsHtAeTH5YFeJ9E");
pub const BTC_NAME: &'static str = "BTC";

pub const MSOL_MINT: Pubkey = pubkey!("mSoLzYCxHdYgdzU16g5QSh3i5K3z3KZK7ytfqcJm7So");
pub const MSOL_NAME: &'static str = "MSOL";

pub const BONK_MINT: Pubkey = pubkey!("DezXAZ8z7PnrnRJjz3wXBoRgixCa6xjnB7YaB1pPB263");
pub const BONK_NAME: &'static str = "BONK";

pub const WRAPPED_SOL_MINT: Pubkey = pubkey!("So11111111111111111111111111111111111111112");
pub const WRAPPED_SOL_NAME: &'static str = "SOL";

pub const JITO_SOL_MINT: Pubkey = pubkey!("J1toso1uCk3RLmjorhTtrVwY9HJ7X8V9yYac6Y7kGCPn");
pub const JITO_SOL_NAME: &'static str = "JITOSOL";

pub const USDT_MINT: Pubkey = pubkey!("Es9vMFrzaCERmJfrF4H2FYD4KCoNkY11McCe8BenwNYB");
pub const USDT_NAME: &'static str = "USDT";

pub fn mint_from_str(name: &str) -> Option<Pubkey> {
    match name.trim().to_uppercase().as_str() {
        "SOL" => Some(WRAPPED_SOL_MINT),
        "JITOSOL" => Some(JITO_SOL_MINT),
        "MSOL" => Some(MSOL_MINT),
        "BONK" => Some(BONK_MINT),
        "USDC" => Some(USDC_MINT),
        "USDT" => Some(USDT_MINT),
        "GOFX" => Some(GOFX_MINT),
        "BTC" => Some(BTC_MINT),
        _ => None,
    }
}

pub fn str_from_mint(pubkey: &Pubkey) -> Option<&'static str> {
    match *pubkey {
        WRAPPED_SOL_MINT => Some(WRAPPED_SOL_NAME),
        JITO_SOL_MINT => Some(JITO_SOL_NAME),
        MSOL_MINT => Some(MSOL_NAME),
        BONK_MINT => Some(BONK_NAME),
        USDC_MINT => Some(USDC_NAME),
        USDT_MINT => Some(USDT_NAME),
        GOFX_MINT => Some(GOFX_NAME),
        BTC_MINT => Some(BTC_NAME),
        _ => None,
    }
}

/// Takes a string like "SOL-USDC", and produces a Pair pubkey.
/// The order of the mints does not matter, it gets normalized.
pub fn pair_from_str(
    pool_registry_address: Pubkey,
    mints: &str,
) -> Option<(Pubkey, (Pubkey, Pubkey))> {
    mints
        .split_once("-")
        .and_then(|(mint_one, mint_two)| Some((mint_from_str(mint_one)?, mint_two)))
        .and_then(|(mint_one, mint_two)| Some((mint_one, mint_from_str(mint_two)?)))
        .map(|(mint_one, mint_two)| {
            (
                Pair::address(pool_registry_address, mint_one, mint_two),
                (mint_one, mint_two),
            )
        })
}
