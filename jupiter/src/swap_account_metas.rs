use anchor_lang::{prelude::AccountMeta, solana_program::pubkey::Pubkey, ToAccountMetas};
use anchor_spl::{associated_token::get_associated_token_address, token};
use gfx_ssl_v2_sdk::state::*;

#[allow(clippy::too_many_arguments)]
pub fn get_account_metas_for_swap(
    pool_registry: Pubkey,
    user_wallet: Pubkey,
    mint_in: Pubkey,
    mint_out: Pubkey,
    input_token_oracle: Pubkey,
    output_token_oracle: Pubkey,
    backup_input_token_oracle: Pubkey,
    backup_output_token_oracle: Pubkey,
    fee_destination: Pubkey,
) -> Vec<AccountMeta> {
    let pair = Pair::address(pool_registry, mint_in, mint_out);
    let ssl_out_fee_vault = get_associated_token_address(&pool_registry, &mint_out);
    let user_ata_in = get_associated_token_address(&user_wallet, &mint_in);
    let user_ata_out = get_associated_token_address(&user_wallet, &mint_out);
    let input_token_price_history =
        OraclePriceHistory::address(&pool_registry, &input_token_oracle);
    let output_token_price_history =
        OraclePriceHistory::address(&pool_registry, &output_token_oracle);
    let backup_input_token_price_history =
        OraclePriceHistory::address(&pool_registry, &backup_input_token_oracle);
    let backup_output_token_price_history =
        OraclePriceHistory::address(&pool_registry, &backup_output_token_oracle);
    let ssl_pool_in_signer = SSLPool::signer_address(pool_registry, mint_in);
    let ssl_pool_out_signer = SSLPool::signer_address(pool_registry, mint_out);
    let ssl_in_main_vault = get_associated_token_address(&ssl_pool_in_signer, &mint_in);
    let ssl_in_secondary_vault = get_associated_token_address(&ssl_pool_in_signer, &mint_out);
    let ssl_out_main_vault = get_associated_token_address(&ssl_pool_out_signer, &mint_out);
    let ssl_out_secondary_vault = get_associated_token_address(&ssl_pool_out_signer, &mint_in);

    gfx_ssl_v2_sdk::anchor::accounts::Swap {
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
        backup_output_token_price_history,
        backup_output_token_oracle,
        backup_input_token_price_history,
        backup_input_token_oracle,
        event_emitter: EventEmitter::address(),
        token_program: token::ID,
    }
    .to_account_metas(None)
}
