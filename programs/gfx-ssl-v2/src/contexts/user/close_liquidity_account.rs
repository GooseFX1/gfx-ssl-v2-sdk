use crate::{LiquidityAccount, EventEmitter};
use anchor_lang::prelude::*;

#[derive(Accounts)]
pub struct CloseLiquidityAccount<'info> {
    /// Signer, user metadata account owner
    #[account(mut)]
    pub owner: Signer<'info>,

    /// Recipient of the lamports that supplied the rent-exempt balance
    /// of the liquidity account account being closed.
    #[account(mut)]
    pub rent_recipient: SystemAccount<'info>,

    /// Close this account
    #[account(
        mut,
        close=rent_recipient,
        has_one = owner,
    )]
    pub liquidity_account: Account<'info, LiquidityAccount>,
    
    #[account(mut)]
    pub event_emitter: Account<'info, EventEmitter>,

    pub system_program: Program<'info, System>,
}
