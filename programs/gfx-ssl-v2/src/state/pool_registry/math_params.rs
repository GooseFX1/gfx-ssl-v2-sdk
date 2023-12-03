use anchor_lang::prelude::*;
use bytemuck::{Pod, Zeroable};
#[cfg(feature = "no-entrypoint")]
use std::fmt::{Display, Formatter};

/// Identical to [SSLMathParams] except that every field is an `Option` type,
/// which mitigates the possibility of error when configuring the math by marking
/// any field as `None` which is not intended for update.
#[derive(Copy, Clone, Debug, Default, AnchorDeserialize, AnchorSerialize)]
#[repr(C)]
pub struct SSLMathConfig {
    pub mean_window: Option<u8>,
    pub std_window: Option<u8>,
    pub fixed_price_distance: Option<u16>,
    pub minimum_price_distance: Option<u16>,
    pub std_weight: Option<u32>,
    pub latest_price_weight: Option<u16>,
}

/// The set of configurable parameters for each SSL pool.
/// These control price calculation, and thresholds for various conditionals
/// that may force the failure of an attempted swap.
#[cfg_attr(feature = "python", pyo3::pyclass)]
#[cfg_attr(
    feature = "no-entrypoint",
    derive(serde::Serialize, serde::Deserialize)
)]
#[derive(
    Copy, Clone, Debug, Default, PartialEq, AnchorDeserialize, AnchorSerialize, Zeroable, Pod,
)]
#[repr(C)]
pub struct SSLMathParams {
    /// The number of elements included in mean calculation.
    pub mean_window: u8,
    /// The number of elements included in std deviation calculation.
    pub std_window: u8,

    /// A percentage distance from the latest oracle price expressed in BPS.
    /// This is not a minimum or a maximum, it is an arbitrarily picked target fraction
    /// of the price, and it is added to the mean price along with a few other
    /// parameters.
    pub fixed_price_distance: u16,

    /// A minimum distance from the latest oracle price expressed in BPS.
    pub minimum_price_distance: u16,

    /// Maximum allowed ratio of this SSL's main token residing in other pools divided
    /// by the amount of main token in this SSL pool.
    /// This number is a percentage expressed in BPS.
    pub _deprecated: u16,

    /// A weight that controls the price influence ratio between
    /// the mean price and the latest price.
    /// This number is a percentage expressed in BPS.
    pub latest_price_weight: u16,

    #[cfg_attr(
        feature = "no-entrypoint",
        serde(default = "crate::state::pool_registry::math_params::default_padding6")
    )]
    pub _pad0: [u8; 6],

    /// A weight used to control how much influence the
    /// std deviation should have on bid/ask prices.
    /// This number is a percentage expressed in BPS, and is _not_ clamped to n <= 100%.
    pub std_weight: u32,

    #[cfg_attr(
        feature = "no-entrypoint",
        serde(default = "crate::state::pool_registry::math_params::default_padding4")
    )]
    pub _pad1: [u8; 4],

    #[cfg_attr(
        feature = "no-entrypoint",
        serde(default = "crate::state::pool_registry::math_params::default_padding32")
    )]
    pub _space: [u8; 32],
}

impl SSLMathParams {
    pub fn configure(&mut self, config: SSLMathConfig) {
        if let Some(val) = config.mean_window {
            self.mean_window = val;
        }
        if let Some(val) = config.std_window {
            self.std_window = val;
        }
        if let Some(val) = config.fixed_price_distance {
            self.fixed_price_distance = val;
        }
        if let Some(val) = config.minimum_price_distance {
            self.minimum_price_distance = val;
        }
        if let Some(val) = config.std_weight {
            self.std_weight = val;
        }
        if let Some(val) = config.latest_price_weight {
            self.latest_price_weight = val;
        }
    }
}

#[cfg_attr(feature = "python", pyo3::pymethods)]
impl SSLMathParams {
    #[cfg(feature = "python")]
    #[new]
    pub fn new_py(
        mean_window: u8,
        std_window: u8,
        fixed_price_distance: u16,
        minimum_price_distance: u16,
        latest_price_weight: u16,
        std_weight: u32,
    ) -> Self {
        Self {
            mean_window,
            std_window,
            fixed_price_distance,
            minimum_price_distance,
            latest_price_weight,
            _deprecated: 0,
            _pad0: [0; 6],
            std_weight,
            _pad1: [0; 4],
            _space: [0; 32],
        }
    }
}

#[cfg(feature = "no-entrypoint")]
impl Display for SSLMathParams {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Mean window {}", self.mean_window)?;
        writeln!(f, "Standard deviation window {}", self.std_window)?;
        writeln!(f, "Standard deviation weight {}", self.std_weight)?;
        writeln!(f, "Fixed price distance {}", self.fixed_price_distance)?;
        writeln!(f, "Minimum price distance {}", self.minimum_price_distance)?;
        write!(f, "Latest price weight {}", self.latest_price_weight)?;

        Ok(())
    }
}

// Compile-time struct size check. Successful deserialization requires
// that the compiler's target architecture agrees with what's on-chain.
const _: [u8; 56] = [0; std::mem::size_of::<SSLMathParams>()];

// For deserialization of [SSLMathParams], serde doesn't take generics
#[cfg(feature = "no-entrypoint")]
pub fn default_padding6() -> [u8; 6] {
    [0u8; 6]
}

// For deserialization of [SSLMathParams], serde doesn't take generics
#[cfg(feature = "no-entrypoint")]
pub fn default_padding4() -> [u8; 4] {
    [0u8; 4]
}

// For deserialization of [SSLMathParams], serde doesn't take generics
#[cfg(feature = "no-entrypoint")]
pub fn default_padding32() -> [u8; 32] {
    [0u8; 32]
}
