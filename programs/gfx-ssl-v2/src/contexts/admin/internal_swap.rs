use crate::{EventEmitter, OraclePriceHistory, PDAIdentifier, Pair, PoolRegistry, SSLPool, SSLV2Error};
use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

/// Fair-value swap between two SSL pools to rebalance their respective
/// liquidity and hold a greater amount of the main token.
///
/// "Token A" / "SSL A" refer to the mint at `self.pair.mints.0`, and
/// "Token B" / "SSL B" refer to the mint at `self.pair.mints.1`.
#[derive(Accounts)]
pub struct InternalSwap<'info> {
    pub pool_registry: AccountLoader<'info, PoolRegistry>,

    #[account(mut, has_one = pool_registry @ SSLV2Error::NotAdmin)]
    pub pair: Box<Account<'info, Pair>>,

    /// Main Token account of SSL Pool A, the mint at `self.pair.mints.0`.
    #[account(
        mut,
        associated_token::mint = pair.mints.0,
        associated_token::authority = ssl_pool_a_signer,
    )]
    pub ssl_a_main_token: Account<'info, TokenAccount>,

    /// Main Token account of SSL Pool B, the mint at `self.pair.mints.1`.
    #[account(
        mut,
        associated_token::mint = pair.mints.1,
        associated_token::authority = ssl_pool_b_signer,
    )]
    pub ssl_b_main_token: Account<'info, TokenAccount>,

    /// Token B account of SSL Pool A
    #[account(
        mut,
        associated_token::mint = pair.mints.1,
        associated_token::authority = ssl_pool_a_signer,
    )]
    pub ssl_a_secondary_token: Account<'info, TokenAccount>,

    /// Token A account of SSL Pool B
    #[account(
        mut,
        associated_token::mint = pair.mints.0,
        associated_token::authority = ssl_pool_b_signer,
    )]
    pub ssl_b_secondary_token: Account<'info, TokenAccount>,

    /// Oracle price history of `token_a`.
    #[account(mut, has_one = pool_registry)]
    pub token_a_price_history: AccountLoader<'info, OraclePriceHistory>,

    /// CHECK: This account must match the pubkey of what is stored on `self.token_a_price_history`.
    pub token_a_oracle: UncheckedAccount<'info>,

    /// Oracle price history of `token_b`.
    #[account(mut, has_one = pool_registry)]
    pub token_b_price_history: AccountLoader<'info, OraclePriceHistory>,

    /// CHECK: This account must match the pubkey of what is stored on `self.token_b_price_history`.
    pub token_b_oracle: UncheckedAccount<'info>,

    /// CHECK: Uninitialized PDA signer, signs for transfer of `mint_out` from SSL Pool "in".
    #[account(
        seeds = [
            SSLPool::IDENT,
            pair.pool_registry.key().as_ref(),
            ssl_a_main_token.mint.key().as_ref(),
        ],
        bump,
    )]
    pub ssl_pool_a_signer: UncheckedAccount<'info>,

    /// CHECK: Uninitialized PDA signer, signs for transfer of `mint_out` from SSL Pool "out".
    #[account(
        seeds = [
            SSLPool::IDENT,
            pair.pool_registry.key().as_ref(),
            ssl_b_main_token.mint.key().as_ref(),
        ],
        bump,
    )]
    pub ssl_pool_b_signer: UncheckedAccount<'info>,

    #[account(mut)]
    pub event_emitter: Box<Account<'info, EventEmitter>>,

    pub token_program: Program<'info, Token>,
}
