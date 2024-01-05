use std::{
    collections::{HashMap, HashSet},
    fs::File,
    io::Read,
    path::{Path, PathBuf},
    time::Instant,
};

use anchor_lang::prelude::UpgradeableLoaderState;
use anyhow::anyhow;
use clap::Parser;
use gfx_ssl_v2_jupiter::jupiter::GfxAmm;
use gfx_ssl_v2_sdk::state::Pair;
use jupiter_amm_interface::{Amm, KeyedAccount, Quote, QuoteParams};
use solana_client::rpc_client::RpcClient;
use solana_sdk::{account::Account, pubkey, pubkey::Pubkey};
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
    local: Option<&Path>,
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
    // gfx_amm.log = true;

    // Perform three account updates
    let mut updated = HashSet::new();
    update_accounts(&mut updated, &mut gfx_amm, client, local);
    update_accounts(&mut updated, &mut gfx_amm, client, local);
    update_accounts(&mut updated, &mut gfx_amm, client, local);

    let quote = gfx_amm.quote(&QuoteParams {
        amount,
        input_mint: mint_in,
        output_mint: mint_out,
        swap_mode: Default::default(),
    });

    let then = Instant::now();
    for _ in 0..10000 {
        gfx_amm.quote(&QuoteParams {
            amount,
            input_mint: mint_in,
            output_mint: mint_out,
            swap_mode: Default::default(),
        })?;
    }
    println!("{:#?}, elapsed: {:?}", quote, then.elapsed() / 10000);
    quote
}

pub fn update_accounts(
    updated: &mut HashSet<Pubkey>,
    gfx_amm: &mut GfxAmm,
    client: &RpcClient,
    local: Option<&Path>,
) {
    let accounts_to_update = gfx_amm.get_accounts_to_update();
    let accounts_map = HashMap::from_iter(
        accounts_to_update
            .into_iter()
            .filter(|key| updated.insert(*key))
            .map(|pubkey| (pubkey, client.get_account(&pubkey).unwrap()))
            .map(|(key, acc)| {
                if let Some(local) = local {
                    if key == pubkey!("DLY1NyXhDJd2xDw8Yj6P4jQXVSoUvbGZAPT5KzhWBVNq") {
                        let mut data = bincode::serialize(&UpgradeableLoaderState::ProgramData {
                            slot: 0,
                            upgrade_authority_address: None,
                        })
                        .unwrap();
                        data.resize(UpgradeableLoaderState::size_of_programdata_metadata(), 0);
                        File::open(local).unwrap().read_to_end(&mut data).unwrap();

                        (
                            key,
                            Account {
                                owner: pubkey!("BPFLoaderUpgradeab1e11111111111111111111111"),
                                lamports: 0,
                                rent_epoch: 0,
                                executable: false,
                                data,
                            },
                        )
                    } else {
                        (key, acc)
                    }
                } else {
                    (key, acc)
                }
            }),
    );
    gfx_amm.update(&accounts_map).unwrap();
}

#[derive(Parser)]
struct Cli {
    #[arg(long, env, default_value = "https://api.mainnet-beta.solana.com")]
    solana_rpc: Url,

    #[arg(long, env)]
    local: Option<PathBuf>,
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    let pair_addr = Pair::address(POOL_REGISTRY, SOL_MINT, USDC_MINT);

    let client = RpcClient::new(cli.solana_rpc.to_string());
    get_quote(
        1_000_000,
        pair_addr,
        USDC_MINT,
        SOL_MINT,
        &client,
        cli.local.as_deref(),
    )?;
    get_quote(
        1_000_000_000,
        pair_addr,
        SOL_MINT,
        USDC_MINT,
        &client,
        cli.local.as_deref(),
    )?;

    Ok(())
}
