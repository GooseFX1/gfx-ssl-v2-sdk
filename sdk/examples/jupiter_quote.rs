use std::collections::HashMap;
use anchor_client::solana_client::rpc_client::RpcClient;
use anyhow::anyhow;
use jupiter_amm_interface::{Amm, KeyedAccount, Quote, QuoteParams};
use solana_sdk::pubkey;
use solana_sdk::pubkey::Pubkey;
use gfx_ssl_v2::Pair;
use gfx_ssl_v2_sdk::jupiter::GfxAmm;

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
    let pair_account = client.get_account(&pair_addr)
        .map_err(|e| anyhow!("Failed to get pair account: {}", e))?;
    let keyed_account = KeyedAccount {
        key: pair_addr,
        account: pair_account,
        params: None,
    };
    let mut gfx_amm = GfxAmm::from_keyed_account(&keyed_account)
        .map_err(|e| anyhow!("Could not make GfxAmm instance from pair account: {}", e))?;

    // Perform two account updates
    update_accounts(&mut gfx_amm, client);
    update_accounts(&mut gfx_amm, client);

    gfx_amm.quote(
        &QuoteParams {
            in_amount: amount,
            input_mint: mint_in,
            output_mint: mint_out,
        }
    )
}

pub fn update_accounts(gfx_amm: &mut GfxAmm, client: &RpcClient) {
    let accounts_to_update = gfx_amm.get_accounts_to_update();
    let accounts_map = HashMap::from_iter(
        accounts_to_update
            .into_iter()
            .map(|pubkey| {
                (pubkey, client.get_account(&pubkey).unwrap())
            })
    );
    gfx_amm.update(&accounts_map).unwrap();
}

fn main() -> anyhow::Result<()> {
    let pair_addr = Pair::address(
        POOL_REGISTRY,
        SOL_MINT,
        USDC_MINT,
    );

    let client = RpcClient::new("https://api.mainnet-beta.solana.com");
    let quote = get_quote(
        1_000_000,
        pair_addr,
        USDC_MINT,
        SOL_MINT,
        &client,
    )?;

    println!("{:#?}", quote);

    Ok(())
}