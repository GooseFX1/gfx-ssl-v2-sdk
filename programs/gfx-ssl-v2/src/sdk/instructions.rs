use crate::{
    AssetType, ConfigPairParams, EventEmitter, LiquidityAccount, OraclePriceHistory, OracleType,
    Pair, PoolRegistry, PoolRegistryConfig, SSLMathConfig, SSLMathParams, SSLPool,
};
use anchor_lang::{
    prelude::AccountMeta,
    solana_program::{instruction::Instruction, pubkey::Pubkey, system_program, sysvar},
    InstructionData, ToAccountMetas,
};
use anchor_spl::{
    associated_token::{self, get_associated_token_address},
    token,
    token::spl_token,
};
use spl_associated_token_account::instruction::create_associated_token_account;

// TODO Split out functions to create the account meta objects from `crate::accounts`
//    That way we can overwrite some values for testing.

pub fn create_pool_registry(admin: Pubkey, funder: Pubkey) -> Instruction {
    let data = crate::instruction::CreatePoolRegistry.data();

    let pool_registry = PoolRegistry::address(admin);

    let accounts = crate::accounts::CreatePoolRegistry {
        admin,
        funder,
        pool_registry,
        system_program: system_program::ID,
    }
    .to_account_metas(None);

    Instruction {
        program_id: crate::ID,
        accounts,
        data,
    }
}

pub fn create_event_emitter(funder: Pubkey) -> Instruction {
    let data = crate::instruction::CreateEventEmitter.data();

    let accounts = crate::accounts::CreateEventEmitter {
        funder,
        event_emitter: EventEmitter::address(),
        system_program: system_program::ID,
    }
    .to_account_metas(None);

    Instruction {
        program_id: crate::ID,
        accounts,
        data,
    }
}

pub fn config_pool_registry(
    config: PoolRegistryConfig,
    admin: Pubkey,
    pool_registry: Pubkey,
) -> Instruction {
    let data = crate::instruction::ConfigPoolRegistry { config }.data();

    let accounts = crate::accounts::ConfigPoolRegistry {
        admin,
        pool_registry,
    }
    .to_account_metas(None);

    Instruction {
        program_id: crate::ID,
        accounts,
        data,
    }
}

pub fn create_ssl_pool_token_accounts(
    payer: Pubkey,
    pool_registry: Pubkey,
    mint: Pubkey,
) -> (Instruction, Instruction) {
    let ssl_pool_signer = SSLPool::signer_address(pool_registry, mint);
    let create_pool_vault =
        create_associated_token_account(&payer, &ssl_pool_signer, &mint, &spl_token::ID);
    let create_fee_vault =
        create_associated_token_account(&payer, &pool_registry, &mint, &spl_token::ID);
    (create_pool_vault, create_fee_vault)
}

