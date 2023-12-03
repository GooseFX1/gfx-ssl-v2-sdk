use crate::utils::PDAIdentifier;
use crate::{LiquidityAccount, PoolRegistry, EventEmitter};
use anchor_lang::prelude::*;
use anchor_spl::token::Mint;

#[derive(Accounts)]
pub struct CreateLiquidityAccount<'info> {
    /// The global registry of SSL pools.
    pub pool_registry: AccountLoader<'info, PoolRegistry>,

    pub mint: Account<'info, Mint>,

    #[account(
        init,
        payer = owner,
        space = 8 + std::mem::size_of::<LiquidityAccount>(),
        seeds = [
            LiquidityAccount::IDENT,
            pool_registry.key().as_ref(),
            mint.key().as_ref(),
            owner.key().as_ref(),
        ],
        bump,
    )]
    pub liquidity_account: Account<'info, LiquidityAccount>,

    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(mut)]
    pub event_emitter: Account<'info, EventEmitter>,

    pub system_program: Program<'info, System>,
}
