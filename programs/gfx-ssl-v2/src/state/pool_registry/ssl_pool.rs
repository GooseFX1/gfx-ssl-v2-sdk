use crate::state::pool_registry::math_params::SSLMathParams;
#[cfg(feature = "no-entrypoint")]
use crate::utils::token_amount;
use crate::PDAIdentifier;
use anchor_lang::prelude::Pubkey;
use anchor_lang::prelude::*;
use anchor_spl::associated_token::get_associated_token_address;
use bytemuck::Zeroable;
#[cfg(feature = "no-entrypoint")]
use std::fmt;
#[cfg(feature = "no-entrypoint")]
use std::fmt::{Display, Formatter};

/// For any given mint, we may want to utilize more than one oracle for price derivation.
/// More than three oracles seems gratuitous, due to constraints in
/// account size and compute budget.
pub const MAX_NUM_ORACLES_PER_MINT: usize = 3;

#[derive(Copy, Clone, Debug, Default, PartialEq, AnchorSerialize, AnchorDeserialize)]
#[repr(C)]
pub enum SSLPoolStatus {
    /// Indicates that a given [SSLPool] entry in the [PoolRegistry] is blank, i.e. all zeroes.
    #[default]
    Uninitialized,
    /// Initialized and swaps enabled.
    Active,
    /// Swaps disabled, and liquidity held by this pool not counted towards
    /// system-wide calculations.
    Suspended,
    Invalid,
}

#[cfg(feature = "no-entrypoint")]
impl Display for SSLPoolStatus {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            SSLPoolStatus::Uninitialized => write!(f, "Uninitialized"),
            SSLPoolStatus::Active => write!(f, "Active"),
            SSLPoolStatus::Suspended => write!(f, "Suspended"),
            SSLPoolStatus::Invalid => write!(f, "Invalid"),
        }
    }
}

impl From<u8> for SSLPoolStatus {
    fn from(value: u8) -> Self {
        match value {
            0 => SSLPoolStatus::Uninitialized,
            1 => SSLPoolStatus::Active,
            2 => SSLPoolStatus::Suspended,
            _ => SSLPoolStatus::Invalid,
        }
    }
}

impl Into<u8> for SSLPoolStatus {
    fn into(self) -> u8 {
        match self {
            SSLPoolStatus::Uninitialized => 0,
            SSLPoolStatus::Active => 1,
            SSLPoolStatus::Suspended => 2,
            SSLPoolStatus::Invalid => u8::MAX,
        }
    }
}

/// Classifies assets into various categories.
/// These values are inert in the first release of the SSLv2 protocol.
#[derive(Copy, Clone, Debug, Default, PartialEq, AnchorSerialize, AnchorDeserialize)]
#[repr(C)]
pub enum AssetType {
    /// Indicates that a given [SSLPool] entry in the [PoolRegistry] is blank, i.e. all zeroes.
    #[default]
    Uninitialized,
    /// Assets like BTC, ETH, and stablecoins
    BlueChip,
    /// Meme tokens, other assets that swing wildly in price
    /// and which are likely to eventually "die out" in trade volume.
    Volatile,
    Stable,
    Invalid,
}

#[cfg(feature = "no-entrypoint")]
impl Display for AssetType {
    fn fmt(&self, f: &mut Formatter<'_>) -> fmt::Result {
        match *self {
            AssetType::Uninitialized => write!(f, "Uninitialized"),
            AssetType::BlueChip => write!(f, "BlueChip"),
            AssetType::Volatile => write!(f, "Volatile"),
            AssetType::Stable => write!(f, "Stable"),
            AssetType::Invalid => write!(f, "Invalid"),
        }
    }
}

impl From<u8> for AssetType {
    fn from(value: u8) -> Self {
        match value {
            0 => AssetType::Uninitialized,
            1 => AssetType::BlueChip,
            2 => AssetType::Volatile,
            3 => AssetType::Stable,
            _ => AssetType::Invalid,
        }
    }
}

impl Into<u8> for AssetType {
    fn into(self) -> u8 {
        match self {
            AssetType::Uninitialized => 0,
            AssetType::BlueChip => 1,
            AssetType::Volatile => 2,
            AssetType::Stable => 3,
            AssetType::Invalid => u8::MAX,
        }
    }
}

/// A single-sided liquidity pool's metadata, recorded as an entry on a [PoolRegistry].
/// Also acts as uninitialized PDA signer and owner of the LP vaults for each SSL pool.
#[account(zero_copy)]
#[derive(Debug, PartialEq, AnchorSerialize, AnchorDeserialize)]
#[repr(C)]
pub struct SSLPool {
    /// This tells us whether this particular SSL Pool entry has data,
    /// and if so, whether or not the pool is suspended.
    pub status: u8,
    pub asset_type: u8,

    pub _pad0: [u8; 6],

    /// The main token mint for this SSL pool.
    pub mint: Pubkey,

