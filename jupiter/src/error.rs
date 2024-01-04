use solana_sdk::pubkey::Pubkey;
use thiserror::Error;

#[derive(Debug, Clone, Error)]
pub enum GfxJupiterIntegrationError {
    #[error("Could not deserialize {0} as type: {1}")]
    DeserializeFailure(Pubkey, String),

    #[error("Math error")]
    MathError,

    #[error("SSL Pool not found in pool registry for mint: {0}")]
    PoolNotFound(Pubkey),

    #[error("Some required accounts are not updated")]
    RequiredAccountUpdate,

    #[error("Could not resolve fee destination from pair")]
    CannotResolveFeeDestination,

    #[error("Program is not upgradable")]
    NotUpgradable,

    #[error("Missing quote line in the program log")]
    MissingQuoteLine,
}
