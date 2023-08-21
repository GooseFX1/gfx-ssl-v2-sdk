use anchor_lang::prelude::*;
use std::convert::TryInto;

const ERROR_CODE_OFFSET: u32 = 6100;

// NOTE: We cannot use ERROR_CODE_OFFSET because it expects an integer literal
/// Custom error code: 6100 + idx => 0x17D4 + 0x${idx}
#[error_code(offset = 6100)]
#[derive(PartialEq)]
pub enum SSLV2Error {
    #[msg("[G100] The pool is suspended")] //0x17D4 (6100)
    Suspended,

    #[msg("[G101] Not admin")] //0x17D5 (6101)
    NotAdmin,

    #[msg("[G102] Mints are not sorted")] //0x17D6 (6102)
    MintsNotSorted,

    #[msg("[G103] The price history is empty")] //0x17D7 (6103)
    PriceHistoryEmpty,

    #[msg("[G104] Oracle accounts and price histories must be present in the order listed on the pool registry")]
    //0x17D8 (6104)
    InvalidCrankAccounts,

    #[msg("[G105] The oracle is not in a healthy state (status)")] //0x17D9 (6105)
    OracleNotHealthyStatus,

    #[msg("[G106] The oracle is not in a healthy state (delay)")] //0x17DA (6106)
    OracleNotHealthyDelay,

    #[msg("[G107] The oracle is not in a healthy state (confidence)")] //0x17DB (6107)
    OracleNotHealthyConfidence,

    #[msg("[G108] SlippageTooLarge")] //0x17DC (6108)
    SlippageTooLarge,

    #[msg("[G109] Percentage out of range")] //0x17DD (6109)
    PercentageOutOfRange,

    #[msg("[G110] The Ema window is too large for the amount of available price history data")]
    //0x17DE (6110)
    EmaOrStdWindowTooLarge,

    #[msg("[G111] Mint does not match the pair")] //0x17DF (6111)
    MintNotMatchPair,

    #[msg("[G112] Fee collector account incorrect")] //0x17E0 (6112)
    FeeCollectorIncorrect,

    #[msg("[G113] The SSL pool is stale")] //0x17E1 (6113)
    SSLStale,

    #[msg("[G114] The Pool Registry is full, no more SSL Pools can be added")] //0x17E2 (6114)
    PoolRegistryIsFull,

    #[msg("[G115] Cannot find an SSL Pool with the provided mint")] //0x17E3 (6115)
    MintNotFound,

    #[msg("[G116] An SSL Pool for that mint is already included")] //0x17E4 (6116)
    MintAlreadyIncluded,

    #[msg("[G117] Invalid Pyth Oracle Price Account")] //0x17E5 (6117)
    InvalidPythOracle,

    #[msg("[G118] Token account not owned by the correct registry")] //0x17E6 (6118)
    InvalidTokenOwner,

    #[msg("[G119] Token account does not have the expected mint")] //0x17E7 (6119)
    InvalidTokenMint,

    #[msg("[G120] The price history is too stale, requires more recent updates")] //0x17E8 (6120)
    StalePriceHistory,

    #[msg("[G121] Cannot initialize or use an oracle with the given type")] //0x17E9 (6121)
    InvalidOracleType,

    #[msg("[G122] Math Error")] //0x17EA (6122)
    MathError,

    #[msg("[G123] Pair mints cannot be equivalent")] //0x17EB (6123)
    MintsCannotBeSame,

    #[msg("[G124] Pair fee Destination does not match the target mint")] //0x17EC (6124)
    InvalidFeeDestination,

    #[msg("[G125] Liquidity account must be fully withdrawn before closing")] //0x17ED (6125)
    CannotCloseLiquidityAccount,

    #[msg("[G126] Liquidity withdraw is too large compared to the amount the user has deposited")]
    //0x17EE (6126)
    WithdrawTooLarge,

    #[msg("[G127] Oracle address does not match was is stored on the price history account")]
    //0x17EF (6127)
    InvalidOracleAddress,

    #[msg("[G128] The swap would make the pool token imbalance exceed configured maximum")]
    //0x17F0 (6128)
    PoolTokenImbalance,

    #[msg("[G129] Pair does not belong to the Pool Registry")] //0x17F1 (6129)
    PoolRegistryNotMatchPair,

    #[msg("[G130] Initial deposit to a newly created SSL Pool cannot be zero")] //0x17F2 (6130)
    ZeroInitialDeposit,

    #[msg("[G131] Oracle price already recorded")] //0x17F3 (6131)
    OraclePriceRecorded,

    #[msg("[G132] Swap amount too small, fees collected would be zero")] //0x17F4 (6132)
    AmountTooSmall,

    #[msg("[G133] Swap amount too large, not enough liquidity in the output token")]
    //0x17F5 (6133)
    NotEnoughLiquidity,

    #[msg("[G134] Provided account is not a token account or does not exist")] //0x17F6 (6134)
    NotATokenAccount,

    #[msg("[G135] The oracle is being throttled")] //0x17F7 (6135)
    OracleThrottled,

    #[msg("[G136] Provided parameter can't be 0.")] //0x17F8 (6136)
    NotZeroParameter,

    #[msg("[G137] Oracle price history does not belong to the Pool Registry")] //0x17F9 (6137)
    PoolRegistryNotMatchOraclePriceHistory,
}

pub const NUM_ERR_VARIANTS: u32 = 38;

impl TryInto<SSLV2Error> for u32 {
    // If the u32 is not within the bounds of [ERROR_CODE_OFFSET] and
    // [ERROR_CODE_OFFSET + NUM_ERR_VARIANTS, this error is returned.
    type Error = ();

    fn try_into(self) -> std::result::Result<SSLV2Error, ()> {
        if (ERROR_CODE_OFFSET..=ERROR_CODE_OFFSET + NUM_ERR_VARIANTS).contains(&self) {
            Ok(unsafe { std::mem::transmute(self - ERROR_CODE_OFFSET) })
        } else {
            Err(())
        }
    }
}
