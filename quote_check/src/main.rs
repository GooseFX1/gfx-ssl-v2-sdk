use std::{collections::HashMap, fs::File, hash::Hash, io::Read, path::PathBuf, time::Duration};

use anchor_lang::prelude::UpgradeableLoaderState;
use anyhow::{anyhow, Error};
use base64::{prelude::BASE64_STANDARD, Engine};
use clap::Parser;
use fehler::{throw, throws};
use gfx_ssl_v2_jupiter::error::GfxJupiterIntegrationError;
use gfx_ssl_v2_quote_check::{serve_metrics, CHECKED_COUNT, ERROR_COUNT, MISMATCH_COUNT};
use gfx_ssl_v2_sdk::state::Pair;
use itertools::Itertools;
use jupiter_amm_interface::{Amm, KeyedAccount, QuoteParams};
use solana_account_decoder::{UiAccount, UiAccountData, UiAccountEncoding};
use solana_client::rpc_response::Response;
use solana_client::{nonblocking::rpc_client::RpcClient, rpc_config::RpcAccountInfoConfig};
use solana_client_async::prelude::{Client, ClientBuilder};
use solana_sdk::account_utils::StateMut;
use solana_sdk::transaction::TransactionError;
use solana_sdk::{account::Account, commitment_config::CommitmentConfig, pubkey, pubkey::Pubkey};
use tokio::spawn;
use tokio::{select, time::interval};
use tracing::{info, warn};
use tracing_subscriber::{
    filter::{EnvFilter, LevelFilter},
    fmt,
    prelude::*,
    Registry,
};
use url::Url;

const POOL_REGISTRY: Pubkey = pubkey!("F451mjRqGEu1azbj46v4FuMEt1CacaPHQKUHzuTqKp4R");

#[derive(Parser, Debug)]
struct Cli {
    #[arg(long, env, default_value = "https://api.mainnet-beta.solana.com")]
    solana_rpc: Url,

    #[arg(long, env, default_value = "wss://api.mainnet-beta.solana.com")]
    solana_ws: Url,

    #[arg(long)]
    program: Option<PathBuf>,

    #[arg(long)]
    mint: Vec<Pubkey>,

    #[arg(long, env, default_value = "9090")]
    metrics_port: u16,
}

type AmmObj = Box<dyn Amm>;

pub struct DoubleLinkedHashMap<K, V> {
    a: HashMap<K, V>,
    b: HashMap<V, K>,
}

impl<K, V> DoubleLinkedHashMap<K, V> {
    fn new() -> Self {
        Self {
            a: Default::default(),
            b: Default::default(),
        }
    }
}

impl<K, V> DoubleLinkedHashMap<K, V>
where
    K: Clone + Eq + Hash,
    V: Clone + Eq + Hash,
{
    fn insert(&mut self, key: K, value: V) {
        self.a.insert(key.clone(), value.clone());
        self.b.insert(value, key);
    }

    fn get_by_value(&self, value: &V) -> Option<&K> {
        self.b.get(value)
    }

    fn contains_key(&self, key: &K) -> bool {
        self.a.contains_key(key)
    }
}

#[allow(unreachable_code)]
#[throws(Error)]
#[tokio::main]
async fn main() {
    let subscriber = Registry::default()
        .with(
            fmt::layer()
                .with_writer(std::io::stdout)
                .with_filter(LevelFilter::INFO),
        )
        .with(EnvFilter::builder().try_from_env().unwrap_or_else(|_| {
            EnvFilter::builder()
                .parse("gfx_ssl_v2_quote_check=info")
                .unwrap()
        }));

    tracing::subscriber::set_global_default(subscriber).unwrap();

    let cli = Cli::parse();

    if cli.mint.is_empty() {
        throw!(anyhow!("Mint is empty"))
    }

    let mut mint_to_amount: HashMap<Pubkey, u64> = HashMap::new();
    mint_to_amount.insert(
        pubkey!("So11111111111111111111111111111111111111112"),
        1_000000000,
    );
    mint_to_amount.insert(
        pubkey!("EPjFWdd5AufqSSqeM2qN1xzybapC8G4wEGGkZwyTDt1v"),
        1_000000,
    );

    spawn(serve_metrics(cli.metrics_port));

    let client = RpcClient::new(cli.solana_rpc.to_string());

    let mut ws_client = ClientBuilder::new()
        .ws_url(cli.solana_ws.as_str())
        .header("origin", "https://app.goosefx.io")
        .build()
        .await?;

    let mut compares = vec![];

    for (mint_a, mint_b) in cli.mint.iter().tuple_combinations() {
        let pair_addr = Pair::address(POOL_REGISTRY, *mint_a, *mint_b);
        let account = client.get_account(&pair_addr).await?;
        let q = Box::new(gfx_ssl_v2_jupiter::jupiter::GfxAmm::from_keyed_account(
            &KeyedAccount {
                key: pair_addr,
                account: account.clone(),
                params: None,
            },
        )?);
        // q.log = true;
        let q = q as Box<dyn Amm>;
        let s = Box::new(gfx_ssl_v2_quote_check::GfxAmm::from_keyed_account(
            &KeyedAccount {
                key: pair_addr,
                account,
                params: None,
            },
        )?) as Box<dyn Amm>;

        compares.push((q, s));
    }

    info!("Check started");

    let mut tick = interval(Duration::from_secs(1));

    let mut accounts = HashMap::new();
    if let Some(local) = cli.program {
        let account = client.get_account(&gfx_ssl_v2_sdk::ID).await?;

        let state: UpgradeableLoaderState =
            account.state().expect("SSL Program is not upgradable?");
        let UpgradeableLoaderState::Program {
            programdata_address,
        } = state
        else {
            panic!("Not upgradable")
        };
        let mut data = bincode::serialize(&UpgradeableLoaderState::ProgramData {
            slot: 0,
            upgrade_authority_address: None,
        })
        .unwrap();
        data.resize(UpgradeableLoaderState::size_of_programdata_metadata(), 0);
        File::open(local).unwrap().read_to_end(&mut data).unwrap();

        accounts.insert(
            programdata_address,
            Account {
                owner: solana_sdk::bpf_loader_upgradeable::id(),
                lamports: 0,
                rent_epoch: 0,
                executable: true,
                data,
            },
        );
    }

    let mut subscriptions = DoubleLinkedHashMap::new();

    loop {
        select! {
            _ = tick.tick() => {
                check_quote(&mint_to_amount, &mut compares, &mut accounts, &mut subscriptions, &client, &mut ws_client).await?;
            }
            subs = ws_client.recv::<Response<UiAccount>>() => handle_subscription(&mut accounts, &subscriptions, subs?)?,
        }
    }
}

