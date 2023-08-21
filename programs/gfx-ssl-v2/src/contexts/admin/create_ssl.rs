use crate::errors::SSLV2Error;
use crate::pool_registry::ssl_pool::SSLPool;
use crate::pool_registry::PoolRegistry;
use crate::OraclePriceHistory;
use crate::PDAIdentifier;
use anchor_lang::prelude::*;
use anchor_spl::{
    associated_token::AssociatedToken,
    token::{Mint, Token, TokenAccount},
};

/// Adds an entry to a [PoolRegistry], and creates a [PoolTokenBalanceRegistry] account.
/// Also creates the associated token account for an [SSLPool]'s main token.
/// Also creates the oracle price history account on a target oracle.
///
/// **This instruction does not create any non-main token accounts.**
///
/// These will be created with the `create_pair` instruction. This is important because
/// until the `create_pair` instruction is executed for all pairs that are entailed by adding
/// a new [SSLPool], all swaps and crank instructions will fail, as those non-main token
/// accounts are required for execution of those instructions.
#[derive(Accounts)]
pub struct CreateSsl<'info> {
    /// The admin. This makes the instruction permissioned.
    #[account(mut)]
    pub admin: Signer<'info>,

    /// The global registry of SSL pools. This instruction adds an entry on this.
    #[account(
        mut,
        has_one = admin @ SSLV2Error::NotAdmin
    )]
    pub pool_registry: AccountLoader<'info, PoolRegistry>,

    /// The mint associated with this SSL, i.e. its "main token".
    pub mint: Account<'info, Mint>,

    /// CHECK: Uninitialized PDA signer
    #[account(
        seeds = [SSLPool::IDENT, pool_registry.key().as_ref(), mint.key().as_ref()],
        bump,
    )]
    pub ssl_pool_signer: UncheckedAccount<'info>,

    /// The SSL's main token vault. User deposits are stored here.
    #[account(
        init,
        associated_token::mint = mint,
        associated_token::authority = ssl_pool_signer,
        payer = admin
    )]
    pub pool_vault: Box<Account<'info, TokenAccount>>,

    /// The SSL's fee vault. Claimable fee revenue accrues here.
    /// Fees accrue on swaps where this SSL's main token is the output token.
    #[account(
        init,
        associated_token::mint = mint,
        associated_token::authority = pool_registry,
        payer = admin
    )]
    pub pool_fee_vault: Box<Account<'info, TokenAccount>>,

    /// The admin must provide an initial deposit to the new SSL pool
    #[account(
        mut,
        associated_token::mint = mint,
        associated_token::authority = admin,
    )]
    pub admin_pool_mint_ata: Box<Account<'info, TokenAccount>>,

    /// Stores the oracle price history for the new asset.
    #[account(
        init,
        payer = admin,
        space = 8 + std::mem::size_of::<OraclePriceHistory>(),
        seeds = [OraclePriceHistory::IDENT, pool_registry.key().as_ref(), oracle_account.key().as_ref()],
        bump,
    )]
    pub oracle_price_history: AccountLoader<'info, OraclePriceHistory>,
    /// CHECK: The oracle account whose history will be recorded on the account history.
    pub oracle_account: UncheckedAccount<'info>,

    /// Needed for a token transfer in this instruction,
    /// and for creating associated token accounts
    pub token_program: Program<'info, Token>,
    /// Needed to create an associated token account
    pub associated_token_program: Program<'info, AssociatedToken>,
    /// Needed to create a new account
    pub system_program: Program<'info, System>,
    /// Needed to create an associated token account
    pub rent: Sysvar<'info, Rent>,
}
