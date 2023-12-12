pub mod contexts;
pub mod errors;
pub mod state;
pub mod utils;

use anchor_lang::prelude::*;

declare_id!("GFXsSL5sSaDfNFQUYsHekbWBW1TsFdjDYzACh62tEHxn");

pub use crate::utils::PDAIdentifier;
use contexts::*;
pub use errors::*;
pub use state::*;

#[allow(unused_variables)]
#[program]
pub mod gfx_ssl_v2 {
    use super::*;

    pub fn create_pool_registry(ctx: Context<CreatePoolRegistry>) -> Result<()> {
        Ok(())
    }
    pub fn create_event_emitter(ctx: Context<CreateEventEmitter>) -> Result<()> {
        Ok(())
    }

    pub fn config_pool_registry(
        ctx: Context<ConfigPoolRegistry>,
        config: PoolRegistryConfig,
    ) -> Result<()> {
        Ok(())
    }

    pub fn create_ssl(
        ctx: Context<CreateSsl>,
        initial_pool_deposit: u64,
        oracle_type: u8,
        asset_type: u8,
        math_params: SSLMathParams,
        number_of_slots_throttle: Option<u8>,
        max_slot_price_staleness: Option<u8>,
    ) -> Result<()> {
        Ok(())
    }

    pub fn config_ssl(
        ctx: Context<ConfigSsl>,
        is_suspended: bool,
        math_params: SSLMathConfig,
    ) -> Result<()> {
        Ok(())
    }

    pub fn suspend_ssl(ctx: Context<SuspendSsl>, is_suspended: bool) -> Result<()> {
        Ok(())
    }

    pub fn config_suspend_admin(ctx: Context<ConfigSuspendAdmin>) -> Result<()> {
        Ok(())
    }

    pub fn create_pair(
        ctx: Context<CreatePair>,
        mint_one_fee_rate: u16,
        mint_two_fee_rate: u16,
    ) -> Result<()> {
        Ok(())
    }

    pub fn config_pair(
        ctx: Context<ConfigPair>,
        mint_one_fee_rate: Option<u16>,
        mint_two_fee_rate: Option<u16>,
    ) -> Result<()> {
        Ok(())
    }

    pub fn config_price_history(
        ctx: Context<ConfigPriceHistory>,
        minimum_elapsed_slots: Option<u8>,
        max_slot_price_staleness: Option<u8>,
    ) -> Result<()> {
        Ok(())
    }

    pub fn crank_price_histories(ctx: Context<CrankPriceHistories>) -> Result<()> {
        Ok(())
    }

    pub fn internal_swap(ctx: Context<InternalSwap>) -> Result<()> {
        Ok(())
    }

    pub fn claim_fees(ctx: Context<ClaimFees>) -> Result<()> {
        Ok(())
    }

    pub fn create_liquidity_account(ctx: Context<CreateLiquidityAccount>) -> Result<()> {
        Ok(())
    }

    pub fn close_liquidity_account(ctx: Context<CloseLiquidityAccount>) -> Result<()> {
        Ok(())
    }

    pub fn deposit(ctx: Context<Deposit>, amount: u64) -> Result<()> {
        Ok(())
    }

    pub fn withdraw(ctx: Context<Withdraw>, amount: u64) -> Result<()> {
        Ok(())
    }

    pub fn swap(ctx: Context<Swap>, amount_in: u64, min_out: u64) -> Result<()> {
        Ok(())
    }

    pub fn quote(ctx: Context<Quote>, amount_in: u64) -> Result<()> {
        Ok(())
    }
}
