use crate::pool_registry::PoolRegistry;
use anchor_lang::prelude::*;
use anchor_spl::{
  associated_token::AssociatedToken,
  token::{Mint, Token, TokenAccount},
};

#[derive(Accounts)]
pub struct ClaimJito<'info> {
    #[account(mut)]
    pub admin: Signer<'info>,

    #[account(
        mut,
        has_one = admin
    )]
    pub pool_registry: AccountLoader<'info, PoolRegistry>,
    /// The [MerkleDistributor].
    /// CHECK: Don't check
    #[account(mut)]
    pub distributor: AccountInfo<'info>,

    /// Claim status PDA
    /// CHECK: Don't check
    #[account(mut)]
    pub claim_status: UncheckedAccount<'info>,

    /// Distributor ATA containing the tokens to distribute.
    #[account(
        mut
    )]
    pub from: Box<Account<'info, TokenAccount>>,

    /// Account to send the claimed tokens to.
    #[account(
        mut)]
    pub to: Box<Account<'info, TokenAccount>>,

    /// Who is claiming the tokens.
    /// CHECK: Don't check
    pub ssl_pool_in_signer: UncheckedAccount<'info>,

    #[account(
        mut)]
    pub final_destination: Box<Account<'info, TokenAccount>>,

    /// JITO program id
    /// CHECK: Don't check
    pub jito_program: UncheckedAccount<'info>,

    /// SPL [Token] program.
    pub token_program: Program<'info, Token>,

    /// The [System] program.
    pub system_program: Program<'info, System>,
}