#[allow(clippy::too_many_arguments)]
pub fn create_ssl(
    initial_pool_deposit: u64,
    oracle_type: OracleType,
    asset_type: AssetType,
    math_params: SSLMathParams,
    admin: Pubkey,
    pool_registry: Pubkey,
    mint: Pubkey,
    oracle_account: Pubkey,
    number_of_slots_throttle: Option<u8>,
    max_slot_price_staleness: Option<u8>,
) -> Instruction {
    let data = crate::instruction::CreateSsl {
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

    let accounts = crate::accounts::CreateSsl {
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
        program_id: crate::ID,
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
    let data = crate::instruction::ConfigSsl {
        is_suspended,
        math_params,
    }
    .data();

    let accounts = crate::accounts::ConfigSsl {
        admin,
        mint,
        pool_registry,
    }
    .to_account_metas(None);

    Instruction {
        program_id: crate::ID,
        accounts,
        data,
    }
}

pub fn config_suspend_admin(
    admin: Pubkey,
    pool_registry: Pubkey,
    suspend_admin: Pubkey,
) -> Instruction {
    let data = crate::instruction::ConfigSuspendAdmin.data();

    let accounts = crate::accounts::ConfigSuspendAdmin {
        admin,
        suspend_admin,
        pool_registry,
    }
    .to_account_metas(None);

    Instruction {
        program_id: crate::ID,
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
    backup_oracle: Option<(Pubkey, OracleType)>,
    backup_oracle2: Option<(Pubkey, OracleType)>,
) -> Instruction {
    let data = crate::instruction::ConfigPriceHistory {
        minimum_elapsed_slots,
        max_slot_price_staleness,
        backup_oracle: backup_oracle.map(|b| (b.0, b.1.into())),
        backup_oracle2: backup_oracle2.map(|b| (b.0, b.1.into())),
    }
    .data();

    let oracle_price_history = OraclePriceHistory::address(&pool_registry, &oracle);

    let accounts = crate::accounts::ConfigPriceHistory {
        admin,
        pool_registry,
        oracle_price_history,
    }
    .to_account_metas(None);

    Instruction {
        program_id: crate::ID,
        accounts,
        data,
    }
}

pub fn suspend_ssl(pool_registry: Pubkey, suspend_admin: Pubkey, mint: Pubkey) -> Instruction {
    let data = crate::instruction::SuspendSsl.data();

    let accounts = crate::accounts::SuspendSsl {
        suspend_admin,
        mint,
        pool_registry,
    }
    .to_account_metas(None);

    Instruction {
        program_id: crate::ID,
        accounts,
        data,
    }
}

pub fn create_pair_token_accounts(
    payer: Pubkey,
    pool_registry: Pubkey,
    mint_one: Pubkey,
    mint_two: Pubkey,
) -> (Instruction, Instruction) {
    let (mint_one, mint_two) = Pair::normalize_mint_order(mint_one, mint_two);

    let ssl_pool_one_signer = SSLPool::signer_address(pool_registry, mint_one);
    let ssl_pool_two_signer = SSLPool::signer_address(pool_registry, mint_two);

    let create_mint_one_secondary_vault =
        create_associated_token_account(&payer, &ssl_pool_one_signer, &mint_two, &spl_token::ID);
    let create_mint_two_secondary_vault =
        create_associated_token_account(&payer, &ssl_pool_two_signer, &mint_one, &spl_token::ID);

    (
        create_mint_one_secondary_vault,
        create_mint_two_secondary_vault,
    )
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

    let data = crate::instruction::CreatePair {
        mint_one_fee_rate,
        mint_two_fee_rate,
    }
    .data();

    let pair = Pair::address(pool_registry, mint_one, mint_two);
    let ssl_pool_one_signer = SSLPool::signer_address(pool_registry, mint_one);
    let ssl_pool_two_signer = SSLPool::signer_address(pool_registry, mint_two);
    let mint_one_secondary_vault = get_associated_token_address(&ssl_pool_one_signer, &mint_two);
    let mint_two_secondary_vault = get_associated_token_address(&ssl_pool_two_signer, &mint_one);

    let accounts = crate::accounts::CreatePair {
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
        program_id: crate::ID,
        accounts,
        data,
    }
}

/// This instruction corrects the order of `mint_one` and `mint_two` and associated
/// parameters to match the order imposed on the `Pair` struct.
pub fn config_pair(
    admin: Pubkey,
    pool_registry: Pubkey,
    mint_one: Pubkey,
    mint_two: Pubkey,
    params: ConfigPairParams,
    mint_one_fee_destination: Option<Pubkey>,
    mint_two_fee_destination: Option<Pubkey>,
) -> Instruction {
    let (
        mint_one_normal_route_fee_rate,
        mint_two_normal_route_fee_rate,
        mint_one_preferred_route_fee_rate,
        mint_two_preferred_route_fee_rate,
        mint_one_fee_destination,
        mint_two_fee_destination,
    ) = if (mint_one, mint_two) == Pair::normalize_mint_order(mint_one, mint_two) {
        let ConfigPairParams {
            mint_one_normal_route_fee_rate,
            mint_two_normal_route_fee_rate,
            mint_one_preferred_route_fee_rate,
            mint_two_preferred_route_fee_rate,
        } = params;
        (
            mint_one_normal_route_fee_rate,
            mint_two_normal_route_fee_rate,
            mint_one_preferred_route_fee_rate,
            mint_two_preferred_route_fee_rate,
            mint_one_fee_destination,
            mint_two_fee_destination,
        )
    } else {
        let ConfigPairParams {
            mint_one_normal_route_fee_rate,
            mint_two_normal_route_fee_rate,
            mint_one_preferred_route_fee_rate,
            mint_two_preferred_route_fee_rate,
        } = params;
        (
            mint_two_normal_route_fee_rate,
            mint_one_normal_route_fee_rate,
            mint_two_preferred_route_fee_rate,
            mint_one_preferred_route_fee_rate,
            mint_two_fee_destination,
            mint_one_fee_destination,
        )
    };

    let data = crate::instruction::ConfigPair {
        params: ConfigPairParams {
            mint_one_normal_route_fee_rate,
            mint_two_normal_route_fee_rate,
            mint_one_preferred_route_fee_rate,
            mint_two_preferred_route_fee_rate,
        },
    }
    .data();

    let pair = Pair::address(pool_registry, mint_one, mint_two);

    let accounts = crate::accounts::ConfigPair {
        admin,
        pool_registry,
        pair,
        mint_one_fee_destination: mint_one_fee_destination.unwrap_or_default(),
        mint_two_fee_destination: mint_two_fee_destination.unwrap_or_default(),
    }
    .to_account_metas(None);

    Instruction {
        program_id: crate::ID,
        accounts,
        data,
    }
}

pub fn crank_oracle_price_histories_account_metas(
    pool_registry: Pubkey,
) -> crate::accounts::CrankPriceHistories {
    crate::accounts::CrankPriceHistories { pool_registry }
}

pub fn crank_oracle_price_histories(
    pool_registry: Pubkey,
    oracle_price_histories: &[(Pubkey, OraclePriceHistory)],
) -> Instruction {
    let data = crate::instruction::CrankPriceHistories.data();

    let mut accounts =
        crank_oracle_price_histories_account_metas(pool_registry).to_account_metas(None);
    for (price_history_addr, price_history) in oracle_price_histories {
        accounts.push(AccountMeta::new(*price_history_addr, false));
        accounts.push(AccountMeta::new_readonly(
            price_history.oracle_address,
            false,
        ));
        if price_history.backup_oracle != Pubkey::default() {
            accounts.push(AccountMeta::new_readonly(
                price_history.backup_oracle,
                false,
            ));
        }
        if price_history.backup_oracle2 != Pubkey::default() {
            accounts.push(AccountMeta::new_readonly(
                price_history.backup_oracle2,
                false,
            ));
        }
    }

    Instruction {
        program_id: crate::ID,
        accounts,
        data,
    }
}

pub fn internal_swap_account_metas(
    pool_registry: Pubkey,
    mint_one: Pubkey,
    mint_two: Pubkey,
    mint_one_oracle: Pubkey,
    mint_two_oracle: Pubkey,
) -> crate::accounts::InternalSwap {
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
    crate::accounts::InternalSwap {
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
}

pub fn internal_swap(
    pool_registry: Pubkey,
    mint_one: Pubkey,
    mint_two: Pubkey,
    mint_one_oracle: Pubkey,
    mint_two_oracle: Pubkey,
) -> Instruction {
    let data = crate::instruction::InternalSwap.data();

    let accounts = internal_swap_account_metas(
        pool_registry,
        mint_one,
        mint_two,
        mint_one_oracle,
        mint_two_oracle,
    )
    .to_account_metas(None);

    Instruction {
        program_id: crate::ID,
        accounts,
        data,
    }
}

pub fn claim_fees(pool_registry: Pubkey, owner: Pubkey, mint: Pubkey) -> Instruction {
    let data = crate::instruction::ClaimFees.data();

    let liquidity_account = LiquidityAccount::address(pool_registry, mint, owner);
    let owner_ata = get_associated_token_address(&owner, &mint);
    let ssl_fee_vault = get_associated_token_address(&pool_registry, &mint);

    let accounts = crate::accounts::ClaimFees {
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
        program_id: crate::ID,
        accounts,
        data,
    }
}

pub fn create_liquidity_account(pool_registry: Pubkey, owner: Pubkey, mint: Pubkey) -> Instruction {
    let data = crate::instruction::CreateLiquidityAccount.data();

    let liquidity_account = LiquidityAccount::address(pool_registry, mint, owner);

    let accounts = crate::accounts::CreateLiquidityAccount {
        liquidity_account,
        mint,
        owner,
        pool_registry,
        event_emitter: EventEmitter::address(),
        system_program: system_program::ID,
    }
    .to_account_metas(None);

    Instruction {
        program_id: crate::ID,
        accounts,
        data,
    }
}

pub fn close_liquidity_account(
    liquidity_account: Pubkey,
    owner: Pubkey,
    rent_recipient: Pubkey,
) -> Instruction {
    let data = crate::instruction::CloseLiquidityAccount.data();

    let accounts = crate::accounts::CloseLiquidityAccount {
        liquidity_account,
        owner,
        rent_recipient,
        event_emitter: EventEmitter::address(),
        system_program: system_program::ID,
    }
    .to_account_metas(None);

    Instruction {
        program_id: crate::ID,
        accounts,
        data,
    }
}

pub fn deposit(pool_registry: Pubkey, owner: Pubkey, mint: Pubkey, amount: u64) -> Instruction {
    let data = crate::instruction::Deposit { amount }.data();

    let liquidity_account = LiquidityAccount::address(pool_registry, mint, owner);
    let ssl_pool_signer = SSLPool::signer_address(pool_registry, mint);
    let pool_vault = SSLPool::vault_address(pool_registry, mint);
    let ssl_fee_vault = get_associated_token_address(&pool_registry, &mint);
    let user_ata = get_associated_token_address(&owner, &mint);

    let accounts = crate::accounts::Deposit {
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
        program_id: crate::ID,
        accounts,
        data,
    }
}

pub fn account_metas_for_withdraw(
    pool_registry: Pubkey,
    owner: Pubkey,
    mint: Pubkey,
) -> crate::accounts::Withdraw {
    let liquidity_account = LiquidityAccount::address(pool_registry, mint, owner);
    let ssl_pool_signer = SSLPool::signer_address(pool_registry, mint);
    let pool_vault = SSLPool::vault_address(pool_registry, mint);
    let ssl_fee_vault = get_associated_token_address(&pool_registry, &mint);
    let user_ata = get_associated_token_address(&owner, &mint);

    crate::accounts::Withdraw {
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
}

pub fn withdraw(pool_registry: Pubkey, owner: Pubkey, mint: Pubkey, amount: u64) -> Instruction {
    let data = crate::instruction::Withdraw { amount }.data();

    let accounts = account_metas_for_withdraw(pool_registry, owner, mint).to_account_metas(None);

    Instruction {
        program_id: crate::ID,
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
    let data = crate::instruction::Swap { amount_in, min_out }.data();

    Instruction {
        program_id: crate::ID,
        accounts: get_account_metas_for_swap(
            pool_registry,
            user_wallet,
            mint_in,
            mint_out,
            input_token_oracle,
            output_token_oracle,
            fee_destination,
        )
        .to_account_metas(None),
        data,
    }
}

#[allow(clippy::too_many_arguments)]
pub fn get_account_metas_for_swap(
    pool_registry: Pubkey,
    user_wallet: Pubkey,
    mint_in: Pubkey,
    mint_out: Pubkey,
    input_token_oracle: Pubkey,
    output_token_oracle: Pubkey,
    fee_destination: Pubkey,
) -> crate::accounts::Swap {
    let pair = Pair::address(pool_registry, mint_in, mint_out);
    let ssl_out_fee_vault = get_associated_token_address(&pool_registry, &mint_out);
    let user_ata_in = get_associated_token_address(&user_wallet, &mint_in);
    let user_ata_out = get_associated_token_address(&user_wallet, &mint_out);
    let input_token_price_history =
        OraclePriceHistory::address(&pool_registry, &input_token_oracle);
    let output_token_price_history =
        OraclePriceHistory::address(&pool_registry, &output_token_oracle);
    let ssl_pool_in_signer = SSLPool::signer_address(pool_registry, mint_in);
    let ssl_pool_out_signer = SSLPool::signer_address(pool_registry, mint_out);
    let ssl_in_main_vault = get_associated_token_address(&ssl_pool_in_signer, &mint_in);
    let ssl_in_secondary_vault = get_associated_token_address(&ssl_pool_in_signer, &mint_out);
    let ssl_out_main_vault = get_associated_token_address(&ssl_pool_out_signer, &mint_out);
    let ssl_out_secondary_vault = get_associated_token_address(&ssl_pool_out_signer, &mint_in);

    crate::accounts::Swap {
        pair,
        pool_registry,
        user_wallet,
        ssl_pool_in_signer,
        ssl_pool_out_signer,
        user_ata_in,
        user_ata_out,
        ssl_out_main_vault,
        ssl_out_secondary_vault,
        ssl_in_main_vault,
        ssl_in_secondary_vault,
        ssl_out_fee_vault,
        fee_destination,
        output_token_price_history,
        output_token_oracle,
        input_token_price_history,
        input_token_oracle,
        event_emitter: EventEmitter::address(),
        token_program: token::ID,
    }
}
