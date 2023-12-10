use crate::pool_registry::PoolRegistry;
use anchor_lang::prelude::*;

/// Updates the global data of a [PoolRegistry] account.
#[derive(Accounts)]
pub struct ConfigPoolRegistry<'info> {
    /// The pool registry admin.
    pub admin: Signer<'info>,

    /// The global registry of SSL pools.
    #[account(mut, has_one = admin)]
    pub pool_registry: AccountLoader<'info, PoolRegistry>,
}
