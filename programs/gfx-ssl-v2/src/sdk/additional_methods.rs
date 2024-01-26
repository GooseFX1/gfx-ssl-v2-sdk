use crate::{PDAIdentifier, Pair, PoolRegistry, SSLPool};
use anchor_lang::prelude::Pubkey;
use spl_associated_token_account::get_associated_token_address;
use std::collections::HashMap;

impl PoolRegistry {
    /// Calculate all the pair addresses for a pool registry, given its address.
    /// The `HashMap` is `(pair_address, (mint_a, mint_b)`, and the mint order is normalized.
    pub fn all_pairs(&self, pool_registry_address: Pubkey) -> HashMap<Pubkey, (Pubkey, Pubkey)> {
        let mints = (0..self.num_entries)
            .map(|index| {
                let pool = &self.entries[index as usize];
                pool.mint
            })
            .collect::<Vec<Pubkey>>();
        let mut pair_addresses = HashMap::new();
        mints.iter().for_each(|mint_a| {
            mints.iter().for_each(|mint_b| {
                if *mint_a != *mint_b {
                    let pair_address = Pair::address(pool_registry_address, *mint_a, *mint_b);
                    let (mint_a, mint_b) = Pair::normalize_mint_order(*mint_a, *mint_b);
                    if !pair_addresses.contains_key(&pair_address) {
                        pair_addresses.insert(pair_address, (mint_a, mint_b));
                    }
                }
            })
        });
        pair_addresses
    }

    /// Calculate all the secondary vault addresses
    /// for a given pool registry address and primary mint.
    pub fn secondary_vault_addresses(
        &self,
        pool_registry_address: Pubkey,
        primary_vault_mint: Pubkey,
    ) -> Vec<Pubkey> {
        self.entries()
            .filter_map(|pool| {
                if primary_vault_mint != pool.mint {
                    Some(SSLPool::secondary_token_vault_address(
                        pool_registry_address,
                        primary_vault_mint,
                        pool.mint,
                    ))
                } else {
                    None
                }
            })
            .collect()
    }

    pub fn fee_vault_address(pool_registry: &Pubkey, mint: &Pubkey) -> Pubkey {
        get_associated_token_address(pool_registry, mint)
    }
}

impl SSLPool {
    pub fn oracle_price_histories(&self) -> Vec<Pubkey> {
        self.oracle_price_histories
            .into_iter()
            .filter(|addr| *addr != Pubkey::default())
            .collect()
    }

    pub fn signer_address_with_bump(pool_registry: Pubkey, mint: Pubkey) -> (Pubkey, u8) {
        Self::get_address_with_bump(&[pool_registry.as_ref(), mint.as_ref()])
    }
}
