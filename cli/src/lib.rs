pub mod pubkey_str;
mod ssl_types;

use anchor_lang::AccountDeserialize;
use anchor_spl::associated_token::get_associated_token_address;
use anchor_spl::token::{Mint, TokenAccount};
use anyhow::anyhow;
use clap::{IntoApp, Parser};
use gfx_ssl_v2_interface::{
    utils::token_amount,
    LiquidityAccount, OraclePriceHistory, Pair, PoolRegistry, SSLMathConfig, SSLPool,
};
use gfx_ssl_v2_sdk::instructions::*;
use gfx_ssl_v2_sdk::state::*;
use solana_client::rpc_client::RpcClient;
use solana_devtools_cli_config::{CommitmentArg, KeypairArg, UrlArg};
use solana_sdk::compute_budget::ComputeBudgetInstruction;
use solana_sdk::instruction::Instruction;
use solana_sdk::message::Message;
use solana_sdk::{pubkey, pubkey::Pubkey, transaction::Transaction};
use std::fs;

#[derive(Parser, Debug)]
pub enum Subcommand {
    /// Create a new pool registry. The `-k/--keypair` signer
    /// is hardcoded as the lamport funder for the new account.
    CreatePoolRegistry,
    /// Create a new SSL pool for a given pool registry.
    CreateSsl {
        /// Instead of executing a transaction, just print a base-58
        /// encoded transaction message, useful for multisig proposals.
        #[clap(long)]
        print_only: bool,
        /// Defaults to the registry derived from the pool admin.
        #[clap(parse(try_from_str=Pubkey::try_from))]
        pool_registry: Pubkey,
        /// Path to a JSON file containing the mathematical parameters
        /// used for price calculation.
        json_params_path: String,
    },
    /// Configure the parameters of an SSL pool.
    ConfigSsl {
        /// Instead of executing a transaction, just print a base-58
        /// encoded transaction message, useful for multisig proposals.
        #[clap(long)]
        print_only: bool,
        /// The pool registry which hosts the SSL pool being configured.
        #[clap(long, parse(try_from_str=Pubkey::try_from))]
        pool_registry: Pubkey,
        /// The mint of the SSL pool being targeted for configuration.
        #[clap(long, parse(try_from_str=Pubkey::try_from))]
        mint: Pubkey,
        /// If this flag is included, then suspend the SSL pool.
        suspend: bool,
        /// The window size for mean calculation.
        #[clap(long)]
        mean_window: Option<u8>,
        /// The window size for standard deviation calculation.
        #[clap(long)]
        std_window: Option<u8>,
        /// A BPS value, expressing a percentage of the price, added to the latest oracle price.
        #[clap(long)]
        fixed_price_distance: Option<u16>,
        /// A BPS value, expressing minimum total distance from the latest oracle price.
        #[clap(long)]
        minimum_price_distance: Option<u16>,
        /// A BPS value, expressing the maximum ratio of USD-normalized token value
        /// of any SSL pool's non-main token balance compared to the main token.
        #[clap(long)]
        max_pool_token_ratio: Option<u16>,
        /// A BPS value, expressing how much of the standard deviation to add
        /// to the price calculation.
        #[clap(long)]
        std_weight: Option<u32>,
        /// A BPS value, expressing what percentage of the latest price to add during the price
        /// calculation.
        #[clap(long)]
        latest_price_weight: Option<u16>,
    },
    /// Configure an admin address that has the limited privilege to suspend
    /// swaps on a given pool.
    /// This configuration instruction is permissioned to the pool registry admin.
    /// The pool registry admin itself still always has permission to suspend pools as well.
    /// The "suspend admin" exists to afford a more rapid response in case the need arises
    /// to stop allowing swaps on a given pool.
    ConfigSuspendAdmin {
        /// The pool registry to configure.
        #[clap(long, parse(try_from_str=Pubkey::try_from))]
        pool_registry: Pubkey,
        /// The address to assign as the "suspend admin".
        #[clap(long, parse(try_from_str=Pubkey::try_from))]
        suspend_admin: Pubkey,
    },
    /// Create a swap pair for a given pool registry.
    /// This requires that mints have already been added to the pool registry as SSL Pools.
    /// The pool registry admin is assumed to be the `-k/--keypair` signer.
    /// **NOTE**: The Pair account will not necessarily preserve the order of the mints
    /// as they appear in the JSON data. The program orders the addresses
    /// such that mint_one < mint_two.
    CreatePair {
        /// Instead of executing a transaction, just print a base-58
        /// encoded transaction message, useful for multisig proposals.
        #[clap(long)]
        print_only: bool,
        /// Defaults to the registry derived from the pool admin.
        #[clap(parse(try_from_str=Pubkey::try_from))]
        pool_registry: Pubkey,
        /// Path to a JSON file containing
        /// the mints and their related parameters
        pair_params_json_path: String,
    },
    /// Configure a swap pair for a given pool registry.
    /// To reduce the possibility of user error,
    /// this command only allows for configuring the parameters for a single mint
    /// at a time.
    ConfigPair {
        /// Instead of executing a transaction, just print a base-58
        /// encoded transaction message, useful for multisig proposals.
        #[clap(long)]
        print_only: bool,
        #[clap(parse(try_from_str=Pubkey::try_from))]
        pool_registry: Pubkey,
        /// The pair to configure
        #[clap(long, parse(try_from_str=Pubkey::try_from))]
        pair: Pubkey,
        /// The mint whose parameters to reconfigure
        #[clap(long, parse(try_from_str=Pubkey::try_from))]
        mint: Pubkey,
        /// Configures the pair to a new fee destination.
        #[clap(long, parse(try_from_str=Pubkey::try_from))]
        fee_destination: Option<Pubkey>,
        #[clap(long)]
        /// Configures the pair to a new fee BPS for the specified mint.
        fee_bps: Option<u16>,
    },
    /// Crank all price histories under a pool registry.
    /// This is compute-intensive and likely will not succeed
    /// if there are more than a couple pools.
    CrankAllPriceHistories {
        /// The pool registry whose histories to crank
        # [clap(parse(try_from_str=Pubkey::try_from))]
        pool_registry: Pubkey,
    },
    /// Crank specific price histories by mint.
    CrankPriceHistoriesPerMint {
        /// The pool registry whose histories to crank
        # [clap(parse(try_from_str=Pubkey::try_from))]
        pool_registry: Option<Pubkey>,
        /// The mint whose oracles needs to be cranked. You can supply this argument
        /// multiple times to multiple price histories.
        #[clap(long, parse(try_from_str=Pubkey::try_from))]
        mint: Vec<Pubkey>,
    },
    /// Configure the parameters of a price history account.
    ConfigPriceHistory {
        /// Instead of executing a transaction, just print a base-58
        /// encoded transaction message, useful for multisig proposals.
        #[clap(long)]
        print_only: bool,
        /// The address to the price history account. If you don't know it,
        /// use the `get-ssl-pool` command to find it.
        price_history: Pubkey,
        /// Used if the oracle needs to be throttled so that the price updates aren't too close to each other.
        #[clap(long)]
        number_of_slots_throttle: Option<u8>,
        /// Used to configure how many slots can pass before a price is considered stale
        #[clap(long)]
        max_slot_price_staleness: Option<u8>,
    },
    /// Execute an internal swap on a pair, which rebalances pools by
    /// swapping two pools' secondary token balances into each other's
    /// main vault.
    InternalSwap {
        /// Target pool registry that owns pools for `mint-one` and `mint-two`.
        #[clap(parse(try_from_str=Pubkey::try_from))]
        pool_registry: Pubkey,
        /// Secondary token on the `mint-two` pool is swapped into this mint's
        /// main token vault.
        /// Order does not matter.
        #[clap(parse(try_from_str=Pubkey::try_from))]
        mint_one: Pubkey,
        /// Secondary token on the `mint-one` pool is swapped into this mint's
        /// main token vault.
        /// Order does not matter.
        #[clap(parse(try_from_str=Pubkey::try_from))]
        mint_two: Pubkey,
    },
    /// User instruction to claim a portion of fees accrued
    /// for a given mint in proportion to the liquidity provided for that mint's pool.
    ClaimFees {
        /// Target pool registry for the given mint.
        #[clap(parse(try_from_str=Pubkey::try_from))]
        pool_registry: Pubkey,
        /// Fees are claimed in this mint.
        #[clap(parse(try_from_str=Pubkey::try_from))]
        mint: Pubkey,
    },
    /// User instruction to create a liquidity account for a given SSL pool
    /// as specified by pool registry and mint.
    /// This is a prerequisite for depositing liquidity to an SSL pool.
    CreateLiquidityAccount {
        /// Target pool registry for which to create a liquidity account.
        #[clap(parse(try_from_str=Pubkey::try_from))]
        pool_registry: Pubkey,
        /// Specifies the SSL pool for this mint.
        #[clap(parse(try_from_str=Pubkey::try_from))]
        mint: Pubkey,
    },
    /// User instruction to close a liquidity account.
    /// Liquidity deposit must be zero.
    CloseLiquidityAccount {
        /// Target pool registry for which to close the liquidity account.
        #[clap(parse(try_from_str=Pubkey::try_from))]
        pool_registry: Pubkey,
        /// Specifies the SSL pool by its main token mint.
        #[clap(parse(try_from_str=Pubkey::try_from))]
        mint: Pubkey,
        /// Defaults to the owner of the liquidity account.
        #[clap(long, parse(try_from_str=Pubkey::try_from))]
        rent_recipient: Option<Pubkey>,
    },
    /// User instruction to deposit liquidity to a pool
    /// as specified by mint.
    /// Requires that the user has created a liquidity pool.
    Deposit {
        /// Target pool registry that hosts the pool in which to deposit.
        #[clap(parse(try_from_str=Pubkey::try_from))]
        pool_registry: Pubkey,
        /// Specifies the SSL pool by its main token mint.
        #[clap(parse(try_from_str=Pubkey::try_from))]
        mint: Pubkey,
        /// Native token amount (i.e. satoshis, lamports, etc) to deposit.
        amount: u64,
    },
    /// User instruction to withdraw liquidity from a pool
    /// as specified by mint.
    Withdraw {
        /// Target pool registry that hosts the pool in which to deposit.
        #[clap(parse(try_from_str=Pubkey::try_from))]
        pool_registry: Pubkey,
        /// Specifies the SSL pool by its main token mint.
        #[clap(parse(try_from_str=Pubkey::try_from))]
        mint: Pubkey,
        /// Native token amount (i.e. satoshis, lamports, etc) to withdraw.
        amount: u64,
    },
    /// User swap instruction.
    /// Does not require that the user has a liquidity account.
    /// The amount that the user receives is determined by a pricing algorithm.
    Swap {
        /// Fail the instruction if the user receives less than this
        /// native amount (i.e. satoshies, lamports, etc) out.
        #[clap(long, default_value_t = 0)]
        min_out: u64,
        /// The mint that the user is relinquishing.
        #[clap(long = "in", parse(try_from_str=Pubkey::try_from))]
        mint_in: Pubkey,
        /// The mint that the user receives.
        #[clap(long = "out", parse(try_from_str=Pubkey::try_from))]
        mint_out: Pubkey,
        /// The pool registry that hosts the SSL pools used in the swap.
        #[clap(long, parse(try_from_str=Pubkey::try_from))]
        pool_registry: Pubkey,
        /// Native token amount (i.e. satoshis, lamports, etc) that the user
        /// is relinquishing. User must have at least this amount in their
        /// associated token account for `mint-in`.
        amount_in: u64,
    },
    /// Print the address for a pool registry PDA.
    GetPoolRegistryAddress {
        /// The seed address from which to derive the pool registry.
        /// The pool registry seed is set immutably on its account data
        /// and is equal to the address of the original pool admin
        /// that created the pool registry.
        /// Defaults to the address of the -k/--keypair argument
        /// or configured Solana CLI signer.
        #[clap(parse(try_from_str=Pubkey::try_from))]
        seed: Option<Pubkey>,
    },
    /// Print some of the special addresses associated with an SSL pool.
    /// Some of these addresses do not exist as accounts and only serve as PDA signers.
    /// This includes addresses such as the pool signer, vaults, and oracle price history.
    GetSSLAddresses {
        /// Pool registry address from which to derive the addresses.
        #[clap(parse(try_from_str=Pubkey::try_from))]
        pool_registry: Pubkey,
        /// Main token mint for the target SSL pool.
        #[clap(parse(try_from_str=Pubkey::try_from))]
        mint: Pubkey,
        /// Oracle address from which to derive a price history.
        /// If left unspecified, no price history address will be printed.
        #[clap(parse(try_from_str=Pubkey::try_from))]
        oracle: Option<Pubkey>,
    },
    /// Print an oracle price history address based on a pool registry and oracle.
    GetOraclePriceHistoryAddress {
        /// Pool registry address from which to derive the price history address.
        #[clap(parse(try_from_str=Pubkey::try_from))]
        pool_registry: Pubkey,
        /// Oracle address from which to derive a price history.
        #[clap(parse(try_from_str=Pubkey::try_from))]
        oracle: Pubkey,
    },
    /// Print the address for a pair account with the given mints.
    GetPairAddress {
        /// Pool registry address from which to derive the pair address.
        #[clap(parse(try_from_str=Pubkey::try_from))]
        pool_registry: Pubkey,
        /// One of the mints from which to derive the pair address.
        /// Order doesn't matter.
        #[clap(parse(try_from_str=Pubkey::try_from))]
        mint_one: Pubkey,
        /// One of the mints from which to derive the pair address.
        /// Order doesn't matter.
        #[clap(parse(try_from_str=Pubkey::try_from))]
        mint_two: Pubkey,
    },
    /// Print the main vault token address for an SSL Pool.
    GetSSLPoolVaultAddress {
        /// Pool registry address from which to derive the address.
        #[clap(parse(try_from_str=Pubkey::try_from))]
        pool_registry: Pubkey,
        /// The mint from which to derive an associated token account.
        #[clap(parse(try_from_str=Pubkey::try_from))]
        mint: Pubkey,
    },
    /// Print the address of a liquidity account for a given user and SSL pool.
    GetLiquidityAccountAddress {
        /// Pool registry address from which to derive the address.
        #[clap(parse(try_from_str=Pubkey::try_from))]
        pool_registry: Pubkey,
        #[clap(parse(try_from_str=Pubkey::try_from))]
        /// The pool main token mint from which to derive liquidity account.
        mint: Pubkey,
        /// The owner from which to derive the liquidity account.
        /// Defaults to the -k/--keypair argument or Solana CLI configured signer.
        #[clap(long, parse(try_from_str=Pubkey::try_from))]
        owner: Option<Pubkey>,
    },
    /// Display the account data on a pool registry.
    GetPoolRegistry {
        /// The pool registry address
        #[clap(parse(try_from_str=Pubkey::try_from))]
        address: Pubkey,
    },
    /// Display the account data for a specific SSL pool.
    /// Also displays the balances of the pool's vaults.
    GetSSLPool {
        /// The pool registry address
        #[clap(parse(try_from_str=Pubkey::try_from))]
        pool_registry: Pubkey,
        /// Identifies the SSL pool by its main token.
        #[clap(parse(try_from_str=Pubkey::try_from))]
        mint: Pubkey,
    },
    /// Display the account data for a Pair account.
    GetPair {
        /// The pool registry address
        #[clap(parse(try_from_str=Pubkey::try_from))]
        pool_registry: Pubkey,
        /// One of the mints from which to derive the pair address.
        /// Order doesn't matter.
        #[clap(parse(try_from_str=Pubkey::try_from))]
        mint_one: Pubkey,
        /// One of the mints from which to derive the pair address.
        /// Order doesn't matter.
        #[clap(parse(try_from_str=Pubkey::try_from))]
        mint_two: Pubkey,
    },
    /// Display the account data for an oracle price history account.
    GetOraclePriceHistory {
        /// Address of the price history account. Use `get-pool-registry`
        /// or `get-ssl-pool` to find the desired address.
        #[clap(parse(try_from_str=Pubkey::try_from))]
        address: Pubkey,
    },
    /// Display the account data for a liquidity account.
    GetLiquidityAccount {
        /// The pool registry address
        #[clap(parse(try_from_str=Pubkey::try_from))]
        pool_registry: Pubkey,
        /// The pool main token mint from which to derive liquidity account.
        #[clap(parse(try_from_str=Pubkey::try_from))]
        mint: Pubkey,
        /// The owner from which to derive the liquidity account.
        /// Defaults to the -k/--keypair argument or Solana CLI configured signer.
        #[clap(long, parse(try_from_str=Pubkey::try_from))]
        owner: Option<Pubkey>,
    },
    /// Get all of a given owner's liquidity accounts
    GetLiquidityAccounts {
        /// The pool registry address
        #[clap(parse(try_from_str=Pubkey::try_from))]
        pool_registry: Pubkey,
        /// The owner from which to derive the liquidity account.
        /// Defaults to the -k/--keypair argument or Solana CLI configured signer.
        #[clap(long, parse(try_from_str=Pubkey::try_from))]
        owner: Option<Pubkey>,
    },
}

