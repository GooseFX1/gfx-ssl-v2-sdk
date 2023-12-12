#[cfg(feature = "no-entrypoint")]
use crate::utils::{u128_from_bytes, u16_to_bps};
use crate::{utils::token_amount, PDAIdentifier, SSLV2Error};
use anchor_lang::prelude::*;
use rust_decimal::Decimal;
#[cfg(feature = "no-entrypoint")]
use std::fmt::{Display, Formatter};
use std::mem;

/// Scale used to record the historical USD volume swapped.
const USD_VOLUME_DECIMALS: u32 = 6;

/// An account that provides additional bookkeeping for a swap market
/// that is made available through the existence of a given pair of SSL pools
/// under the same admin. When a user is swapping between two mints,
/// they're swapping between two SSL pools, and this [Pair] account type
/// adds additional bookkeeping across that swap pair.
#[account]
#[derive(Debug, PartialEq)]
pub struct Pair {
    /// Associated admin account.
    pub pool_registry: Pubkey,
    /// The two mints that make up the pair.
    /// These mints are always ordered "numerically".
    pub mints: (Pubkey, Pubkey),

    /// Fees go to this pubkey's associated token account for the input mint of a swap.
    /// That means if a user is swapping SOL for USDC, they're charged fees in SOL,
    /// and we would move fees to the wrapped SOL associated token account owned by `self.fee_collector`.
    pub fee_collector: (Pubkey, Pubkey),
    /// Matched to mints.0 and mints.1 respectively.
    /// Fee rates are allowed to be different depending on the input mint.
    /// They are expressed as BPS (theoretical maximum = 10_000).
    pub fee_rates: (u16, u16),

    /// Historical value that records the total amount of fees collected
    /// over the lifetime of the pair.
    /// Recorded as native token amounts.
    pub total_fees_generated_native: ([u8; 16], [u8; 16]),

    /// Total volume swapped on this pair. Parsed as a u128, little-endian.
    /// Scaled to [USD_VOLUME_DECIMALS].
    pub total_historical_volume: [u8; 16],

    /// Historical value that records the total amount of internally swapped
    /// volume over the lifetime of the pair.
    /// Recorded as native token amounts.
    pub total_internally_swapped: ([u8; 16], [u8; 16]),

    pub _space: [u8; 128],
}

// Compile-time struct size check. Successful deserialization requires
// that the compiler's target architecture agrees with what's on-chain.
const _: [u8; 372] = [0; mem::size_of::<Pair>()];

impl Default for Pair {
    fn default() -> Self {
        Self {
            pool_registry: Default::default(),
            mints: (Default::default(), Default::default()),
            fee_collector: (Default::default(), Default::default()),
            fee_rates: (0, 0),
            total_fees_generated_native: ([0; 16], [0; 16]),
            total_historical_volume: [0; 16],
            total_internally_swapped: ([0; 16], [0; 16]),
            _space: [0; 128],
        }
    }
}

#[cfg(feature = "no-entrypoint")]
impl Display for Pair {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let total_historical_volume = token_amount::u128_to_ui(
            u128_from_bytes(&self.total_historical_volume),
            USD_VOLUME_DECIMALS,
        );
        writeln!(
            f,
            "Total historical volume (USD): {}",
            total_historical_volume
        )?;
        writeln!(f, "Mint A: {}", self.mints.0)?;
        writeln!(f, "\tExternal Fee Destination: {}", self.fee_collector.0)?;
        writeln!(f, "\tFee Rate {}", u16_to_bps(self.fee_rates.0))?;
        writeln!(
            f,
            "\tFees Generated (Native): {}",
            u128_from_bytes(&self.total_fees_generated_native.0)
        )?;
        writeln!(
            f,
            "\tTotal Internally Swapped: {}",
            u128_from_bytes(&self.total_internally_swapped.0)
        )?;
        writeln!(f, "")?;
        writeln!(f, "Mint B: {}", self.mints.1)?;
        writeln!(f, "\tExternal Fee Destination: {}", self.fee_collector.1)?;
        writeln!(f, "\tFee Rate {}", u16_to_bps(self.fee_rates.1))?;
        writeln!(
            f,
            "\tFees Generated (Native): {}",
            u128_from_bytes(&self.total_fees_generated_native.1)
        )?;
        write!(
            f,
            "\tTotal Internally Swapped: {}",
            u128_from_bytes(&self.total_internally_swapped.1)
        )?;
        Ok(())
    }
}

