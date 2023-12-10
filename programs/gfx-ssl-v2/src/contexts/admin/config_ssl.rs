use crate::{errors::SSLV2Error, pool_registry::PoolRegistry};
use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

/// Suspend the SSL pool or update it with new price calculation parameters.
///
/// Note: Fee parameters are contained elsewhere, on the `Pair` account.
#[derive(Accounts)]
pub struct ConfigSsl<'info> {
    /// The global registry of SSL pools. This instruction modifies an entry on this.
    #[account(mut, has_one = admin @ SSLV2Error::NotAdmin)]
    pub pool_registry: AccountLoader<'info, PoolRegistry>,

    /// The mint associated with the [SSLPool] being configured.
    pub mint: Account<'info, Mint>,

    /// The pool registry admin. This makes the instruction permissioned.
    #[account(mut)]
    pub admin: Signer<'info>,
}
