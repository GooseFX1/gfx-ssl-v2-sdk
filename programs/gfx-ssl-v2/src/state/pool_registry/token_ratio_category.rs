use rust_decimal::Decimal;
use anchor_lang::prelude::*;
use crate::{AssetType, PoolRegistry, SSLV2Error};
use crate::utils::u16_to_bps;

/// Values are unpacked during instruction runtime in the following order (input --> output):
///
/// 0. `AssetType::BlueChip` --> `AssetType::BlueChip`
/// 1. `AssetType::BlueChip` --> `AssetType::Stable`
/// 2. `AssetType::BlueChip` --> `AssetType::Volatile`
/// 3. `AssetType::Stable` --> `AssetType::BlueChip`
/// 4. `AssetType::Stable` --> `AssetType::Stable`
/// 5. `AssetType::Stable` --> `AssetType::Volatile`
/// 6. `AssetType::Volatile` --> `AssetType::BlueChip`
/// 7. `AssetType::Volatile` --> `AssetType::Stable`
/// 8. `AssetType::Volatile` --> `AssetType::Volatile`
///
/// which is in order of the `AssetType` enum variants listed below,
/// (all variants except `Uninitialized` and `Invalid`).
pub const ASSET_TYPES: [AssetType; NUM_ASSET_TYPES] = [
    AssetType::BlueChip,
    AssetType::Stable,
    AssetType::Volatile,
];
pub const NUM_ASSET_TYPES: usize = 3;

/// Get the index where a given pool-registry value is stored
pub fn index_of(
    input_token: &AssetType,
    output_token: &AssetType,
) -> std::result::Result<usize, SSLV2Error> {
    let input_token_idx = ASSET_TYPES.iter().position(|t| *t == *input_token)
        .ok_or(SSLV2Error::InvalidAssetType)?;
    let output_token_idx = ASSET_TYPES.iter().position(|t| *t == *output_token)
        .ok_or(SSLV2Error::InvalidAssetType)?;
    Ok((input_token_idx * NUM_ASSET_TYPES) + output_token_idx)
}

impl PoolRegistry {
    /// Read a max pool token ratio from the pool registry
    pub fn max_pool_token_ratio(
        &self,
        input_token: &AssetType,
        output_token: &AssetType,
    ) -> std::result::Result<Decimal, SSLV2Error> {
        let idx = index_of(input_token, output_token)?;
        let raw_value = self.categorical_pool_token_ratios[idx];
        Ok(u16_to_bps(raw_value))
    }

    /// Write a max pool token ratio to the pool registry
    pub fn set_max_pool_token_ratio(
        &mut self,
        input_token: &AssetType,
        output_token: &AssetType,
        new_value: u16,
    ) -> std::result::Result<(), SSLV2Error> {
        let idx = index_of(input_token, output_token)?;
        self.categorical_pool_token_ratios[idx] = new_value;
        Ok(())
    }
}

/// For Anchor instruction encoding.
/// If you need to simply look up the max pool token ratio,
/// use [PoolRegistry::max_pool_token_ratio].
#[derive(Copy, Clone, Debug, Default, AnchorDeserialize, AnchorSerialize)]
#[repr(C)]
pub struct MaxPoolTokenRatio {
    pub input_token: u8,
    pub output_token: u8,
    pub pool_token_ratio: u16,
}

impl MaxPoolTokenRatio {
    pub fn new(
        input_token: AssetType,
        output_token: AssetType,
        pool_token_ratio: u16,
    ) -> Self {
        Self {
            input_token: input_token.into(),
            output_token: output_token.into(),
            pool_token_ratio,
        }
    }
}
