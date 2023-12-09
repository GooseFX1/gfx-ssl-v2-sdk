use std::{
    collections::{HashMap, HashSet},
    time::Instant,
};

use anyhow::anyhow;
use clap::Parser;
use gfx_ssl_v2_jupiter::jupiter::GfxAmm;
use gfx_ssl_v2_sdk::state::Pair;
use jupiter_amm_interface::{Amm, KeyedAccount, Quote, QuoteParams};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{pubkey, pubkey::Pubkey};
use url::Url;

const POOL_REGISTRY: Pubkey = pubkey!("F451mjRqGEu1azbj46v4FuMEt1CacaPHQKUHzuTqKp4R");

const SOL_MINT: Pubkey = pubkey!("So11111111111111111111111111111111111111112");
const USDC_MINT: Pubkey = pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v");

pub fn get_quote(
    amount: u64,
    pair_addr: Pubkey,
    mint_in: Pubkey,
    mint_out: Pubkey,
    client: &RpcClient,
) -> anyhow::Result<Quote> {
    let pair_account = client
        .get_account(&pair_addr)
        .map_err(|e| anyhow!("Failed to get pair account: {}", e))?;
    let keyed_account = KeyedAccount {
        key: pair_addr,
        account: pair_account,
        params: None,
    };
    let mut gfx_amm = GfxAmm::from_keyed_account(&keyed_account)
        .map_err(|e| anyhow!("Could not make GfxAmm instance from pair account: {}", e))?;

    // Perform two account updates
    let mut updated = HashSet::new();
    update_accounts(&mut updated, &mut gfx_amm, client);
    update_accounts(&mut updated, &mut gfx_amm, client);
    update_accounts(&mut updated, &mut gfx_amm, client);

    let quote = gfx_amm.quote(&QuoteParams {
        amount,
        input_mint: mint_in,
        output_mint: mint_out,
        swap_mode: Default::default(),
    });

    let then = Instant::now();
    for _ in 0..100 {
        gfx_amm.quote(&QuoteParams {
            amount,
            input_mint: mint_in,
            output_mint: mint_out,
            swap_mode: Default::default(),
        })?;
    }
    println!("{:#?}, elapsed: {:?}", quote, then.elapsed() / 100);
    quote
}

pub fn update_accounts(updated: &mut HashSet<Pubkey>, gfx_amm: &mut GfxAmm, client: &RpcClient) {
    let accounts_to_update = gfx_amm.get_accounts_to_update();
    let accounts_map = HashMap::from_iter(
        accounts_to_update
            .into_iter()
            .filter(|key| updated.insert(*key))
            .map(|pubkey| (pubkey, client.get_account(&pubkey).unwrap())),
    );
    gfx_amm.update(&accounts_map).unwrap();
}

#[derive(Parser)]
struct Cli {
    #[arg(long, env, default_value = "https://api.mainnet-beta.solana.com")]
    solana_rpc: Url,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let pair_addr = Pair::address(POOL_REGISTRY, SOL_MINT, USDC_MINT);

    let client = RpcClient::new(cli.solana_rpc.to_string());
    get_quote(1_000_000, pair_addr, USDC_MINT, SOL_MINT, &client)?;
    get_quote(1_000_000_000, pair_addr, SOL_MINT, USDC_MINT, &client)?;

    Ok(())
}
