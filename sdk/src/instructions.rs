use crate::state::get_account_metas_for_swap;
use anchor_client::anchor_lang::{
    prelude::AccountMeta,
    solana_program::{instruction::Instruction, pubkey::Pubkey, system_program, sysvar},
    InstructionData, ToAccountMetas,
};
use anchor_spl::{
    associated_token::{self, get_associated_token_address},
    token,
};
use gfx_ssl_v2_interface::{pool_registry::PoolRegistry, LiquidityAccount, OraclePriceHistory, Pair, SSLMathConfig, SSLMathParams, SSLPool, PoolRegistryConfig, EventEmitter};

pub fn create_pool_registry(admin: Pubkey, funder: Pubkey) -> Instruction {
    let data = gfx_ssl_v2_interface::instruction::CreatePoolRegistry.data();

    let pool_registry = PoolRegistry::address(admin);

    let accounts = gfx_ssl_v2_interface::accounts::CreatePoolRegistry {
        admin,
        funder,
        pool_registry,
        system_program: system_program::ID,
    }
    .to_account_metas(None);

    Instruction {
        program_id: gfx_ssl_v2_interface::ID,
        accounts,
        data,
    }
}

pub fn config_pool_registry(
    config: PoolRegistryConfig,
    admin: Pubkey,
    pool_registry: Pubkey,
) -> Instruction {
    let data = gfx_ssl_v2_interface::instruction::ConfigPoolRegistry {
        config
    }
        .data();

    let accounts = gfx_ssl_v2_interface::accounts::ConfigPoolRegistry {
        admin,
        pool_registry,
    }
        .to_account_metas(None);

    Instruction {
        program_id: gfx_ssl_v2_interface::ID,
        accounts,
        data,
    }
}
#[allow(clippy::too_many_arguments)]
pub fn create_ssl(
    initial_pool_deposit: u64,
    oracle_type: gfx_ssl_v2_interface::OracleType,
    asset_type: gfx_ssl_v2_interface::AssetType,
    math_params: SSLMathParams,
    admin: Pubkey,
    pool_registry: Pubkey,
    mint: Pubkey,
    oracle_account: Pubkey,
    number_of_slots_throttle: Option<u8>,
    max_slot_price_staleness: Option<u8>,
) -> Instruction {
    let data = gfx_ssl_v2_interface::instruction::CreateSsl {
        initial_pool_deposit,
        oracle_type: oracle_type.into(),
        asset_type: asset_type.into(),
        math_params,
        number_of_slots_throttle,
        max_slot_price_staleness,
    }
    .data();

    let ssl_pool_signer = SSLPool::signer_address(pool_registry, mint);
    let pool_vault = get_associated_token_address(&ssl_pool_signer, &mint);
    let pool_fee_vault = get_associated_token_address(&pool_registry, &mint);
    let admin_pool_mint_ata = get_associated_token_address(&admin, &mint);
    let oracle_price_history = OraclePriceHistory::address(&pool_registry, &oracle_account);

    let accounts = gfx_ssl_v2_interface::accounts::CreateSsl {
        admin,
        pool_registry,
        mint,
        ssl_pool_signer,
        pool_vault,
        pool_fee_vault,
        admin_pool_mint_ata,
        oracle_price_history,
        oracle_account,
        rent: sysvar::rent::ID,
        token_program: token::ID,
        associated_token_program: associated_token::ID,
        system_program: system_program::ID,
    }
    .to_account_metas(None);

    Instruction {
        program_id: gfx_ssl_v2_interface::ID,
        accounts,
        data,
    }
}

pub fn config_ssl(
    is_suspended: bool,
    math_params: SSLMathConfig,
    admin: Pubkey,
    pool_registry: Pubkey,
    mint: Pubkey,
) -> Instruction {
    let data = gfx_ssl_v2_interface::instruction::ConfigSsl {
        is_suspended,
        math_params,
    }
    .data();

    let accounts = gfx_ssl_v2_interface::accounts::ConfigSsl {
        admin,
        mint,
        pool_registry,
    }
    .to_account_metas(None);

    Instruction {
        program_id: gfx_ssl_v2_interface::ID,
        accounts,
        data,
    }
}

pub fn config_suspend_admin(
    admin: Pubkey,
    pool_registry: Pubkey,
    suspend_admin: Pubkey,
) -> Instruction {
    let data = gfx_ssl_v2_interface::instruction::ConfigSuspendAdmin.data();

    let accounts = gfx_ssl_v2_interface::accounts::ConfigSuspendAdmin {
        admin,
        suspend_admin,
        pool_registry,
    }
    .to_account_metas(None);

    Instruction {
        program_id: gfx_ssl_v2_interface::ID,
        accounts,
        data,
    }
}

