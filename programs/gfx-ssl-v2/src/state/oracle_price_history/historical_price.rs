use anchor_lang::prelude::*;
use rust_decimal::Decimal;
#[cfg(feature = "no-entrypoint")]
use std::fmt::{Display, Formatter};
use std::io::Write;

/// Assuming a max 5-second delay, and a new slot every 0.4 sec,
/// 5 / .4 = 12.5. As a robustness measure, rounding down and subtracting 1.
pub const DEFAULT_MINIMUM_ELAPSED_SLOTS: u8 = 11;

/// A decimal type that can be stored on-chain and converted to a [rust_decimal::Decimal].
#[account(zero_copy)]
#[derive(Debug, Default, PartialEq, AnchorSerialize, AnchorDeserialize)]
#[repr(C)]
pub struct HistoricalDecimal {
    pub num: i64,
    pub scale: u32,
    pub inv: f32,
}

impl Into<Decimal> for HistoricalDecimal {
    fn into(self) -> Decimal {
        Decimal::new(self.num, self.scale)
    }
}

impl Into<f64> for HistoricalDecimal {
    fn into(self) -> f64 {
        self.num as f64 / 10f64.powi(self.scale.try_into().unwrap())
    }
}

impl AccountSerialize for HistoricalDecimal {
    fn try_serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(bytemuck::bytes_of(self))?;
        Ok(())
    }
}

/// A common type derivable from any particular oracle price history, whether that
/// be a Pyth oracle, a Switchboard oracle, etc.
///
/// This is not a value stored on-chain, it is merely generated during instruction execution.
/// Therefore it doesn't need to derive any of the Anchor values.
#[account(zero_copy)]
#[derive(Debug, Default, PartialEq, AnchorSerialize, AnchorDeserialize)]
#[repr(C)]
pub struct HistoricalPrice {
    /// The price of the given asset.
    pub price: HistoricalDecimal,
    /// The slot during which this price was recorded.
    /// This should be a conservative value, e.g. if the price history account's latest entry
    /// is from slot X, but the oracle recorded it during slot X-1, the latter should be recorded
    /// here.
    pub slot: u64,
}

#[cfg(feature = "no-entrypoint")]
impl Display for HistoricalPrice {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let price: Decimal = self.price.into();

        write!(f, "Price: {}, Slot: {}", price, self.slot)?;
        Ok(())
    }
}

const _: [u8; 24] = [0u8; std::mem::size_of::<HistoricalPrice>()];

impl AccountSerialize for HistoricalPrice {
    fn try_serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        writer.write_all(bytemuck::bytes_of(self))?;
        Ok(())
    }
}

/// Output type for the [OraclePriceHistory] method that calculates
/// mean and std deviation.
#[derive(Copy, Clone, Debug, Default)]
#[repr(C)]
pub struct BollingerBand {
    /// Exponential moving average
    pub mean: Decimal,
    /// Standard deviation
    pub std: Decimal,
}