impl Pair {
    /// Puts the mints into correct order,
    /// but does not enforce the failure condition where mint_one == mint_two.
    pub fn normalize_mint_order(mint_one: Pubkey, mint_two: Pubkey) -> (Pubkey, Pubkey) {
        if mint_one < mint_two {
            (mint_one, mint_two)
        } else {
            (mint_two, mint_one)
        }
    }

    /// Uses [Pair::normalize_mint_order] to normalize mint ordering.
    pub fn address(pool_registry: Pubkey, mint_one: Pubkey, mint_two: Pubkey) -> Pubkey {
        let (m1, m2) = Self::normalize_mint_order(mint_one, mint_two);
        Self::get_address(&[pool_registry.key().as_ref(), m1.as_ref(), m2.as_ref()])
    }

    /// Anchor account initialization is done with a macro attribute
    /// in an `Accounts` context. This initialization function should be
    /// executed in the instruction body.
    pub fn initialize(
        &mut self,
        pool_registry: Pubkey,
        mint_one: Pubkey,
        mint_two: Pubkey,
        mint_one_fee_destination: Pubkey,
        mint_two_fee_destination: Pubkey,
        mint_one_fee_rate: u16,
        mint_two_fee_rate: u16,
    ) {
        self.pool_registry = pool_registry;
        self.mints = (mint_one, mint_two);
        self.fee_collector = (mint_one_fee_destination, mint_two_fee_destination);
        self.fee_rates = (mint_one_fee_rate, mint_two_fee_rate);
    }

    /// Find the appropriate fee rate and collector, and verify the in/out mints match the pair.
    /// The fee rate and destination should always come from the output mint.
    pub fn find_fee_attrs(
        &self,
        mint_in: Pubkey,
        mint_out: Pubkey,
    ) -> std::result::Result<(Decimal, Pubkey, SwapIxMintOrdering), SSLV2Error> {
        if (mint_in, mint_out) == self.mints {
            Ok((
                Decimal::new(self.fee_rates.1 as i64, 4),
                self.fee_collector.1,
                SwapIxMintOrdering::InOut,
            ))
        } else if (mint_out, mint_in) == self.mints {
            Ok((
                Decimal::new(self.fee_rates.0 as i64, 4),
                self.fee_collector.0,
                SwapIxMintOrdering::InOut,
            ))
        } else {
            Err(SSLV2Error::MintNotFound)
        }
    }

    pub fn historical_volume(&self) -> u128 {
        u128_from_bytes(&self.total_historical_volume)
    }

    pub fn total_fees_generated(&self) -> (u128, u128) {
        (
            u128_from_bytes(&self.total_fees_generated_native.0),
            u128_from_bytes(&self.total_fees_generated_native.1),
        )
    }

    pub fn total_internally_swapped(&self) -> (u128, u128) {
        (
            u128_from_bytes(&self.total_internally_swapped.0),
            u128_from_bytes(&self.total_internally_swapped.1),
        )
    }
}

impl PDAIdentifier for Pair {
    const IDENT: &'static [u8] = b"pair";

    #[inline(always)]
    fn program_id() -> &'static Pubkey {
        &crate::ID
    }
}

/// The output mint of a swap could either be the first mint in the pair, or the second.
/// This enum denotes which is the case.
pub enum SwapIxMintOrdering {
    /// When pair.mints = (mint_in, mint_out)
    InOut,
    /// When pair.mints = (mint_out, mint_in)
    OutIn,
}
