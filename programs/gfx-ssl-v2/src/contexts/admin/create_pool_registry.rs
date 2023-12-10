use crate::{pool_registry::PoolRegistry, PDAIdentifier};
use anchor_lang::prelude::*;

/// Creates a [PoolRegistry] account tied to a given [Controller].
#[derive(Accounts)]
pub struct CreatePoolRegistry<'info> {
    /// Lamports for rent funded from here.
    #[account(mut)]
    pub funder: Signer<'info>,

    /// The pool registry admin.
    pub admin: Signer<'info>,

    /// The global registry of SSL pools.
    #[account(
        init,
        payer = funder,
        space = 8 + std::mem::size_of::<PoolRegistry>(),
        seeds = [PoolRegistry::IDENT, admin.key().as_ref()],
        bump,
    )]
    pub pool_registry: AccountLoader<'info, PoolRegistry>,

    /// Needed to create a new account
    pub system_program: Program<'info, System>,
}
