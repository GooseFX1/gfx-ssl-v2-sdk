use anchor_client::solana_sdk::pubkey::Pubkey;
use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum GfxSslSdkError {
    #[error("Account not found: {0}")]
    AccountNotFound(Pubkey),

    #[error("Could not deserialize {0} as type: {1}")]
    DeserializeFailure(Pubkey, String),

    #[error("SSL Pool not found in pool registry for mint: {0}")]
    PoolNotFound(Pubkey),

    #[error("Pair does not contain both mints: {0} {1}")]
    MintNotInPair(Pubkey, Pubkey),

    #[error("Some required accounts are not updated")]
    RequiredAccountUpdate,

    #[error("Price history accounts need to be updated")]
    PriceHistoryUpdateRequired,

    #[error("The AMM does not support provided mints")]
    UnexpectedMints,

    #[error("Tried initializing a GfxAmm instance with inconsistent account data")]
    InconsistentInitializationData,
}

pub type Result<T> = std::result::Result<T, GfxSslSdkError>;
