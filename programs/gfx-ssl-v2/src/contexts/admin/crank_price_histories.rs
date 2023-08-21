use crate::PoolRegistry;
use anchor_lang::prelude::*;

/// Index a new historical price into a single price history.
#[derive(Accounts)]
pub struct CrankPriceHistories<'info> {
    /// The global registry of SSL pools.
    pub pool_registry: AccountLoader<'info, PoolRegistry>,
}
