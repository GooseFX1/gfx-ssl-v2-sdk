use crate::PDAIdentifier;
use anchor_lang::prelude::*;
#[cfg(feature = "no-entrypoint")]
use chrono::{DateTime, NaiveDateTime, Utc};
#[cfg(feature = "no-entrypoint")]
use std::fmt::{Display, Formatter};

#[account]
#[derive(Debug)]
pub struct LiquidityAccount {
    pub pool_registry: Pubkey,
    /// Associated mint of the liquidity account.
    pub mint: Pubkey,
    /// The authority over the liquidity account.
    pub owner: Pubkey,
    /// Current deposited liquidity.
    pub amount_deposited: u64,
    /// Last observed total accumulated profit (recorded from the [SSLPool]).
    pub last_observed_tap: u64,
    /// Unix timestamp of the last execution of fee collection.
    pub last_claimed: i64,
    /// Historical value that records the total earned fee revenue
    /// over the lifetime of this account.
    pub total_earned: u64,
    /// Unix timestamp of the time of account creation.
    pub created_at: i64,
    pub _space: [u8; 128],
}

impl Default for LiquidityAccount {
    fn default() -> Self {
        Self {
            pool_registry: Default::default(),
            mint: Default::default(),
            owner: Default::default(),
            amount_deposited: 0,
            last_observed_tap: 0,
            last_claimed: 0,
            total_earned: 0,
            created_at: 0,
            _space: [0; 128],
        }
    }
}

#[cfg(feature = "no-entrypoint")]
impl Display for LiquidityAccount {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Pool Registry: {}", self.pool_registry)?;
        writeln!(f, "Mint: {}", self.mint)?;
        writeln!(f, "Owner: {}", self.owner)?;
        writeln!(f, "Amount deposited: {}", self.amount_deposited)?;
        writeln!(f, "Total earned: {}", self.total_earned)?;
        writeln!(
            f,
            "Last observed total accumulated profit: {}",
            self.last_observed_tap
        )?;
        let last_claimed = DateTime::<Utc>::from_naive_utc_and_offset(
            NaiveDateTime::from_timestamp_opt(self.last_claimed, 0).unwrap(),
            Utc,
        );
        writeln!(f, "Last claimed (UTC): {}", last_claimed)?;
        let created_at = DateTime::<Utc>::from_naive_utc_and_offset(
            NaiveDateTime::from_timestamp_opt(self.created_at, 0).unwrap(),
            Utc,
        );
        write!(f, "Created at: {}", created_at)?;
        Ok(())
    }
}

const _: [u8; 264] = [0; std::mem::size_of::<LiquidityAccount>()];

impl LiquidityAccount {
    pub fn initialize(
        &mut self,
        pool_registry: Pubkey,
        mint: Pubkey,
        owner: Pubkey,
        last_observed_tap: u64,
        created_at: i64,
    ) {
        self.pool_registry = pool_registry;
        self.mint = mint;
        self.owner = owner;
        self.last_observed_tap = last_observed_tap;
        self.created_at = created_at;
    }

    pub fn address(pool_registry: Pubkey, mint: Pubkey, owner: Pubkey) -> Pubkey {
        Self::get_address(&[pool_registry.as_ref(), mint.as_ref(), owner.as_ref()])
    }
}

impl PDAIdentifier for LiquidityAccount {
    const IDENT: &'static [u8] = b"liquidity_account";

    #[inline(always)]
    fn program_id() -> &'static Pubkey {
        &crate::ID
    }
}
