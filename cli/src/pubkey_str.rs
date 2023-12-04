//! # Usage
//!
//! ```ignore
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

pub mod pubkey_array {
    use serde::{self, Deserializer, Serializer};
    pub use solana_sdk::pubkey::Pubkey;
    use serde::ser::SerializeSeq;
    use gfx_ssl_v2_interface::ssl_pool::MAX_NUM_ORACLES_PER_MINT;

    pub fn serialize<S>(pubkeys: &[Pubkey; MAX_NUM_ORACLES_PER_MINT], serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(MAX_NUM_ORACLES_PER_MINT))?;

        // Iterate over the elements, convert them to String and serialize them
        for element in &pubkeys.to_vec() {
            seq.serialize_element(&element.to_string())?;
        }

        seq.end()
    }

    pub fn deserialize<'de, D>(_deserializer: D) -> Result<[Pubkey; MAX_NUM_ORACLES_PER_MINT], D::Error>
        where
            D: Deserializer<'de>,
    {
        todo!()
    }
}

pub mod pubkey_pair {
    use serde::{self, Serializer};
    pub use solana_sdk::pubkey::Pubkey;
    use serde::ser::SerializeSeq;

    pub fn serialize<S>(pubkeys: &(Pubkey, Pubkey), serializer: S) -> Result<S::Ok, S::Error>
        where
            S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(2))?;

        // Iterate over the elements, convert them to String and serialize them
        seq.serialize_element(&pubkeys.0.to_string())?;
        seq.serialize_element(&pubkeys.1.to_string())?;
        seq.end()
    }
}
