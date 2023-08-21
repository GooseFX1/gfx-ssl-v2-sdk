use anchor_lang::prelude::Pubkey;
use rust_decimal::Decimal;

/// Convert between native u64 token amounts
/// and their [rust_decimal::Decimal] "UI" representations.
/// Native == Lamports
/// "UI" == SOL
pub mod token_amount {
    use num_traits::ToPrimitive;
    use rust_decimal::Decimal;

    /// Convert a [Decimal] to a native u64 amount.
    /// You must pass the mint's `decimals` as the scale.
    pub fn to_native(mut amount: Decimal, scale: u32) -> u64 {
        if amount.scale() < scale {
            amount.rescale(scale);
        }
        amount.set_scale(amount.scale() - scale).unwrap();
        amount.to_u64().unwrap()
    }

    /// Convert a u64 value to a "UI" [Decimal] representation.
    /// You must pass the mint's `decimals` as the scale.
    pub fn to_ui(amount: u64, scale: u32) -> Decimal {
        let mut amount = Decimal::from(amount);
        amount.set_scale(scale).unwrap();

        amount
    }

    /// Convert a u128 value to a "UI" [Decimal] representation.
    /// You must pass the mint's `decimals` as the scale.
    pub fn u128_to_ui(amount: u128, scale: u32) -> Decimal {
        let mut amount = Decimal::from(amount);
        amount.set_scale(scale).unwrap();

        amount
    }
}

pub trait PDAIdentifier {
    const IDENT: &'static [u8];

    fn program_id() -> &'static Pubkey;

    fn get_address(seeds: &[&[u8]]) -> Pubkey {
        Self::get_address_with_bump(seeds).0
    }

    fn get_address_with_bump(seeds: &[&[u8]]) -> (Pubkey, u8) {
        let mut seeds = seeds.to_vec();
        seeds.insert(0, Self::IDENT);
        Pubkey::find_program_address(&seeds, Self::program_id())
    }
}

pub fn u128_from_bytes(value: &[u8; 16]) -> u128 {
    *bytemuck::from_bytes::<u128>(&value[..])
}

/// Convert u16 representing percentage basis-points (BPS) to a [Decimal].
pub fn u16_to_bps(val: u16) -> Decimal {
    Decimal::new(val as i64, 4)
}

/// Convert u32 representing percentage basis-points (BPS) to a [Decimal].
pub fn u32_to_bps(val: u32) -> Decimal {
    Decimal::new(val as i64, 4)
}
