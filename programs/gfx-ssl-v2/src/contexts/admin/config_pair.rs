use crate::{Pair, PoolRegistry, SSLV2Error};
use anchor_lang::prelude::*;

/// Suspend the SSL pool or update it with new price calculation parameters.
///
/// Note: Fee parameters are contained elsewhere, on the `Pair` account.
#[derive(Accounts)]
pub struct ConfigPair<'info> {
    /// Fees collected of `pair.mints.0` will flow here.
    /// CHECK: If this `== Pubkey::default` it is ignored.
    pub mint_one_fee_destination: UncheckedAccount<'info>,

    /// Fees collected of `pair.mints.1` will flow here.
    /// CHECK: If this `== Pubkey::default` it is ignored.
    pub mint_two_fee_destination: UncheckedAccount<'info>,

    /// The pair to be configured.
    #[account(mut, has_one = pool_registry @ SSLV2Error::NotAdmin)]
    pub pair: Account<'info, Pair>,

    /// The global registry of SSL pools. This instruction adds an entry on this.
    #[account(
        mut,
        has_one = admin @ SSLV2Error::NotAdmin
    )]
    pub pool_registry: AccountLoader<'info, PoolRegistry>,

    /// The pair admin. This makes the instruction permissioned.
    pub admin: Signer<'info>,
}