pub fn config_price_history(
    admin: Pubkey,
    pool_registry: Pubkey,
    oracle: Pubkey,
    minimum_elapsed_slots: Option<u8>,
    max_slot_price_staleness: Option<u8>,
) -> Instruction {
    let data = gfx_ssl_v2_interface::instruction::ConfigPriceHistory {
        minimum_elapsed_slots,
        max_slot_price_staleness,
    }
    .data();

    let oracle_price_history = OraclePriceHistory::address(&pool_registry, &oracle);

    let accounts = gfx_ssl_v2_interface::accounts::ConfigPriceHistory {
        admin,
        pool_registry,
        oracle_price_history,
    }
    .to_account_metas(None);

    Instruction {
        program_id: gfx_ssl_v2_interface::ID,
        accounts,
        data,
    }
}

pub fn suspend_ssl(
    pool_registry: Pubkey,
    suspend_admin: Pubkey,
    mint: Pubkey,
    is_suspended: bool,
) -> Instruction {
    let data = gfx_ssl_v2_interface::instruction::SuspendSsl { is_suspended }.data();

    let accounts = gfx_ssl_v2_interface::accounts::SuspendSsl {
        suspend_admin,
        mint,
        pool_registry,
    }
    .to_account_metas(None);

    Instruction {
        program_id: gfx_ssl_v2_interface::ID,
        accounts,
        data,
    }
}

pub fn create_pair(
    mint_one_fee_rate: u16,
    mint_two_fee_rate: u16,
    admin: Pubkey,
    pool_registry: Pubkey,
    mint_one: Pubkey,
    mint_two: Pubkey,
    mint_one_fee_destination: Pubkey,
    mint_two_fee_destination: Pubkey,
) -> Instruction {
    let (
        mint_one,
        mint_two,
        mint_one_fee_destination,
        mint_two_fee_destination,
        mint_one_fee_rate,
        mint_two_fee_rate,
    ) = if Pair::normalize_mint_order(mint_one, mint_two) == (mint_one, mint_two) {
        (
            mint_one,
            mint_two,
            mint_one_fee_destination,
            mint_two_fee_destination,
            mint_one_fee_rate,
            mint_two_fee_rate,
        )
    } else {
        (
            mint_two,
            mint_one,
            mint_two_fee_destination,
            mint_one_fee_destination,
            mint_two_fee_rate,
            mint_one_fee_rate,
        )
    };

    let data = gfx_ssl_v2_interface::instruction::CreatePair {
        mint_one_fee_rate,
        mint_two_fee_rate,
    }
    .data();

    let pair = Pair::address(pool_registry, mint_one, mint_two);
    let ssl_pool_one_signer = SSLPool::signer_address(pool_registry, mint_one);
    let ssl_pool_two_signer = SSLPool::signer_address(pool_registry, mint_two);
    let mint_one_secondary_vault = get_associated_token_address(&ssl_pool_one_signer, &mint_two);
    let mint_two_secondary_vault = get_associated_token_address(&ssl_pool_two_signer, &mint_one);

    let accounts = gfx_ssl_v2_interface::accounts::CreatePair {
        pool_registry,
        mint_one,
        mint_two,
        mint_one_fee_destination,
        mint_two_fee_destination,
        pair,
        ssl_pool_one_signer,
        ssl_pool_two_signer,
        mint_one_secondary_vault,
        mint_two_secondary_vault,
        admin,
        rent: sysvar::rent::ID,
        token_program: token::ID,
        associated_token_program: associated_token::ID,
        system_program: system_program::ID,
    }
    .to_account_metas(None);

    Instruction {
        program_id: gfx_ssl_v2_interface::ID,
        accounts,
        data,
    }
}

