use crate::{errors::SSLV2Error, pool_registry::PoolRegistry};
use anchor_lang::prelude::*;

/// Update the suspend admin field of a pool registry.
#[derive(Accounts)]
pub struct ConfigSuspendAdmin<'info> {
    /// The global registry of SSL pools. This instruction modifies an entry on this.
    #[account(mut, has_one = admin @ SSLV2Error::NotAdmin)]
    pub pool_registry: AccountLoader<'info, PoolRegistry>,

    /// The admin. This makes the instruction permissioned.
    pub admin: Signer<'info>,

    /// CHECK: The new admin to set.
    pub suspend_admin: UncheckedAccount<'info>,
}
