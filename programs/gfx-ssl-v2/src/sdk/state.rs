pub use crate::state::*;
use anchor_lang::solana_program::pubkey::Pubkey;
use solana_client::{nonblocking::rpc_client::RpcClient, rpc_client};

use crate::sdk::{
    error::Result,
    utils::{get_state, get_state_blocking},
};

/// Acquire the addresses for all oracles and price histories
/// listed in a pool registry. These are fed as `remaining_accounts`
/// in price history crank instructions.
pub fn get_all_oracles_and_price_histories(
    pool_registry_addr: &Pubkey,
    client: &rpc_client::RpcClient,
) -> Result<Vec<(Pubkey, OraclePriceHistory)>> {
    let pool_registry = get_pool_registry_blocking(pool_registry_addr, client)?;

    let mut out = vec![];
    for idx in 0usize..pool_registry.num_entries as usize {
        let pool = &pool_registry.entries[idx];
        for price_history_addr in &pool.oracle_price_histories {
            if *price_history_addr != Pubkey::default() {
                let oph_data = get_oracle_price_history_blocking(&price_history_addr, client)?;
                out.push((*price_history_addr, oph_data));
            }
        }
    }

    Ok(out)
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
