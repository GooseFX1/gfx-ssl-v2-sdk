use crate::{EventEmitter, OraclePriceHistory, Pair, PoolRegistry};
use anchor_lang::prelude::*;
use anchor_spl::token::{Token, TokenAccount};

/// Perform a swap between two assets
#[derive(Accounts)]
pub struct Swap<'info> {
    /// Stores state regarding fees
    #[account(mut, has_one = pool_registry)]
    pub pair: Box<Account<'info, Pair>>,

    #[account(mut)]
    pub pool_registry: AccountLoader<'info, PoolRegistry>,

    /// Wallet performing the swap
    pub user_wallet: Signer<'info>,

    /// CHECK: Uninitialized PDA signer, signs for transfer of `mint_out` from SSL Pool "in".
    pub ssl_pool_in_signer: UncheckedAccount<'info>,

    /// CHECK: Uninitialized PDA signer, signs for transfer of `mint_out` from SSL Pool "out".
    pub ssl_pool_out_signer: UncheckedAccount<'info>,

    /// User signs for a debit to this account in `amount`.
    #[account(mut)]
    pub user_ata_in: Box<Account<'info, TokenAccount>>,

    /// User receives funds in this account.
    #[account(mut)]
    pub user_ata_out: Box<Account<'info, TokenAccount>>,

    /// The SSL-out's vault containing its main token, i.e. the one LPs deposit.
    #[account(
        mut,
        associated_token::mint = user_ata_out.mint,
        associated_token::authority = ssl_pool_out_signer,
    )]
    pub ssl_out_main_vault: Box<Account<'info, TokenAccount>>,

    /// The _non_ main token of SSL-out. For example, if swapping SOL _in_ to USDC out,
    /// this would be the SOL vault of the USDC SSL pool.
    #[account(
        mut,
        associated_token::mint = user_ata_in.mint,
        associated_token::authority = ssl_pool_out_signer,
    )]
    pub ssl_out_secondary_vault: Box<Account<'info, TokenAccount>>,

    #[account(
    mut,
    associated_token::mint = user_ata_in.mint,
    associated_token::authority = ssl_pool_in_signer,
    )]
    pub ssl_in_main_vault: Box<Account<'info, TokenAccount>>,

    /// Potentially we swap out of this pool vault instead.
    #[account(
        mut,
        associated_token::mint = user_ata_out.mint,
        associated_token::authority = ssl_pool_in_signer,
    )]
    pub ssl_in_secondary_vault: Box<Account<'info, TokenAccount>>,

    /// SSL fee vault of output mint, destination for fees shared with LPs.
    #[account(
        mut,
        associated_token::mint = user_ata_out.mint,
        associated_token::authority = pool_registry,
    )]
    pub ssl_out_fee_vault: Box<Account<'info, TokenAccount>>,

    /// The destination for the unshared portion of fees collected.
    /// Specified on `self.pair`, must match the output mint.
    #[account(mut)]
    pub fee_destination: Box<Account<'info, TokenAccount>>,

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

    #[account(mut)]
    pub event_emitter: Account<'info, EventEmitter>,

    pub token_program: Program<'info, Token>,
}
