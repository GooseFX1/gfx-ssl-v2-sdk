use anchor_lang::prelude::*;
use crate::{EventEmitter, PDAIdentifier};


/// Creates an [EventEmitter] account.
#[derive(Accounts)]
pub struct CreateEventEmitter<'info> {
    /// Lamports for rent funded from here.
    #[account(mut)]
    pub funder: Signer<'info>,

    #[account(
        init,
        payer = funder,
        space = 8 + std::mem::size_of::<EventEmitter>(),
        seeds = [EventEmitter::IDENT],
        bump,
    )]
    pub event_emitter: Account<'info, EventEmitter>,

    /// Needed to create a new account
    pub system_program: Program<'info, System>,
}