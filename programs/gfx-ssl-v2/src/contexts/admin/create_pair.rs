use crate::pool_registry::PoolRegistry;
use crate::PDAIdentifier;
use crate::Pair;
use crate::SSLPool;
use crate::SSLV2Error;
use anchor_lang::prelude::*;
use anchor_spl::associated_token::AssociatedToken;
use anchor_spl::token::{Mint, Token, TokenAccount};

/// Suspend the SSL pool or update it with new price calculation parameters.
///
/// Note: Fee parameters are contained on the `Pair` account.
#[derive(Accounts)]
pub struct CreatePair<'info> {
    /// The global registry of SSL pools.
    #[account(has_one = admin @ SSLV2Error::NotAdmin)]
    pub pool_registry: AccountLoader<'info, PoolRegistry>,

    /// A mint associated with an [SSLPool], and one of the tokens in the newly created [Pair].
    pub mint_one: Account<'info, Mint>,

    /// A mint associated with an [SSLPool], and one of the tokens in the newly created [Pair].
    pub mint_two: Account<'info, Mint>,

    /// Fees collected of `mint_one` will flow here.
    pub mint_one_fee_destination: Account<'info, TokenAccount>,

    /// Fees collected of `mint_two` will flow here.
    pub mint_two_fee_destination: Account<'info, TokenAccount>,

    #[account(
        init,
        payer = admin,
        space = 8 + std::mem::size_of::<Pair>(),
        seeds = [
            Pair::IDENT,
            pool_registry.key().as_ref(),
            mint_one.key().as_ref(),
            mint_two.key().as_ref(),
        ],
        bump,
    )]
    pub pair: Box<Account<'info, Pair>>,

    #[account(
        seeds = [
            SSLPool::IDENT,
            pool_registry.key().as_ref(),
            mint_one.key().as_ref(),
        ],
        bump,
    )]
    /// CHECK: Uninitialized PDA signer
    pub ssl_pool_one_signer: UncheckedAccount<'info>,

    #[account(
        seeds = [
            SSLPool::IDENT,
            pool_registry.key().as_ref(),
            mint_two.key().as_ref(),
        ],
        bump,
    )]
    /// CHECK: Uninitialized PDA signer
    pub ssl_pool_two_signer: UncheckedAccount<'info>,

    /// Mint one SSL Pool's vault for mint two
    #[account(
        init,
        associated_token::mint = mint_two,
        associated_token::authority = ssl_pool_one_signer,
        payer = admin
    )]
    pub mint_one_secondary_vault: Box<Account<'info, TokenAccount>>,

    /// Mint two SSL Pool's vault for mint one
    #[account(
        init,
        associated_token::mint = mint_one,
        associated_token::authority = ssl_pool_two_signer,
        payer = admin
    )]
    pub mint_two_secondary_vault: Box<Account<'info, TokenAccount>>,

    /// The pool registry admin. This makes the instruction permissioned.
    /// This admin address is also recorded on the [Pair].
    #[account(mut)]
    pub admin: Signer<'info>,

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
