use anchor_lang::{prelude::AccountMeta, solana_program::pubkey::Pubkey, ToAccountMetas};
use anchor_spl::{associated_token::get_associated_token_address, token};
pub use gfx_ssl_v2_interface::state::*;
use solana_client::{nonblocking::rpc_client::RpcClient, rpc_client};

use crate::{
    error::{GfxSslSdkError, Result},
    utils::{get_state, get_state_blocking},
};

/// A pair of accounts that is passed in to a price history crank instruction.
/// The crank takes N such pairs, as many as the pool registry has, up to
/// whatever headroom is offered by the Solana runtime.
#[derive(Clone, Copy, Debug)]
pub struct OracleAndPriceHistory {
    pub oracle: Pubkey,
    pub price_history: Pubkey,
}

/// Acquire the addresses for all oracles and price histories
/// listed in a pool registry. These are fed as `remaining_accounts`
/// in price history crank instructions.
pub fn get_all_oracles_and_price_histories(
    pool_registry_addr: &Pubkey,
    client: &rpc_client::RpcClient,
) -> Result<Vec<OracleAndPriceHistory>> {
    let pool_registry = get_pool_registry_blocking(pool_registry_addr, client)?;

    let mut out = vec![];
    for idx in 0usize..pool_registry.num_entries as usize {
        let pool = &pool_registry.entries[idx];
        let price_history_addr = pool.oracle_price_histories[0];
        let oph_data = get_oracle_price_history_blocking(&price_history_addr, client)?;
        out.push(OracleAndPriceHistory {
            price_history: price_history_addr,
            oracle: oph_data.oracle_address,
        });
    }

    Ok(out)
}

/// Acquire an individual oracle and price history pairing, rather than all of them.
pub fn get_oracle_and_price_history(
    pool_registry: &PoolRegistry,
    mint: Pubkey,
    client: &rpc_client::RpcClient,
) -> Result<OracleAndPriceHistory> {
    let pool = pool_registry
        .find_pool(mint)
        .map_err(|_| GfxSslSdkError::PoolNotFound(mint))?;
    let oph_data = get_oracle_price_history_blocking(&pool.oracle_price_histories[0], client)?;
    Ok(OracleAndPriceHistory {
        price_history: pool.oracle_price_histories[0],
        oracle: oph_data.oracle_address,
    })
}

pub async fn get_oracle_price_history(
    address: &Pubkey,
    client: &RpcClient,
) -> Result<OraclePriceHistory> {
    get_state(address, client, "OraclePriceHistory").await
}

pub fn get_oracle_price_history_blocking(
    address: &Pubkey,
    client: &rpc_client::RpcClient,
) -> Result<OraclePriceHistory> {
    get_state_blocking(address, client, "OraclePriceHistory")
}

pub async fn get_pool_registry(address: &Pubkey, client: &RpcClient) -> Result<PoolRegistry> {
    get_state(address, client, "PoolRegistry").await
}

pub fn get_pool_registry_blocking(
    address: &Pubkey,
    client: &rpc_client::RpcClient,
) -> Result<PoolRegistry> {
    get_state_blocking(address, client, "PoolRegistry")
}

pub async fn get_pair(address: &Pubkey, client: &RpcClient) -> Result<Pair> {
    get_state(address, client, "Pair").await
}

pub fn get_pair_blocking(address: &Pubkey, client: &rpc_client::RpcClient) -> Result<Pair> {
    get_state_blocking(address, client, "Pair")
}

pub async fn get_liquidity_account(
    address: &Pubkey,
    client: &RpcClient,
) -> Result<LiquidityAccount> {
    get_state(address, client, "LiquidityAccount").await
}

pub fn get_liquidity_account_blocking(
    address: &Pubkey,
    client: &rpc_client::RpcClient,
) -> Result<LiquidityAccount> {
    get_state_blocking(address, client, "LiquidityAccount")
}

#[allow(clippy::too_many_arguments)]
pub fn get_account_metas_for_swap(
    pool_registry: Pubkey,
    user_wallet: Pubkey,
    mint_in: Pubkey,
    mint_out: Pubkey,
    input_token_oracle: Pubkey,
    output_token_oracle: Pubkey,
    fee_destination: Pubkey,
) -> Vec<AccountMeta> {
    let pair = Pair::address(pool_registry, mint_in, mint_out);
    let ssl_out_fee_vault = get_associated_token_address(&pool_registry, &mint_out);
    let user_ata_in = get_associated_token_address(&user_wallet, &mint_in);
    let user_ata_out = get_associated_token_address(&user_wallet, &mint_out);
    let input_token_price_history =
        OraclePriceHistory::address(&pool_registry, &input_token_oracle);
    let output_token_price_history =
        OraclePriceHistory::address(&pool_registry, &output_token_oracle);
    let ssl_pool_in_signer = SSLPool::signer_address(pool_registry, mint_in);
    let ssl_pool_out_signer = SSLPool::signer_address(pool_registry, mint_out);
    let ssl_in_main_vault = get_associated_token_address(&ssl_pool_in_signer, &mint_in);
    let ssl_in_secondary_vault = get_associated_token_address(&ssl_pool_in_signer, &mint_out);
    let ssl_out_main_vault = get_associated_token_address(&ssl_pool_out_signer, &mint_out);
    let ssl_out_secondary_vault = get_associated_token_address(&ssl_pool_out_signer, &mint_in);

    gfx_ssl_v2_interface::accounts::Swap {
        pair,
        pool_registry,
        user_wallet,
        ssl_pool_in_signer,
        ssl_pool_out_signer,
        user_ata_in,
        user_ata_out,
        ssl_out_main_vault,
        ssl_out_secondary_vault,
        ssl_in_main_vault,
        ssl_in_secondary_vault,
        ssl_out_fee_vault,
        fee_destination,
        output_token_price_history,
        output_token_oracle,
        input_token_price_history,
        input_token_oracle,
        event_emitter: EventEmitter::address(),
        token_program: token::ID,
    }
    .to_account_metas(None)
}