#[throws(Error)]
fn handle_subscription(
    accounts: &mut HashMap<Pubkey, Account>,
    subscriptions: &DoubleLinkedHashMap<Pubkey, u64>,
    (id, resp): (u64, Response<UiAccount>),
) {
    let account = resp.value;
    let UiAccountData::Binary(data, _) = account.data else {
        unreachable!()
    };
    let zstd_data = BASE64_STANDARD.decode(&data)?;

    let mut data = vec![];
    let mut reader = zstd::stream::read::Decoder::new(&*zstd_data)?;
    reader.read_to_end(&mut data)?;

    let key = subscriptions.get_by_value(&id).unwrap();

    accounts.insert(
        *key,
        Account {
            lamports: account.lamports,
            data,
            owner: account.owner.parse()?,
            executable: account.executable,
            rent_epoch: account.rent_epoch,
        },
    )
}

#[throws(Error)]
async fn check_quote(
    mint_to_amount: &HashMap<Pubkey, u64>,
    compares: &mut [(AmmObj, AmmObj)],
    accounts: &mut HashMap<Pubkey, Account>,
    subscriptions: &mut DoubleLinkedHashMap<Pubkey, u64>,
    rpc: &RpcClient,
    ws: &mut Client,
) {
    for (q, s) in compares {
        for m in [&mut **q, &mut **s] {
            let mut h = HashMap::new();
            for key in m.get_accounts_to_update() {
                if !subscriptions.contains_key(&key) {
                    let id = ws
                        .account_subscribe(
                            &key,
                            Some(RpcAccountInfoConfig {
                                encoding: Some(UiAccountEncoding::Base64Zstd),
                                data_slice: None,
                                commitment: Some(CommitmentConfig::finalized()),
                                min_context_slot: None,
                            }),
                        )
                        .await?
                        .await?;
                    subscriptions.insert(key, id);

                    // subscription needs a kick-start
                    if !accounts.contains_key(&key) {
                        let account = rpc.get_account(&key).await?;
                        accounts.insert(key, account);
                    }
                }

                h.insert(key, accounts[&key].clone());
            }

            m.update(&h)?;
        }

        for mints in q.get_reserve_mints().into_iter().permutations(2) {
            CHECKED_COUNT.inc();
            let (mint_in, mint_out) = (mints[0], mints[1]);
            if let Err(e) = check_quote_inner(&**q, &**s, mint_to_amount, mint_in, mint_out) {
                ERROR_COUNT.inc();
                if let Some(
                    GfxJupiterIntegrationError::RequiredAccountUpdate
                    | GfxJupiterIntegrationError::MissingQuoteReturn,
                ) = e.downcast_ref()
                {
                    continue;
                }

                if let Some(TransactionError::InstructionError(..)) = e.downcast_ref() {
                    warn!("Quote failed: {:?}", e);
                    continue;
                }

                warn!("Quote failed: {:?}", e);
            }
        }
    }
}

#[throws(Error)]
fn check_quote_inner(
    q: &dyn Amm,
    s: &dyn Amm,
    mint_to_amount: &HashMap<Pubkey, u64>,
    mint_in: Pubkey,
    mint_out: Pubkey,
) {
    let quote0 = q.quote(&QuoteParams {
        amount: mint_to_amount[&mint_in],
        input_mint: mint_in,
        output_mint: mint_out,
        swap_mode: Default::default(),
    })?;
    let quote1 = s.quote(&QuoteParams {
        amount: mint_to_amount[&mint_in],
        input_mint: mint_in,
        output_mint: mint_out,
        swap_mode: Default::default(),
    })?;
    if quote0.out_amount != quote1.out_amount {
        MISMATCH_COUNT.inc();
        warn!("Quote mismatch: {:?} vs {:?}", quote0, quote1);
    }
}
