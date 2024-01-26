use crate::{
    sdk::error::GfxSslSdkError,
    token_ratio_category::{
        index_of, MaxPoolTokenRatio, ASSET_TYPES, NUM_ASSET_TYPES, NUM_POOL_TOKEN_RATIOS,
    },
    AssetType, PoolRegistry, PoolRegistryConfig, SSLV2Error,
};
use serde::{Deserialize, Serialize};

/// For Anchor instruction encoding.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct UiPoolRegistryConfig(Vec<UiMaxPoolTokenRatio>);

/// For Anchor instruction encoding.
#[derive(Clone, Copy, Debug, Default, PartialEq, Serialize, Deserialize)]
#[repr(C)]
pub struct UiMaxPoolTokenRatio {
    pub input_token: AssetType,
    pub output_token: AssetType,
    pub pool_token_ratio: u16,
}

impl UiMaxPoolTokenRatio {
    pub fn from_ratio_at_index(idx: usize, pool_token_ratio: u16) -> Option<Self> {
        if idx >= 9 {
            return None;
        }
        let input_token = ASSET_TYPES[idx / NUM_ASSET_TYPES];
        let output_token = ASSET_TYPES[idx % NUM_ASSET_TYPES];
        Some(Self {
            input_token,
            output_token,
            pool_token_ratio,
        })
    }

    pub fn validate_complete_list(ratios: &[Self]) -> crate::sdk::error::Result<[Self; 9]> {
        for input_asset in ASSET_TYPES {
            for output_asset in ASSET_TYPES {
                let matched = ratios
                    .iter()
                    .filter(|ratio| {
                        ratio.input_token == input_asset && ratio.output_token == output_asset
                    })
                    .collect::<Vec<_>>();
                if matched.is_empty() {
                    return Err(GfxSslSdkError::MissingTokenRatio(input_asset, output_asset));
                }
                if matched.len() != 1 {
                    return Err(GfxSslSdkError::DuplicateTokenRatio(
                        input_asset,
                        output_asset,
                    ));
                }
            }
        }
        let mut arr = [UiMaxPoolTokenRatio {
            input_token: AssetType::Uninitialized,
            output_token: AssetType::Uninitialized,
            pool_token_ratio: 0,
        }; 9];
        arr.copy_from_slice(&ratios);
        Ok(arr)
    }

    pub fn from_pool_registry(pool_registry: &PoolRegistry) -> Vec<Self> {
        pool_registry
            .categorical_pool_token_ratios
            .iter()
            .enumerate()
            .filter_map(|(idx, ratio)| UiMaxPoolTokenRatio::from_ratio_at_index(idx, *ratio))
            .collect()
    }

    /// The items passed in must be deduplicated. Duplicates are "erased"
    /// such that the last item in the input is kept.
    pub fn normalize_order(items: &[Self]) -> Result<Vec<Self>, SSLV2Error> {
        let mut normalized = vec![UiMaxPoolTokenRatio::default(); NUM_POOL_TOKEN_RATIOS];
        for item in items {
            let idx = index_of(&item.input_token, &item.output_token)?;
            normalized[idx] = *item;
        }
        Ok(normalized)
    }
}

impl Into<MaxPoolTokenRatio> for UiMaxPoolTokenRatio {
    fn into(self) -> MaxPoolTokenRatio {
        MaxPoolTokenRatio {
            input_token: self.input_token.into(),
            output_token: self.output_token.into(),
            pool_token_ratio: self.pool_token_ratio,
        }
    }
}

impl Into<PoolRegistryConfig> for UiPoolRegistryConfig {
    fn into(self) -> PoolRegistryConfig {
        PoolRegistryConfig {
            new_admin: None,
            new_suspend_admin: None,
            max_pool_token_ratios: self.0.into_iter().map(|r| r.into()).collect(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{sdk::ui_types::UiMaxPoolTokenRatio, token_ratio_category};

    #[cfg(feature = "no-entrypoint")]
    #[test]
    fn token_ratios_by_index() {
        for i in 0usize..9 {
            let ratio = UiMaxPoolTokenRatio::from_ratio_at_index(i, 99).unwrap();
            assert_eq!(
                i,
                token_ratio_category::index_of(&ratio.input_token, &ratio.output_token).unwrap()
            );
        }
        for i in 9usize..100 {
            assert!(UiMaxPoolTokenRatio::from_ratio_at_index(i, 99).is_none());
        }
    }
}
