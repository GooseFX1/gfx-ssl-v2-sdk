use crate::{EventEmitter, LiquidityAccount, PoolRegistry};
use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

/// Claim a share of the fees in form of USDC.
#[derive(Accounts)]
pub struct ClaimFees<'info> {
    /// The global registry of SSL pools.
    pub pool_registry: AccountLoader<'info, PoolRegistry>,

    /// Signer, user metadata account owner
    #[account(mut)]
    pub owner: Signer<'info>,

    #[account(
        mut,
        associated_token::mint = liquidity_account.mint,
        associated_token::authority = pool_registry,
    )]
    pub ssl_fee_vault: Box<Account<'info, TokenAccount>>,

    /// Destination account for the claimed USDC fees.
    #[account(
        mut,
        associated_token::mint = liquidity_account.mint,
        associated_token::authority = owner,
    )]
    pub owner_ata: Box<Account<'info, TokenAccount>>,

    #[account(
        mut,
        has_one = owner,
        has_one = pool_registry,
    )]
    pub liquidity_account: Box<Account<'info, LiquidityAccount>>,

    #[account(mut)]
    pub event_emitter: Account<'info, EventEmitter>,

    pub token_program: Program<'info, Token>,
}