#[derive(Parser, Debug)]
pub struct Opt {
    #[clap(flatten)]
    rpc_url: UrlArg,
    #[clap(flatten)]
    keypair: KeypairArg,
    #[clap(flatten)]
    commitment: CommitmentArg,
    #[clap(subcommand)]
    subcommand: Subcommand,
}

impl Opt {
    pub fn process(self) -> anyhow::Result<()> {
        let app = Opt::into_app();
        let matches = app.get_matches();
        let rpc_url = self.rpc_url.resolve()?;
        let commitment = self.commitment.resolve()?;
        let client = RpcClient::new_with_commitment(rpc_url, commitment);
        let signer = self.keypair.resolve(&matches)?;
        let signer_pubkey = signer.pubkey();
        match self.subcommand {
            Subcommand::CreatePoolRegistry => {
                let ix = create_pool_registry(signer_pubkey, signer_pubkey);
                let tx = Transaction::new_signed_with_payer(
                    &[ix],
                    Some(&signer_pubkey),
                    &vec![signer],
                    client.get_latest_blockhash()?,
                );
                let signature = client.send_transaction(&tx).map_err(|e| {
                    println!("{:#?}", &e);
                    e
                })?;
                println!("{}", signature);
            }
            Subcommand::CreateSsl {
                print_only,
                pool_registry,
                json_params_path,
            } => {
                let json = &fs::read_to_string(json_params_path).map_err(|e| {
                    anyhow!("Failed to read the SSL creation params JSON file: {}", e)
                })?;
                let params: ssl_types::CreateSSLParams = serde_json::from_str(json)
                    .map_err(|e| anyhow!("Failed to deserialize SSL creation params: {}", e))?;
                println!("Creating SSL with the following parameters");
                println!("{:#?}", params);
                let ssl_types::CreateSSLParams {
                    mint,
                    asset_type,
                    oracle,
                    oracle_type,
                    initial_deposit,
                    number_of_slots_throttle,
                    max_slot_price_staleness,
                    math_params,
                } = params;
                // Check that it's a mint
                let data = client
                    .get_account_data(&mint)
                    .map_err(|e| anyhow!("Failed to fetch the specified mint: {}", e))?;
                let _ = Mint::try_deserialize(&mut data.as_slice())
                    .map_err(|e| anyhow!("Failed to deserialize the specified mint: {}", e))?;
                // Convert from Clap type to program type (necessary because of deserialization from CLI)
                let oracle_type = match oracle_type {
                    ssl_types::OracleType::Pyth => gfx_ssl_v2_interface::OracleType::Pyth,
                    ssl_types::OracleType::Switchboard => gfx_ssl_v2_interface::OracleType::Switchboardv2,
                };
                let asset_type = match asset_type {
                    ssl_types::AssetType::BlueChip => gfx_ssl_v2_interface::AssetType::BlueChip,
                    ssl_types::AssetType::Volatile => gfx_ssl_v2_interface::AssetType::Volatile,
                    ssl_types::AssetType::Stable => gfx_ssl_v2_interface::AssetType::Stable,
                };
                let pool_registry_data = get_pool_registry_blocking(&pool_registry, &client)
                    .map_err(|e| {
                        anyhow!("Failed to get pool registry at {}: {}", pool_registry, e)
                    })?;
                let ix = create_ssl(
                    initial_deposit,
                    oracle_type,
                    asset_type,
                    math_params.into(),
                    pool_registry_data.admin,
                    pool_registry,
                    mint,
                    oracle,
                    number_of_slots_throttle,
                    max_slot_price_staleness,
                );
                if print_only {
                    let message = Message::new(&[ix], None);
                    println!(
                        "{}",
                        solana_sdk::bs58::encode(message.serialize()).into_string()
                    );
                } else {
                    let tx = Transaction::new_signed_with_payer(
                        &[ix],
                        Some(&signer_pubkey),
                        &vec![signer],
                        client.get_latest_blockhash()?,
                    );
                    let signature = client.send_transaction(&tx).map_err(|e| {
                        println!("{:#?}", &e);
                        e
                    })?;
                    println!("{}", signature);
                }
            }
            Subcommand::ConfigSsl {
                print_only,
                pool_registry,
                mint,
                suspend,
                mean_window,
                std_window,
                fixed_price_distance,
                minimum_price_distance,
                max_pool_token_ratio,
                std_weight,
                latest_price_weight,
            } => {
                // Check that it's a mint
                let data = client
                    .get_account_data(&mint)
                    .map_err(|e| anyhow!("Failed to fetch the specified mint: {}", e))?;
                let _ = Mint::try_deserialize(&mut data.as_slice())
                    .map_err(|e| anyhow!("Failed to deserialize the specified mint: {}", e))?;
                let config = SSLMathConfig {
                    mean_window,
                    std_window,
                    fixed_price_distance,
                    minimum_price_distance,
                    max_pool_token_ratio,
                    std_weight,
                    latest_price_weight,
                };
                let pool_registry_data = get_pool_registry_blocking(&pool_registry, &client)
                    .map_err(|e| {
                        anyhow!("Failed to get pool registry at {}: {}", pool_registry, e)
                    })?;
                let ix = config_ssl(
                    suspend,
                    config,
                    pool_registry_data.admin,
                    pool_registry,
                    mint,
                );
                if print_only {
                    let message = Message::new(&[ix], None);
                    println!(
                        "{}",
                        solana_sdk::bs58::encode(message.serialize()).into_string()
                    );
                } else {
                    let tx = Transaction::new_signed_with_payer(
                        &[ix],
                        Some(&signer_pubkey),
                        &vec![signer],
                        client.get_latest_blockhash()?,
                    );
                    let signature = client.send_transaction(&tx).map_err(|e| {
                        println!("{:#?}", &e);
                        e
                    })?;
                    println!("{}", signature);
                }
            }
            Subcommand::ConfigSuspendAdmin {
                pool_registry,
                suspend_admin,
            } => {
                let pool_registry_data = get_pool_registry_blocking(&pool_registry, &client)
                    .map_err(|e| {
                        anyhow!("Failed to get pool registry at {}: {}", pool_registry, e)
                    })?;
                let ix =
                    config_suspend_admin(pool_registry_data.admin, pool_registry, suspend_admin);
                let tx = Transaction::new_signed_with_payer(
                    &[ix],
                    Some(&signer_pubkey),
                    &vec![signer],
                    client.get_latest_blockhash()?,
                );
                let signature = client.send_transaction(&tx).map_err(|e| {
                    println!("{:#?}", &e);
                    e
                })?;
                println!("{}", signature);
            }
            Subcommand::CreatePair {
                print_only,
                pool_registry,
                pair_params_json_path,
            } => {
                let pair_params: ssl_types::PairInitializationParams = serde_json::from_str(
                    &fs::read_to_string(pair_params_json_path)
                        .map_err(|e| anyhow!("Failed to read the pair params JSON file: {}", e))?,
                )
                .map_err(|e| anyhow!("Failed to deserialize pair params: {}", e))?;
                let pool_registry_data = get_pool_registry_blocking(&pool_registry, &client)
                    .map_err(|e| {
                        anyhow!("Failed to get pool registry at {}: {}", pool_registry, e)
                    })?;
                let ix = create_pair(
                    pair_params.0.fee_bps,
                    pair_params.1.fee_bps,
                    pool_registry_data.admin,
                    pool_registry,
                    pair_params.0.mint,
                    pair_params.1.mint,
                    pair_params.0.fee_destination,
                    pair_params.1.fee_destination,
                );
                if print_only {
                    let message = Message::new(&[ix], None);
                    println!(
                        "{}",
                        solana_sdk::bs58::encode(message.serialize()).into_string()
                    );
                } else {
                    let tx = Transaction::new_signed_with_payer(
                        &[ix],
                        Some(&signer_pubkey),
                        &vec![signer],
                        client.get_latest_blockhash()?,
                    );
                    let signature = client.send_transaction(&tx).map_err(|e| {
                        println!("{:#?}", &e);
                        e
                    })?;
                    println!("{}", signature);
                }
            }
            Subcommand::ConfigPair {
                print_only,
                pool_registry,
                pair,
                mint,
                fee_destination,
                fee_bps,
            } => {
                let pool_registry_data = get_pool_registry_blocking(&pool_registry, &client)
                    .map_err(|e| {
                        anyhow!("Failed to get pool registry at {}: {}", pool_registry, e)
                    })?;
                let pair = get_pair_blocking(&pair, &client)
                    .map_err(|e| anyhow!("Failed to fetch the specified pair: {}", e))?;
                // Check whether the specified mint is mint one, mint two, or not found.
                let mut is_mint_one = false;
                if pair.mints.0 == mint {
                    is_mint_one = true;
                } else if pair.mints.1 != mint {
                    return Err(anyhow!("Mint not found in pair"));
                }

                let (mint_one_fee_rate, mint_two_fee_rate) = if is_mint_one {
                    (fee_bps, None)
                } else {
                    (None, fee_bps)
                };
                let (mint_one_fee_dest, mint_two_fee_dest) = if is_mint_one {
                    (fee_destination, None)
                } else {
                    (None, fee_destination)
                };
                let ix = config_pair(
                    pool_registry_data.admin,
                    pool_registry,
                    pair.mints.0,
                    pair.mints.1,
                    mint_one_fee_rate,
                    mint_two_fee_rate,
                    mint_one_fee_dest,
                    mint_two_fee_dest,
                );
                if print_only {
                    let message = Message::new(&[ix], None);
                    println!(
                        "{}",
                        solana_sdk::bs58::encode(message.serialize()).into_string()
                    );
                } else {
                    let tx = Transaction::new_signed_with_payer(
                        &[ix],
                        Some(&signer_pubkey),
                        &vec![signer],
                        client.get_latest_blockhash()?,
                    );
                    let signature = client.send_transaction(&tx).map_err(|e| {
                        println!("{:#?}", &e);
                        e
                    })?;
                    println!("{}", signature);
                }
            }
            Subcommand::CrankAllPriceHistories { pool_registry } => {
                let registry_addrs = get_all_oracles_and_price_histories(&pool_registry, &client)?;
                let ix = crank_oracle_price_histories(pool_registry, &registry_addrs);
                let tx = Transaction::new_signed_with_payer(
                    &[ix],
                    Some(&signer_pubkey),
                    &vec![signer],
                    client.get_latest_blockhash()?,
                );
                let signature = client.send_transaction(&tx).map_err(|e| {
                    println!("{:#?}", &e);
                    e
                })?;
                println!("{}", signature);
            }
            Subcommand::CrankPriceHistoriesPerMint {
                pool_registry,
                mint: mints,
            } => {
                let pool_registry_addr =
                    pool_registry.unwrap_or(PoolRegistry::address(signer_pubkey));
                let pool_registry_acc = get_pool_registry_blocking(&pool_registry_addr, &client)?;

                let registry_addrs: Vec<OracleAndPriceHistory> = mints
                    .iter()
                    .map(|mint| {
                        get_oracle_and_price_history(&pool_registry_acc, *mint, &client).unwrap()
                    })
                    .collect();

                let ix = crank_oracle_price_histories(pool_registry_addr, &registry_addrs);
                let tx = Transaction::new_signed_with_payer(
                    &[ix],
                    Some(&signer_pubkey),
                    &vec![signer],
                    client.get_latest_blockhash()?,
                );
                let signature = client.send_transaction(&tx).map_err(|e| {
                    println!("{:#?}", &e);
                    e
                })?;
                println!("{}", signature);
            }
            Subcommand::ConfigPriceHistory {
                print_only,
                price_history,
                number_of_slots_throttle,
                max_slot_price_staleness,
            } => {
                let price_history = get_oracle_price_history_blocking(&price_history, &client)
                    .map_err(|_| {
                        anyhow!("unable to find oracle price history at {}", price_history)
                    })?;
                let OraclePriceHistory {
                    pool_registry,
                    oracle_address,
                    ..
                } = price_history;
                let pool_registry_data = get_pool_registry_blocking(&pool_registry, &client)
                    .map_err(|e| {
                        anyhow!("Failed to get pool registry at {}: {}", pool_registry, e)
                    })?;
                let ix = config_price_history(
                    pool_registry_data.admin,
                    pool_registry,
                    oracle_address,
                    number_of_slots_throttle,
                    max_slot_price_staleness,
                );
                if print_only {
                    let message = Message::new(&[ix], None);
                    println!(
                        "{}",
                        solana_sdk::bs58::encode(message.serialize()).into_string()
                    );
                } else {
                    let tx = Transaction::new_signed_with_payer(
                        &[ix],
                        Some(&signer_pubkey),
                        &vec![signer],
                        client.get_latest_blockhash()?,
                    );
                    let signature = client.send_transaction(&tx).map_err(|e| {
                        println!("{:#?}", &e);
                        e
                    })?;
                    println!("{}", signature);
                }
            }
            Subcommand::InternalSwap {
                pool_registry,
                mint_one,
                mint_two,
            } => {
                let pool_registry_data = get_pool_registry_blocking(&pool_registry, &client)
                    .map_err(|_| anyhow!("Failed to find pool registry"))?;
                let token_a_oracle = {
                    let ssl_pool = pool_registry_data
                        .find_pool(mint_one)
                        .map_err(|_| {
                            anyhow!(
                                "Failed to find an SSL pool for {} on pool registry {}",
                                mint_one,
                                pool_registry,
                            )
                        })
                        .unwrap();
                    let price_hist = ssl_pool.oracle_price_histories[0];
                    let price_hist_data = get_oracle_price_history_blocking(&price_hist, &client)
                        .map_err(
                            |_| anyhow!("Failed to get oracle price history for {}", mint_one,),
                        )
                        .unwrap();
                    price_hist_data.oracle_address
                };
                let token_b_oracle = {
                    let ssl_pool = pool_registry_data
                        .find_pool(mint_two)
                        .map_err(|_| {
                            anyhow!(
                                "Failed to find an SSL pool for {} on pool registry {}",
                                mint_two,
                                pool_registry,
                            )
                        })
                        .unwrap();
                    let price_hist = ssl_pool.oracle_price_histories[0];
                    let price_hist_data = get_oracle_price_history_blocking(&price_hist, &client)
                        .map_err(
                            |_| anyhow!("Failed to get oracle price history for {}", mint_two,),
                        )
                        .unwrap();
                    price_hist_data.oracle_address
                };
                let ix = internal_swap(
                    pool_registry,
                    mint_one,
                    mint_two,
                    token_a_oracle,
                    token_b_oracle,
                );
                let tx = Transaction::new_signed_with_payer(
                    &[ix],
                    Some(&signer_pubkey),
                    &vec![signer],
                    client.get_latest_blockhash()?,
                );
                let signature = client.send_transaction(&tx).map_err(|e| {
                    println!("{:#?}", &e);
                    e
                })?;
                println!("{}", signature);
            }
            Subcommand::ClaimFees {
                pool_registry,
                mint,
            } => {
                let ix = claim_fees(pool_registry, signer_pubkey, mint);
                let tx = Transaction::new_signed_with_payer(
                    &[ix],
                    Some(&signer_pubkey),
                    &vec![signer],
                    client.get_latest_blockhash()?,
                );
                let signature = client.send_transaction(&tx).map_err(|e| {
                    println!("{:#?}", &e);
                    e
                })?;
                println!("{}", signature);
            }
            Subcommand::CreateLiquidityAccount {
                pool_registry,
                mint,
            } => {
                let ix = create_liquidity_account(pool_registry, signer_pubkey, mint);
                let tx = Transaction::new_signed_with_payer(
                    &[ix],
                    Some(&signer_pubkey),
                    &vec![signer],
                    client.get_latest_blockhash()?,
                );
                let signature = client.send_transaction(&tx).map_err(|e| {
                    println!("{:#?}", &e);
                    e
                })?;
                println!("{}", signature);
            }
            Subcommand::CloseLiquidityAccount {
                pool_registry,
                mint,
                rent_recipient,
            } => {
                let liquidity_account =
                    LiquidityAccount::address(pool_registry, mint, signer_pubkey);
                let ix = close_liquidity_account(
                    liquidity_account,
                    signer_pubkey,
                    rent_recipient.unwrap_or(signer_pubkey),
                );
                let tx = Transaction::new_signed_with_payer(
                    &[ix],
                    Some(&signer_pubkey),
                    &vec![signer],
                    client.get_latest_blockhash()?,
                );
                let signature = client.send_transaction(&tx).map_err(|e| {
                    println!("{:#?}", &e);
                    e
                })?;
                println!("{}", signature);
            }
            Subcommand::Deposit {
                pool_registry,
                mint,
                amount,
            } => {
                let ix = deposit(pool_registry, signer_pubkey, mint, amount);
                let tx = Transaction::new_signed_with_payer(
                    &[ix],
                    Some(&signer_pubkey),
                    &vec![signer],
                    client.get_latest_blockhash()?,
                );
                let signature = client.send_transaction(&tx).map_err(|e| {
                    println!("{:#?}", &e);
                    e
                })?;
                println!("{}", signature);
            }
            Subcommand::Withdraw {
                pool_registry,
                mint,
                amount,
            } => {
                let ix = withdraw(pool_registry, signer_pubkey, mint, amount);
                let tx = Transaction::new_signed_with_payer(
                    &[ix],
                    Some(&signer_pubkey),
                    &vec![signer],
                    client.get_latest_blockhash()?,
                );
                let signature = client.send_transaction(&tx).map_err(|e| {
                    println!("{:#?}", &e);
                    e
                })?;
                println!("{}", signature);
            }
            Subcommand::Swap {
                amount_in,
                min_out,
                mint_in,
                mint_out,
                pool_registry,
            } => {
                let pool_registry_data = get_pool_registry_blocking(&pool_registry, &client)
                    .map_err(|_| {
                        anyhow!("Could not find pool registry at address: {}", pool_registry)
                    })?;
                let pair_address = Pair::address(pool_registry, mint_in, mint_out);
                let pair = get_pair_blocking(&pair_address, &client).map_err(|_| {
                    anyhow!(
                        "Could not find pair for mints: {} and {} in pool registry {}",
                        mint_in,
                        mint_out,
                        pool_registry,
                    )
                })?;
                let ssl_in = pool_registry_data.find_pool(mint_in).map_err(|_| {
                    anyhow!(
                        "Could not find the input mint {} in pool registry {}",
                        mint_in,
                        pool_registry,
                    )
                })?;
                let price_hist_in =
                    get_oracle_price_history_blocking(&ssl_in.oracle_price_histories[0], &client)
                        .map_err(|_| {
                        anyhow!(
                            "Could not find the oracle price history for ssl pool of mint {}",
                            mint_in,
                        )
                    })?;
                let ssl_out = pool_registry_data.find_pool(mint_out).map_err(|_| {
                    anyhow!(
                        "Could not find the output mint {} in pool registry {}",
                        mint_out,
                        pool_registry,
                    )
                })?;
                let price_hist_out =
                    get_oracle_price_history_blocking(&ssl_out.oracle_price_histories[0], &client)
                        .map_err(|_| {
                            anyhow!(
                                "Could not find the oracle price history for ssl pool of mint {}",
                                mint_out,
                            )
                        })?;
                let (_, fee_destination, _) = pair
                    .find_fee_attrs(mint_in, mint_out)
                    .map_err(|_| anyhow!("Could not resolve fee destination from pair"))?;
                let compute_budget_ix = Instruction::new_with_borsh(
                    pubkey!("ComputeBudget111111111111111111111111111111"),
                    &ComputeBudgetInstruction::RequestUnitsDeprecated {
                        units: 1_000_000,
                        additional_fee: 0,
                    },
                    vec![],
                );
                let ix = swap(
                    amount_in,
                    min_out,
                    pool_registry,
                    signer_pubkey,
                    mint_in,
                    mint_out,
                    price_hist_in.oracle_address,
                    price_hist_out.oracle_address,
                    fee_destination,
                );
                let tx = Transaction::new_signed_with_payer(
                    &[compute_budget_ix, ix],
                    Some(&signer_pubkey),
                    &vec![signer],
                    client.get_latest_blockhash()?,
                );
                let signature = client.send_transaction(&tx).map_err(|e| {
                    println!("{:#?}", &e);
                    e
                })?;
                println!("{}", signature);
            }
            Subcommand::GetPoolRegistryAddress { seed } => {
                let pool_registry = PoolRegistry::address(seed.unwrap_or(signer_pubkey));
                println!("{}", pool_registry);
            }
            Subcommand::GetSSLAddresses {
                pool_registry,
                mint,
                oracle,
            } => {
                let signer_address = SSLPool::signer_address(pool_registry, mint);
                println!("Pool Registry: {}", pool_registry);
                println!("Mint: {}", mint);
                println!("Signer: {}", signer_address);
                println!(
                    "Pool Vault: {}",
                    SSLPool::vault_address(pool_registry, mint)
                );
                println!(
                    "Fee Vault: {}",
                    get_associated_token_address(&pool_registry, &mint)
                );
                if let Some(oracle) = oracle {
                    println!(
                        "Oracle Price History: {}",
                        OraclePriceHistory::address(&pool_registry, &oracle)
                    );
                }
            }
            Subcommand::GetOraclePriceHistoryAddress {
                pool_registry,
                oracle,
            } => {
                let price_history_address = OraclePriceHistory::address(&pool_registry, &oracle);
                println!("{}", price_history_address);
            }
            Subcommand::GetPairAddress {
                pool_registry,
                mint_one,
                mint_two,
            } => {
                let pair = Pair::address(pool_registry, mint_one, mint_two);
                println!("{}", pair);
            }
            Subcommand::GetSSLPoolVaultAddress {
                pool_registry,
                mint,
            } => {
                let ssl_pool_vault = SSLPool::vault_address(pool_registry, mint);
                println!("{}", ssl_pool_vault);
            }
            Subcommand::GetLiquidityAccountAddress {
                pool_registry,
                mint,
                owner,
            } => {
                let owner = owner.unwrap_or(signer_pubkey);
                let liquidity_account = LiquidityAccount::address(pool_registry, mint, owner);
                println!("{}", liquidity_account);
            }
            Subcommand::GetPoolRegistry { address } => {
                let pool_registry = get_pool_registry_blocking(&address, &client)?;
                pool_registry
                    .entries
                    .into_iter()
                    .filter(|pool| *pool != SSLPool::default())
                    .for_each(|pool| {
                        println!("{}", pool);
                        let main_vault = SSLPool::vault_address(address, pool.mint);
                        let main_vault_balance = client
                            .get_token_account_balance(&main_vault)
                            .map_err(|_| anyhow!("Unable to fetch main vault token balance"))
                            .unwrap();
                        println!(
                            "Main vault ({}) balance: {}",
                            main_vault, main_vault_balance.ui_amount_string
                        );
                        let other_pools: Vec<SSLPool> = pool_registry
                            .entries
                            .into_iter()
                            .filter(|other_pool| *other_pool != SSLPool::default())
                            .filter(|other_pool| other_pool.mint != pool.mint)
                            .collect();
                        let secondary_vaults: Vec<Pubkey> = other_pools
                            .iter()
                            .map(|other_pool| {
                                SSLPool::secondary_token_vault_address(
                                    address,
                                    pool.mint,
                                    other_pool.mint,
                                )
                            })
                            .collect();
                        let accounts = client.get_multiple_accounts(&secondary_vaults).unwrap();
                        accounts
                            .into_iter()
                            .zip(other_pools)
                            .filter(|(act, _)| act.is_some())
                            .for_each(|(act, pool)| {
                                let act = act.unwrap();
                                let token_act =
                                    TokenAccount::try_deserialize(&mut &act.data[..]).unwrap();
                                let ui_amount = token_amount::to_ui(
                                    token_act.amount,
                                    pool.mint_decimals as u32,
                                );
                                println!(
                                    "secondary vault for mint {} balance: {}",
                                    pool.mint, ui_amount,
                                );
                            });
                        println!("---");
                    })
            }
            Subcommand::GetSSLPool {
                pool_registry,
                mint,
            } => {
                let pool_registry_data = get_pool_registry_blocking(&pool_registry, &client)?;
                let pool = pool_registry_data.find_pool(mint).map_err(|_| {
                    anyhow!(
                        "Failed to find pool for mint {} on pool registry {}",
                        mint,
                        pool_registry_data,
                    )
                })?;
                println!("{}", pool);
                let main_vault = SSLPool::vault_address(pool_registry, pool.mint);
                let main_vault_balance = client
                    .get_token_account_balance(&main_vault)
                    .map_err(|_| anyhow!("Unable to fetch main vault token balance"))?;
                println!(
                    "Main vault ({}) balance: {}",
                    main_vault, main_vault_balance.ui_amount_string
                );
                let other_pools: Vec<SSLPool> = pool_registry_data
                    .entries
                    .into_iter()
                    .filter(|pool| *pool != SSLPool::default())
                    .filter(|pool| pool.mint != mint)
                    .collect();
                let secondary_vaults: Vec<Pubkey> = other_pools
                    .iter()
                    .map(|pool| {
                        SSLPool::secondary_token_vault_address(pool_registry, mint, pool.mint)
                    })
                    .collect();
                let accounts = client.get_multiple_accounts(&secondary_vaults)?;
                accounts
                    .into_iter()
                    .zip(other_pools)
                    .filter(|(act, _)| act.is_some())
                    .for_each(|(act, pool)| {
                        let act = act.unwrap();
                        let token_act = TokenAccount::try_deserialize(&mut &act.data[..]).unwrap();
                        let ui_amount =
                            token_amount::to_ui(token_act.amount, pool.mint_decimals as u32);
                        println!(
                            "secondary vault for mint {} balance: {}",
                            pool.mint, ui_amount,
                        );
                    })
            }
            Subcommand::GetPair {
                pool_registry,
                mint_one,
                mint_two,
            } => {
                let pair =
                    get_pair_blocking(&Pair::address(pool_registry, mint_one, mint_two), &client)?;
                print!("{}", pair);
            }
            Subcommand::GetOraclePriceHistory { address } => {
                let price_history = get_oracle_price_history_blocking(&address, &client)?;
                print!("{}", price_history);
            }
            Subcommand::GetLiquidityAccount {
                pool_registry,
                mint,
                owner,
            } => {
                let liquidity_account_addr =
                    LiquidityAccount::address(pool_registry, mint, owner.unwrap_or(signer_pubkey));
                let liquidity_account =
                    get_liquidity_account_blocking(&liquidity_account_addr, &client)?;

                print!("{}", liquidity_account);
            }
            Subcommand::GetLiquidityAccounts {
                pool_registry,
                owner,
            } => {
                let pool_registry_data = get_pool_registry_blocking(&pool_registry, &client)?;
                for i in 0..pool_registry_data.num_entries {
                    let pool = &pool_registry_data.entries[i as usize];
                    let liquidity_account_addr = LiquidityAccount::address(
                        pool_registry,
                        pool.mint,
                        owner.unwrap_or(signer_pubkey),
                    );
                    let liquidity_account =
                        get_liquidity_account_blocking(&liquidity_account_addr, &client);
                    match liquidity_account {
                        Ok(data) => println!("{}", data),
                        Err(_) => {
                            println!("No liquidity account for mint {}", pool.mint)
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
