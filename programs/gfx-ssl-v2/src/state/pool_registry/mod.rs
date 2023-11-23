pub mod math_params;
pub mod ssl_pool;
pub mod token_ratio_category;

use crate::PDAIdentifier;
use anchor_lang::prelude::*;
use anchor_lang::Discriminator;
use bytemuck::Zeroable;
#[cfg(feature = "no-entrypoint")]
use std::fmt::{Display, Formatter};
use std::io::Write;

pub use crate::state::liquidity_account::LiquidityAccount;
use crate::SSLV2Error;
pub use math_params::{SSLMathConfig, SSLMathParams};
pub use ssl_pool::{AssetType, SSLPool, SSLPoolStatus};
use crate::token_ratio_category::MaxPoolTokenRatio;

/// We need to enforce a maximum number of pools per admin
/// because of Solana's on-chain compute limitations.
pub const MAX_SSL_POOLS_PER_ADMIN: usize = 32;

/// A global registry that stores an exhaustive list of all SSL pools owned by a specific admin.
///
/// All SSL pools owned by the same admin exist under the same swappable domain.
/// The admin therefore splits groups of SSL pools into separate "magesteria" of liquidity,
/// and an SSL pool owned by admin A isn't swappable with a pool owned by admin B. Admin A and
/// admin B would thus also have different [PoolRegistry] accounts, as well.
#[account(zero_copy)]
#[repr(C)]
#[derive(Debug)]
pub struct PoolRegistry {
    /// Only this signer can sign for mutations of accounts under a given pool registry.
    pub admin: Pubkey,
    pub seed: Pubkey,
    /// This address is controlled by the admin, and only has the authority to suspend pools.
    pub suspend_admin: Pubkey,
    pub bump: u8,
    pub _pad0: [u8; 7],
    pub num_entries: u32,
    pub _pad1: [u8; 4],
    pub categorical_pool_token_ratios: [u16; 16],
    pub _space: [u8; 96],
    /// A list of oracles whose mints are offered as SSL pools under the domain of a given
    /// admin.
    pub entries: [SSLPool; MAX_SSL_POOLS_PER_ADMIN],
}

const _: [u8; 9200] = [0u8; std::mem::size_of::<PoolRegistry>()];

// Manually implemented because you can't directly derive or impl
// Default for an array.
impl Default for PoolRegistry {
    fn default() -> Self {
        Self::zeroed()
    }
}

impl PDAIdentifier for PoolRegistry {
    const IDENT: &'static [u8] = b"pool_registry";

    #[inline(always)]
    fn program_id() -> &'static Pubkey {
        &crate::ID
    }
}

impl AccountSerialize for PoolRegistry {
    fn try_serialize<W: Write>(&self, writer: &mut W) -> Result<()> {
        let mut disc = Self::discriminator().to_vec();
        disc.append(&mut bytemuck::bytes_of(self).to_vec());
        writer.write_all(&disc)?;
        Ok(())
    }
}

#[cfg(feature = "no-entrypoint")]
impl Display for PoolRegistry {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Seed: {}", self.seed)?;
        writeln!(f, "Admin: {}", self.admin)?;
        writeln!(f, "Suspend admin: {}", self.suspend_admin)?;
        writeln!(f, "Number of entries {}", self.num_entries)?;

        for (idx, entry) in self
            .entries
            .iter()
            .filter(|pool| **pool != SSLPool::default())
            .enumerate()
        {
            writeln!(f, "SSL Pool: {}", idx)?;
            writeln!(f, "{}", entry)?;
            write!(
                f,
                "Main Token Vault: {}",
                SSLPool::vault_address(PoolRegistry::address(self.seed), entry.mint,)
            )?;
        }
        Ok(())
    }
}

impl PoolRegistry {
    /// Derives the address of the [PoolRegistry] PDA, given an admin address.
    ///
    /// Convenience method, more semantically descriptive than the similar function provided by the
    /// [PDAIdentifier] trait.
    pub fn address(seed: Pubkey) -> Pubkey {
        Self::get_address(&[seed.as_ref()])
    }

    /// This must be used inside the instruction body that initializes the [PoolRegistry]
    /// owned by a given admin.
    /// Since the Anchor account initialization will create the account completely zeroed out,
    /// we need a separate method to set any non-zero initial values.
    pub fn initialize(&mut self, admin: Pubkey, bump: u8) {
        self.admin = admin;
        self.seed = admin;
        self.suspend_admin = admin;
        self.bump = bump;
    }

    /// Fetch an immutable ref SSL pool from the array of pools.
    pub fn find_pool(&self, mint: Pubkey) -> Result<&SSLPool> {
        let pool = self.entries.iter().find(|entry| entry.mint == mint);
        if let Some(pool) = pool {
            return Ok(pool);
        }
        return err!(SSLV2Error::MintNotFound);
    }

    /// Fetch a mutable ref to an SSL pool from the array of pools.
    pub fn find_pool_mut(&mut self, mint: Pubkey) -> Result<&mut SSLPool> {
        let pool = self.entries.iter_mut().find(|entry| entry.mint == mint);
        if let Some(pool) = pool {
            return Ok(pool);
        }
        return err!(SSLV2Error::MintNotFound);
    }
}

/// For the `config_pool_registry` instruction
#[derive(Clone, Debug, Default, AnchorDeserialize, AnchorSerialize)]
#[repr(C)]
pub struct PoolRegistryConfig {
    pub new_admin: Option<Pubkey>,
    pub new_suspend_admin: Option<Pubkey>,
    pub max_pool_token_ratios: Vec<MaxPoolTokenRatio>,
}
