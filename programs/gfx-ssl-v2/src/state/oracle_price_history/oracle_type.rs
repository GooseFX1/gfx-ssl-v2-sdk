#[cfg(feature = "no-entrypoint")]
use std::fmt::{Display, Formatter};

/// Disambiguates between the various types of oracles we might use to capture historical
/// price data.
///
/// Each oracle price data needs to be computed in its own special way, depending
/// on the type of oracle whose historical data is being recorded.
///
/// For [Pod] safety this type is converted to/from a [u8] when represented
/// as on-chain account data.
#[derive(Copy, Clone, Debug, Default)]
#[cfg_attr(
    feature = "no-entrypoint",
    derive(serde::Serialize, serde::Deserialize)
)]
pub enum OracleType {
    #[default]
    Uninitialized,
    /// [pyth_sdk_solana::PriceFeed]
    Pyth,
    /// [switchboard_v2::AggregatorAccountData]
    Switchboardv2,
    /// To catch any invalid bit-patterns
    Invalid,
}

#[cfg(feature = "no-entrypoint")]
impl Display for OracleType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match *self {
            OracleType::Uninitialized => write!(f, "Uninitialized"),
            OracleType::Pyth => write!(f, "Pyth"),
            OracleType::Switchboardv2 => write!(f, "SwitchBoard v2"),
            OracleType::Invalid => write!(f, "Invalid"),
        }
    }
}

impl From<u8> for OracleType {
    fn from(value: u8) -> Self {
        match value {
            0 => OracleType::Uninitialized,
            1 => OracleType::Pyth,
            2 => OracleType::Switchboardv2,
            _ => OracleType::Invalid,
        }
    }
}

impl Into<u8> for OracleType {
    fn into(self) -> u8 {
        match self {
            OracleType::Uninitialized => 0,
            OracleType::Pyth => 1,
            OracleType::Switchboardv2 => 2,
            OracleType::Invalid => u8::MAX,
        }
    }
}