    /// a copy from the mint
    pub mint_decimals: u8,

    /// Stored for CPI signatures
    pub bump: u8,
    pub _pad1: [u8; 6],

    /// Total historical record of fees accrued to LPs.
    /// This is not only for informational purposes, but also
    /// is used in bookkeeping LP rewards.
    pub total_accumulated_lp_reward: u64,

    /// Since swaps will alter the main token balance,
    /// we need to keep track separately from the token account's balance
    /// to correctly calculate LP fee claims.
    pub total_liquidity_deposits: u64,

    /// The price history accounts that record oracle prices.
    /// In principle, we allow for storage of multiple oracles, although these extra
    /// indices are not yet used in the program logic.
    pub oracle_price_histories: [Pubkey; MAX_NUM_ORACLES_PER_MINT],

    /// All parameters related to the mathematical calculations required for price
    /// determination and swap rule enforcement. "Swap rules" are conditions that
    /// must not be violated, otherwise an attempted swap should fail.
    pub math_params: SSLMathParams,
    pub _space: [u8; 64],
}

impl Default for SSLPool {
    fn default() -> Self {
        Self::zeroed()
    }
}

// Compile-time struct size check. Successful deserialization requires
// that the compiler's target architecture agrees with what's on-chain.
const _: [u8; std::mem::size_of::<SSLPool>()] = [0; 280];

#[cfg(feature = "no-entrypoint")]
impl Display for SSLPool {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Mint: {}", self.mint)?;
        writeln!(f, "Mint decimals: {}", self.mint_decimals)?;
        writeln!(f, "Asset type: {}", AssetType::from(self.asset_type))?;
        writeln!(f, "Pool status: {}", SSLPoolStatus::from(self.status))?;
        let lp_rewards =
            token_amount::to_ui(self.total_accumulated_lp_reward, self.mint_decimals as u32);
        writeln!(f, "Total accumulated LP rewards: {}", lp_rewards)?;
        let total_liq_deposits =
            token_amount::to_ui(self.total_liquidity_deposits, self.mint_decimals as u32);
        writeln!(f, "Total liquidity deposits: {}", total_liq_deposits)?;
        for (idx, entry) in self.oracle_price_histories.iter().enumerate() {
            writeln!(f, "Price History {}: {}", idx, entry)?;
        }
        write!(f, "Math params: {}", self.math_params)?;

        Ok(())
    }
}

impl SSLPool {
    /// Derive the address for a given LP vault's owner
    pub fn signer_address(pool_registry: Pubkey, mint: Pubkey) -> Pubkey {
        Self::get_address(&[pool_registry.as_ref(), mint.as_ref()])
    }

    /// Derive the address for a given mint's LP vault
    pub fn vault_address(pool_registry: Pubkey, mint: Pubkey) -> Pubkey {
        get_associated_token_address(&Self::signer_address(pool_registry, mint), &mint)
    }

    pub fn secondary_token_vault_address(
        pool_registry: Pubkey,
        primary_mint: Pubkey,
        secondary_mint: Pubkey,
    ) -> Pubkey {
        get_associated_token_address(
            &Self::signer_address(pool_registry, primary_mint),
            &secondary_mint,
        )
    }

    pub fn new(
        status: SSLPoolStatus,
        asset_type: AssetType,
        mint: Pubkey,
        mint_decimals: u8,
        bump: u8,
        first_oracle_history_account: Pubkey,
        initial_deposit: u64,
        math_params: SSLMathParams,
    ) -> Self {
        Self {
            status: status.into(),
            asset_type: asset_type.into(),
            _pad0: [0; 6],
            mint,
            mint_decimals,
            bump,
            _pad1: [0; 6],
            total_accumulated_lp_reward: 0,
            total_liquidity_deposits: initial_deposit,
            oracle_price_histories: [
                first_oracle_history_account,
                Default::default(),
                Default::default(),
            ],
            _space: [0; 64],
            math_params,
        }
    }

    /// Whether the pool is suspended. Suspended pools do not count toward
    /// the system imbalance calculations.
    pub fn is_suspended(&self) -> bool {
        self.status() == SSLPoolStatus::Suspended
    }

    pub fn status(&self) -> SSLPoolStatus {
        SSLPoolStatus::from(self.status)
    }

    pub fn asset_type(&self) -> AssetType {
        AssetType::from(self.asset_type)
    }

    /// This method is used during iteration over the [PoolRegistry]'s entries,
    /// to determine whether we can simply skip over a given entry.
    pub fn is_initialized(&self) -> bool {
        SSLPoolStatus::from(self.status) != SSLPoolStatus::Uninitialized
    }
}

impl PDAIdentifier for SSLPool {
    const IDENT: &'static [u8] = b"ssl_pool";

    #[inline(always)]
    fn program_id() -> &'static Pubkey {
        &crate::ID
    }
}
