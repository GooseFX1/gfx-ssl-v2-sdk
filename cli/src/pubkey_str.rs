//! # Usage
//!
//! ```
//! use solana_sdk::pubkey::Pubkey;
//!
//! #[derive(serde::Serialize, serde::Deserialize)]
//! pub struct MyStruct {
//!     /// Token pubkey.
//!     #[serde(with = "pubkey")]
//!     pub address: Pubkey,
//! }
//! ```

pub mod pubkey {
    use serde::{self, Deserialize, Deserializer, Serializer};
    pub use solana_sdk::pubkey::Pubkey;
    use std::str::FromStr;

    pub fn serialize<S>(pubkey: &Pubkey, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let s = format!("{}", pubkey);
        serializer.serialize_str(&s)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<Pubkey, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Pubkey::from_str(&s).map_err(serde::de::Error::custom)
    }
}
