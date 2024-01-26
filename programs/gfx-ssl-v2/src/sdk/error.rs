use crate::{AssetType, SSLV2Error};
use anchor_lang::solana_program::pubkey::Pubkey;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum GfxSslSdkError {
    #[error("Account not found: {0}")]
    AccountNotFound(Pubkey),

    #[error("Could not deserialize {0} as type: {1}")]
    DeserializeFailure(Pubkey, String),

    #[error("SSL Pool not found in pool registry for mint: {0}")]
    PoolNotFound(Pubkey),

    #[error("Pair does not contain both mints: {0} {1}")]
    MintNotInPair(Pubkey, Pubkey),
    // TODO Add these to Jupiter (after migrating to BPF-based quote)
    //
    // #[error("Some required accounts are not updated")]
    // RequiredAccountUpdate,
    //
    // #[error("Price history accounts need to be updated")]
    // PriceHistoryUpdateRequired,
    //
    // #[error("The AMM does not support provided mints")]
    // UnexpectedMints,
    #[error("Mint not found {0}")]
    UnknownMint(Pubkey),

    #[error("Mint not found {0}")]
    UnknownMintStr(String),

    /// Failed attempt to index on a `HashMap<Pubkey, OraclePriceHistory>`, keyed by mint.
    #[error("Missing oracle price history for mint {0}")]
    MissingOraclePriceHistoryEntry(Pubkey),

    /// Failed attempt to index on a `HashMap<Pubkey, Pair>`, keyed by pair address.
    #[error("Missing pair account {0}")]
    MissingPairEntry(Pubkey),

    #[error("Tried initializing a GfxAmm instance with inconsistent account data")]
    InconsistentInitializationData,

    #[error("Missing categorical pool token ratio {0} -> {1}")]
    MissingTokenRatio(AssetType, AssetType),

    #[error("Duplicate categorical pool token ratio {0} -> {1}")]
    DuplicateTokenRatio(AssetType, AssetType),

    #[error("HistoricalPrice could not be parsed")]
    HistoricalPriceParseError,

    #[error("SSLv2 Program Error: {0:?}")]
    SSLv2Error(SSLV2Error),

    #[error("Missing an expected configuration parameter during comparison: {0}")]
    MissingConfigurationSpaceParam(String),
}

pub type Result<T> = std::result::Result<T, GfxSslSdkError>;

impl From<SSLV2Error> for GfxSslSdkError {
    fn from(value: SSLV2Error) -> Self {
        Self::SSLv2Error(value)
    }
}
