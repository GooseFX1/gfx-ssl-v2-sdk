use crate::utils::PDAIdentifier;
#[cfg(feature = "debug-msg")]
use crate::SSLPoolStatus;
use crate::{EventEmitter, LiquidityAccount, PoolRegistry, SSLPool};
use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

/// User deposit liquidity into an SSL pool.
#[derive(Accounts)]
pub struct Deposit<'info> {
    #[account(
        mut,
        has_one = owner,
        has_one = pool_registry,
    )]
    pub liquidity_account: Account<'info, LiquidityAccount>,

    pub owner: Signer<'info>,

    /// Origin of the user deposit.
    #[account(
        mut,
        associated_token::mint = liquidity_account.mint,
        associated_token::authority = owner,
    )]
    pub user_ata: Box<Account<'info, TokenAccount>>,

    /// CHECK: Uninitialized PDA signer, owner of pool vault
    #[account(
        seeds = [
            SSLPool::IDENT,
            pool_registry.key().as_ref(),
            liquidity_account.mint.as_ref(),
        ],
        bump,
    )]
    pub ssl_pool_signer: UncheckedAccount<'info>,

    /// The token vault where all user deposits on this SSL are stored.
    #[account(
        mut,
        associated_token::mint = liquidity_account.mint,
        associated_token::authority = ssl_pool_signer,
    )]
    pub pool_vault: Box<Account<'info, TokenAccount>>,

    /// Fee-claim routine is executed during this instruction.
    #[account(
        mut,
        associated_token::mint = liquidity_account.mint,
        associated_token::authority = pool_registry,
    )]
    pub ssl_fee_vault: Box<Account<'info, TokenAccount>>,

    #[account(mut)]
    pub pool_registry: AccountLoader<'info, PoolRegistry>,

    #[account(mut)]
    pub event_emitter: Account<'info, EventEmitter>,

    pub token_program: Program<'info, Token>,
}
