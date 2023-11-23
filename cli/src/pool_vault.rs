use anchor_lang::AccountDeserialize;
use anchor_spl::token::TokenAccount;
use serde::Serialize;
use solana_client::rpc_client::RpcClient;
use solana_sdk::pubkey::Pubkey;
use gfx_ssl_v2_interface::{PoolRegistry, SSLPool};
use crate::display::{mint_decimals, mint_ui_name, ui_amount};
use crate::pubkey_str::pubkey;

#[derive(Serialize, Clone, Copy)]
pub struct MainVault {
    #[serde(with = "pubkey")]
    pub address: Pubkey,
    #[serde(with = "pubkey")]
    pub mint: Pubkey,
    pub mint_decimals: u32,
    pub balance: u64,
}

impl MainVault {
    pub fn from_rpc_client(
        pool_registry_address: Pubkey,
        pool_registry: &PoolRegistry,
        mint: Pubkey,
        client: &RpcClient,
    ) -> anyhow::Result<Self> {
        let address = SSLPool::vault_address(pool_registry_address, mint);
        let pool = pool_registry.find_pool(mint)?;
        let mint_decimals = pool.mint_decimals as u32;
        let act = client.get_account_data(&address)?;
        let token_act =
            TokenAccount::try_deserialize(&mut &act[..]).unwrap();
        Ok(Self {
            mint,
            address,
            mint_decimals,
            balance: token_act.amount,
        })
    }
}

#[derive(Serialize, Clone)]
pub struct MainVaultUiData {
    #[serde(with = "pubkey")]
    pub address: Pubkey,
    #[serde(with = "pubkey")]
    pub mint: Pubkey,
    pub mint_name: Option<String>,
    pub balance: Option<String>,
}

impl From<&MainVault> for MainVaultUiData {
    fn from(value: &MainVault) -> Self {
        let mint_name = mint_ui_name(value.mint);
        let balance = ui_amount(
            value.balance,
            mint_decimals(value.mint)
        );
        Self {
            address: value.address,
            mint: value.mint,
            mint_name,
            balance,
        }
    }
}

#[derive(Serialize, Clone, Copy)]
pub struct SecondaryVault {
    #[serde(with = "pubkey")]
    pub main_token: Pubkey,
    #[serde(with = "pubkey")]
    pub mint: Pubkey,
    #[serde(with = "pubkey")]
    pub address: Pubkey,
    pub mint_decimals: u32,
    pub balance: u64,
}

impl SecondaryVault {
    pub fn from_rpc_client(
        pool_registry_address: Pubkey,
        pool_registry: &PoolRegistry,
        primary_mint: Pubkey,
        secondary_mint: Pubkey,
        client: &RpcClient,
    ) -> anyhow::Result<Self> {
        let address = SSLPool::secondary_token_vault_address(
            pool_registry_address,
            primary_mint,
            secondary_mint,
        );
        let pool = pool_registry.find_pool(secondary_mint)?;
        let mint_decimals = pool.mint_decimals as u32;
        let act = client.get_account_data(&address)?;
        let token_act =
            TokenAccount::try_deserialize(&mut &act[..]).unwrap();
        Ok(Self {
            main_token: primary_mint,
            mint: secondary_mint,
            address,
            mint_decimals,
            balance: token_act.amount,
        })
    }
}

#[derive(Serialize, Clone)]
pub struct SecondaryVaultUiData {
    #[serde(with = "pubkey")]
    main_token: Pubkey,
    #[serde(with = "pubkey")]
    address: Pubkey,
    #[serde(with = "pubkey")]
    mint: Pubkey,
    mint_name: Option<String>,
    balance: Option<String>,
}

impl From<&SecondaryVault> for SecondaryVaultUiData {
    fn from(value: &SecondaryVault) -> Self {
        let mint_name = mint_ui_name(value.mint);
        let balance = ui_amount(
            value.balance,
            mint_decimals(value.mint)
        );
        Self {
            main_token: value.main_token,
            address: value.address,
            mint: value.mint,
            mint_name,
            balance,
        }
    }
}
