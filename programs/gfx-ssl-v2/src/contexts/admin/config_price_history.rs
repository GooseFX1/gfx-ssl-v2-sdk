use crate::{OraclePriceHistory, PoolRegistry, SSLV2Error};
use anchor_lang::prelude::*;

/// Modifies the number of slots to wait before updating the oracle price history.
#[derive(Accounts)]
pub struct ConfigPriceHistory<'info> {
    /// The admin. This makes the instruction permissioned.
    pub admin: Signer<'info>,

    /// The global registry of SSL pools
    #[account(has_one = admin @ SSLV2Error::NotAdmin)]
    pub pool_registry: AccountLoader<'info, PoolRegistry>,

    /// Stores the oracle price history.
    #[account(mut, has_one = pool_registry @ SSLV2Error::PoolRegistryNotMatchOraclePriceHistory)]
    pub oracle_price_history: AccountLoader<'info, OraclePriceHistory>,
}
