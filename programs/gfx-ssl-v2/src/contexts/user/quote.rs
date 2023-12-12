use crate::{OraclePriceHistory, Pair, PoolRegistry};
use anchor_lang::prelude::*;
use anchor_spl::token::TokenAccount;

#[derive(Accounts)]
pub struct Quote<'info> {
    /// Stores state regarding fees
    #[account(mut, has_one = pool_registry)]
    pub pair: Box<Account<'info, Pair>>,

    #[account(mut)]
    pub pool_registry: AccountLoader<'info, PoolRegistry>,

    pub ssl_out_main_vault: Box<Account<'info, TokenAccount>>,
    pub ssl_out_secondary_vault: Box<Account<'info, TokenAccount>>,
    pub ssl_in_main_vault: Box<Account<'info, TokenAccount>>,
    pub ssl_in_secondary_vault: Box<Account<'info, TokenAccount>>,

    /// Oracle price history of `mint_out`.
    #[account(mut, has_one = pool_registry)]
    pub output_token_price_history: AccountLoader<'info, OraclePriceHistory>,

    /// CHECK: This account must match the pubkey of what is stored on `self.main_token_price_history`.
    pub output_token_oracle: UncheckedAccount<'info>,

    /// Oracle price history of input mint.
    #[account(mut, has_one = pool_registry)]
    pub input_token_price_history: AccountLoader<'info, OraclePriceHistory>,

    /// CHECK: This account must match the pubkey of what is stored on `self.non_main_token_price_history`.
    pub input_token_oracle: UncheckedAccount<'info>,
}
