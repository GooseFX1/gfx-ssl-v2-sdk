use gfx_ssl_v2_interface::{
    utils::{u16_to_bps, u32_to_bps},
    SSLMathParams,
};
use serde::{Deserialize, Serialize};

/// Raw data with serde traits, skipping padding and extra space fields
#[derive(Serialize, Deserialize, Clone)]
pub struct SSLMathParamsRawData {
    pub mean_window: u8,
    pub std_window: u8,
    pub fixed_price_distance: u16,
    pub minimum_price_distance: u16,
    pub latest_price_weight: u16,
    pub std_weight: u32,
}

impl From<&SSLMathParams> for SSLMathParamsRawData {
    fn from(value: &SSLMathParams) -> Self {
        Self {
            mean_window: value.mean_window,
            std_window: value.std_window,
            fixed_price_distance: value.fixed_price_distance,
            minimum_price_distance: value.minimum_price_distance,
            latest_price_weight: value.latest_price_weight,
            std_weight: value.std_weight,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
pub struct SSLMathParamsUiData {
    pub mean_window: u8,
    pub std_window: u8,
    pub fixed_price_distance: String,
    pub minimum_price_distance: String,
    pub latest_price_weight: String,
    pub std_weight: String,
}

impl From<&SSLMathParams> for SSLMathParamsUiData {
    fn from(value: &SSLMathParams) -> Self {
        Self {
            mean_window: value.mean_window,
            std_window: value.std_window,
            fixed_price_distance: u16_to_bps(value.fixed_price_distance).to_string(),
            minimum_price_distance: u16_to_bps(value.minimum_price_distance).to_string(),
            latest_price_weight: u16_to_bps(value.latest_price_weight).to_string(),
            std_weight: u32_to_bps(value.std_weight).to_string(),
        }
    }
}
