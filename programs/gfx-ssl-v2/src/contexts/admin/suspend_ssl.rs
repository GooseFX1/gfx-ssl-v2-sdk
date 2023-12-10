use crate::{errors::SSLV2Error, pool_registry::PoolRegistry};
use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

/// Suspend the SSL pool or update it with new price calculation parameters.
/// Only executable by suspend_admin. The admin can suspend the pool
/// using the `config_ssl` instruction.
#[derive(Accounts)]
pub struct SuspendSsl<'info> {
    /// The global registry of SSL pools. This instruction modifies an entry on this.
    #[account(mut, has_one = suspend_admin @ SSLV2Error::NotAdmin)]
    pub pool_registry: AccountLoader<'info, PoolRegistry>,

    /// The mint associated with the [SSLPool] being configured.
    pub mint: Account<'info, Mint>,

    /// The suspend admin. This makes the instruction permissioned.
    #[account(mut)]
    pub suspend_admin: Signer<'info>,
}
