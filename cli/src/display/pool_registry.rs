use crate::{
    display::ssl_pool::{SSLPoolRawData, SSLPoolUiData},
    pubkey_str::pubkey,
};
use gfx_ssl_v2_interface::MAX_SSL_POOLS_PER_ADMIN;
use serde::Serialize;
use solana_sdk::pubkey::Pubkey;

#[derive(Serialize, Clone)]
pub struct PoolRegistryRawData {
    #[serde(with = "pubkey")]
    pub admin: Pubkey,
    #[serde(with = "pubkey")]
    pub seed: Pubkey,
    #[serde(with = "pubkey")]
    pub suspend_admin: Pubkey,
    pub bump: u8,
    pub num_entries: u32,
    pub entries: [SSLPoolRawData; MAX_SSL_POOLS_PER_ADMIN],
}

#[derive(Serialize, Clone)]
pub struct PoolRegistryUiData {
    #[serde(with = "pubkey")]
    pub admin: Pubkey,
    #[serde(with = "pubkey")]
    pub seed: Pubkey,
    #[serde(with = "pubkey")]
    pub suspend_admin: Pubkey,
    #[serde(with = "entries")]
    pub entries: [SSLPoolUiData; MAX_SSL_POOLS_PER_ADMIN],
}

pub mod entries {
    use crate::display::ssl_pool::SSLPoolUiData;
    use gfx_ssl_v2_interface::{SSLPoolStatus, MAX_SSL_POOLS_PER_ADMIN};
    use serde::{self, ser::SerializeSeq, Deserializer, Serializer};
    pub use solana_sdk::pubkey::Pubkey;

    pub fn serialize<S>(
        entries: &[SSLPoolUiData; MAX_SSL_POOLS_PER_ADMIN],
        serializer: S,
    ) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(Some(MAX_SSL_POOLS_PER_ADMIN))?;

        // Skip uninitialized entries
        for element in &entries.to_vec() {
            if element.status != SSLPoolStatus::Uninitialized {
                seq.serialize_element(&element)?;
            }
        }

        seq.end()
    }

    pub fn deserialize<'de, D>(
        _deserializer: D,
    ) -> Result<[SSLPoolUiData; MAX_SSL_POOLS_PER_ADMIN], D::Error>
    where
        D: Deserializer<'de>,
    {
        todo!()
    }
}