pub fn config_pair(
    admin: Pubkey,
    pool_registry: Pubkey,
    mint_one: Pubkey,
    mint_two: Pubkey,
    mint_one_fee_rate: Option<u16>,
    mint_two_fee_rate: Option<u16>,
    mint_one_fee_destination: Option<Pubkey>,
    mint_two_fee_destination: Option<Pubkey>,
) -> Instruction {
    let (mint_one_fee_rate, mint_two_fee_rate, mint_one_fee_destination, mint_two_fee_destination) =
        if (mint_one, mint_two) == Pair::normalize_mint_order(mint_one, mint_two) {
            (
                mint_one_fee_rate,
                mint_two_fee_rate,
                mint_one_fee_destination,
                mint_two_fee_destination,
            )
        } else {
            (
                mint_two_fee_rate,
                mint_one_fee_rate,
                mint_two_fee_destination,
                mint_one_fee_destination,
            )
        };

    let data = gfx_ssl_v2_interface::instruction::ConfigPair {
        mint_one_fee_rate,
        mint_two_fee_rate,
    }
    .data();

    let pair = Pair::address(pool_registry, mint_one, mint_two);

    let accounts = gfx_ssl_v2_interface::accounts::ConfigPair {
        admin,
        pool_registry,
        pair,
        mint_one_fee_destination: mint_one_fee_destination.unwrap_or_default(),
        mint_two_fee_destination: mint_two_fee_destination.unwrap_or_default(),
    }
    .to_account_metas(None);

    Instruction {
        program_id: gfx_ssl_v2_interface::ID,
        accounts,
        data,
    }
}

pub fn crank_oracle_price_histories(
    pool_registry: Pubkey,
    registry_accounts: &Vec<crate::state::OracleAndPriceHistory>,
) -> Instruction {
    let data = gfx_ssl_v2_interface::instruction::CrankPriceHistories.data();

    let remaining_accounts: Vec<_> = registry_accounts
        .into_iter()
        .flat_map(|pair| {
            [
                AccountMeta::new(pair.price_history, false),
                AccountMeta::new(pair.oracle, false),
            ]
        })
        .collect();

    let mut accounts =
        gfx_ssl_v2_interface::accounts::CrankPriceHistories { pool_registry }.to_account_metas(None);

    accounts.extend(remaining_accounts);

    Instruction {
        program_id: gfx_ssl_v2_interface::ID,
        accounts,
        data,
    }
}

pub fn internal_swap(
    pool_registry: Pubkey,
    mint_one: Pubkey,
    mint_two: Pubkey,
    mint_one_oracle: Pubkey,
    mint_two_oracle: Pubkey,
) -> Instruction {
    let data = gfx_ssl_v2_interface::instruction::InternalSwap.data();

    // Normalize mints and oracles
    let (m1, m2) = Pair::normalize_mint_order(mint_one, mint_two);
    let (oracle1, oracle2) = if m1 != mint_one {
        (mint_two_oracle, mint_one_oracle)
    } else {
        (mint_one_oracle, mint_two_oracle)
    };

    let pair = Pair::address(pool_registry, m1, m2);
    let ssl_pool_a_signer = SSLPool::signer_address(pool_registry, m1);
    let ssl_pool_b_signer = SSLPool::signer_address(pool_registry, m2);
    let token_a_price_history = OraclePriceHistory::address(&pool_registry, &oracle1);
    let token_b_price_history = OraclePriceHistory::address(&pool_registry, &oracle2);
    let ssl_a_main_token = get_associated_token_address(&ssl_pool_a_signer, &m1);
    let ssl_b_main_token = get_associated_token_address(&ssl_pool_b_signer, &m2);
    let ssl_a_secondary_token = get_associated_token_address(&ssl_pool_a_signer, &m2);
    let ssl_b_secondary_token = get_associated_token_address(&ssl_pool_b_signer, &m1);

    let accounts = gfx_ssl_v2_interface::accounts::InternalSwap {
        pair,
        pool_registry,
        ssl_a_main_token,
        ssl_b_main_token,
        ssl_pool_a_signer,
        ssl_pool_b_signer,
        ssl_a_secondary_token,
        ssl_b_secondary_token,
        token_a_oracle: oracle1,
        token_b_oracle: oracle2,
        token_a_price_history,
        token_b_price_history,
        event_emitter: EventEmitter::address(),
        token_program: token::ID,
    }
    .to_account_metas(None);

    Instruction {
        program_id: gfx_ssl_v2_interface::ID,
        accounts,
        data,
    }
}

pub fn claim_fees(pool_registry: Pubkey, owner: Pubkey, mint: Pubkey) -> Instruction {
    let data = gfx_ssl_v2_interface::instruction::ClaimFees.data();

    let liquidity_account = LiquidityAccount::address(pool_registry, mint, owner);
    let owner_ata = get_associated_token_address(&owner, &mint);
    let ssl_fee_vault = get_associated_token_address(&pool_registry, &mint);

    let accounts = gfx_ssl_v2_interface::accounts::ClaimFees {
        owner,
        owner_ata,
        liquidity_account,
        pool_registry,
        ssl_fee_vault,
        event_emitter: EventEmitter::address(),
        token_program: token::ID,
    }
    .to_account_metas(None);

    Instruction {
        program_id: gfx_ssl_v2_interface::ID,
        accounts,
        data,
    }
}

