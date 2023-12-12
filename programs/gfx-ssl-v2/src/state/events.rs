use crate::PDAIdentifier;
use anchor_lang::prelude::*;

/// Tracker for event emission.
#[account]
#[derive(Debug)]
pub struct EventEmitter {
    /// One-up, for tracking gaps in recorded program history
    pub event_id: i64,
}

impl PDAIdentifier for EventEmitter {
    const IDENT: &'static [u8] = b"event";

    #[inline(always)]
    fn program_id() -> &'static Pubkey {
        &crate::ID
    }
}

impl EventEmitter {
    pub fn address() -> Pubkey {
        Self::get_address(&[])
    }
}