pub fn create_liquidity_account(pool_registry: Pubkey, owner: Pubkey, mint: Pubkey) -> Instruction {
    let data = gfx_ssl_v2_interface::instruction::CreateLiquidityAccount.data();

    let liquidity_account = LiquidityAccount::address(pool_registry, mint, owner);

    let accounts = gfx_ssl_v2_interface::accounts::CreateLiquidityAccount {
        liquidity_account,
        mint,
        owner,
        pool_registry,
        event_emitter: EventEmitter::address(),
        system_program: system_program::ID,
    }
    .to_account_metas(None);

    Instruction {
        program_id: gfx_ssl_v2_interface::ID,
        accounts,
        data,
    }
}

pub fn close_liquidity_account(
    liquidity_account: Pubkey,
    owner: Pubkey,
    rent_recipient: Pubkey,
) -> Instruction {
    let data = gfx_ssl_v2_interface::instruction::CloseLiquidityAccount.data();

    let accounts = gfx_ssl_v2_interface::accounts::CloseLiquidityAccount {
        liquidity_account,
        owner,
        rent_recipient,
        event_emitter: EventEmitter::address(),
        system_program: system_program::ID,
    }
    .to_account_metas(None);

    Instruction {
        program_id: gfx_ssl_v2_interface::ID,
        accounts,
        data,
    }
}

pub fn deposit(pool_registry: Pubkey, owner: Pubkey, mint: Pubkey, amount: u64) -> Instruction {
    let data = gfx_ssl_v2_interface::instruction::Deposit { amount }.data();

    let liquidity_account = LiquidityAccount::address(pool_registry, mint, owner);
    let ssl_pool_signer = SSLPool::signer_address(pool_registry, mint);
    let pool_vault = SSLPool::vault_address(pool_registry, mint);
    let ssl_fee_vault = get_associated_token_address(&pool_registry, &mint);
    let user_ata = get_associated_token_address(&owner, &mint);

    let accounts = gfx_ssl_v2_interface::accounts::Deposit {
        liquidity_account,
        owner,
        pool_registry,
        user_ata,
        ssl_pool_signer,
        pool_vault,
        ssl_fee_vault,
        event_emitter: EventEmitter::address(),
        token_program: token::ID,
    }
    .to_account_metas(None);

    Instruction {
        program_id: gfx_ssl_v2_interface::ID,
        accounts,
        data,
    }
}

pub fn withdraw(pool_registry: Pubkey, owner: Pubkey, mint: Pubkey, amount: u64) -> Instruction {
    let data = gfx_ssl_v2_interface::instruction::Withdraw { amount }.data();

    let liquidity_account = LiquidityAccount::address(pool_registry, mint, owner);
    let ssl_pool_signer = SSLPool::signer_address(pool_registry, mint);
    let pool_vault = SSLPool::vault_address(pool_registry, mint);
    let ssl_fee_vault = get_associated_token_address(&pool_registry, &mint);
    let user_ata = get_associated_token_address(&owner, &mint);

    let accounts = gfx_ssl_v2_interface::accounts::Withdraw {
        liquidity_account,
        owner,
        pool_registry,
        user_ata,
        ssl_pool_signer,
        pool_vault,
        ssl_fee_vault,
        event_emitter: EventEmitter::address(),
        token_program: token::ID,
    }
    .to_account_metas(None);

    Instruction {
        program_id: gfx_ssl_v2_interface::ID,
        accounts,
        data,
    }
}

#[allow(clippy::too_many_arguments)]
pub fn swap(
    amount_in: u64,
    min_out: u64,
    pool_registry: Pubkey,
    user_wallet: Pubkey,
    mint_in: Pubkey,
    mint_out: Pubkey,
    input_token_oracle: Pubkey,
    output_token_oracle: Pubkey,
    fee_destination: Pubkey,
) -> Instruction {
    let data = gfx_ssl_v2_interface::instruction::Swap { amount_in, min_out }.data();

    Instruction {
        program_id: gfx_ssl_v2_interface::ID,
        accounts: get_account_metas_for_swap(
            pool_registry,
            user_wallet,
            mint_in,
            mint_out,
            input_token_oracle,
            output_token_oracle,
            fee_destination,
        ),
        data,
    }
}
