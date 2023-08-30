use crate::avec::A8Bytes;
use crate::error::GfxSslSdkError;
use crate::state::get_account_metas_for_swap;
use anchor_client::anchor_lang::{AccountDeserialize, AccountSerialize};
use anchor_spl::{associated_token::get_associated_token_address, token::TokenAccount};
use anyhow::{anyhow, Error};
use gfx_ssl_v2_interface::{OraclePriceHistory, Pair, PoolRegistry, SSLPool};
use jupiter_amm_interface::{
    AccountMap, Amm, KeyedAccount, Quote, QuoteParams, Swap, SwapAndAccountMetas, SwapParams,
};
use rust_decimal::Decimal;
use solana_sdk::pubkey::Pubkey;
use std::ffi::CString;
use std::fmt::Debug;
use std::mem::size_of;

const DISCRIMINANT: usize = 8;

#[repr(C)]
pub struct SwapResult {
    pub amount_in: u64,
    pub fee_paid: u64,
    pub amount_out: u64,
    pub price_impact: f64,
    pub swap_price: f64,
    pub insta_price: f64,
    pub oracle_price: f64,
    pub iter: u32,
}

#[repr(C)]
pub struct PriceQuote {
    pub amount_in: u64,
    pub amount_out: u64,
    pub fees_paid: u64,
    pub price_impact: f64,
}

/// For FFI.
/// A price in USD, and an amount out in u64,
/// or an error string
#[allow(dead_code)]
#[repr(C)]
pub enum QuoteResult {
    Ok(PriceQuote),
    Error(*mut i8),
}

extern "C" {
    /// A function in a pre-compiled dylib that does
    /// the heavy lifting to derive an accurate swap quote.
    fn quote(
        pool_registry: PoolRegistry,
        pair_act_data: &[u8; size_of::<Pair>() + DISCRIMINANT],
        mint_in_pubkey: Pubkey,
        mint_out_pubkey: Pubkey,
        mint_in_price_history: OraclePriceHistory,
        mint_out_price_history: OraclePriceHistory,
        ssl_in_mint_in_amount: u64,
        ssl_in_mint_out_amount: u64,
        ssl_out_mint_in_amount: u64,
        ssl_out_mint_out_mount: u64,
        amount_in: u64,
    ) -> QuoteResult;
}

/// Struct that implements the `jupiter_core::amm::Amm` trait.
#[derive(Debug, Clone, Default)]
pub struct GfxAmm {
    pool_registry_address: Pubkey,
    pool_registry: Option<PoolRegistry>,

    ssl_a_mint: Pubkey,
    ssl_b_mint: Pubkey,
    /// Keyed by SSL mint
    mint_a_price_history_address: Option<Pubkey>,
    mint_b_price_history_address: Option<Pubkey>,
    mint_a_price_history: Option<OraclePriceHistory>,
    mint_b_price_history: Option<OraclePriceHistory>,

    // Token vaults
    ssl_a_vault_a: Pubkey,
    ssl_a_vault_a_data: Option<TokenAccount>,
    ssl_a_vault_b: Pubkey,
    ssl_a_vault_b_data: Option<TokenAccount>,
    ssl_b_vault_a: Pubkey,
    ssl_b_vault_a_data: Option<TokenAccount>,
    ssl_b_vault_b: Pubkey,
    ssl_b_vault_b_data: Option<TokenAccount>,

    pub pair_pubkey: Pubkey,
    pair_data: Option<A8Bytes<{ size_of::<Pair>() + DISCRIMINANT }>>,
    pair: Option<Pair>, // deserialized for the fee_rate
}

impl GfxAmm {
    pub fn from_keyed_account(pair: &KeyedAccount) -> anyhow::Result<Self> {
        let pair_pubkey = pair.key;
        let data = &pair.account.data;
        let data: A8Bytes<{ size_of::<Pair>() + DISCRIMINANT }> =
            data.clone().try_into().map_err(|_| {
                GfxSslSdkError::InvalidAccountSize(
                    pair.key,
                    size_of::<Pair>() + DISCRIMINANT,
                    data.len(),
                )
            })?;
        let pair_data = Some(data);
        let pair: Pair = Pair::try_deserialize(&mut data.as_slice())
            .map_err(|_| GfxSslSdkError::DeserializeFailure(pair_pubkey, "Pair".to_string()))?;
        let (ssl_a_mint, ssl_b_mint) = pair.mints;
        let pool_registry_address = pair.pool_registry;

        let ssl_a_signer = SSLPool::signer_address(pair.pool_registry, ssl_a_mint);
        let ssl_b_signer = SSLPool::signer_address(pair.pool_registry, ssl_b_mint);

        // Calculate PDAs of GFX accounts
        let ssl_a_vault_a = get_associated_token_address(&ssl_a_signer, &ssl_a_mint);
        let ssl_a_vault_b = get_associated_token_address(&ssl_a_signer, &ssl_b_mint);
        let ssl_b_vault_a = get_associated_token_address(&ssl_b_signer, &ssl_a_mint);
        let ssl_b_vault_b = get_associated_token_address(&ssl_b_signer, &ssl_b_mint);

        Ok(Self {
            pool_registry_address,
            ssl_a_mint,
            ssl_b_mint,
            ssl_a_vault_a,
            ssl_a_vault_b,
            ssl_b_vault_a,
            ssl_b_vault_b,
            pair_pubkey,
            pair_data,
            pair: Some(pair),
            ..Default::default()
        })
    }

    #[allow(clippy::too_many_arguments)]
    /// Variables `a` and `b` correspond to `pair.mints.0` and `pair.mints.1` respectively.
    pub fn new(
        pool_registry: PoolRegistry,
        pair: Pair,
        price_history_a: OraclePriceHistory,
        price_history_b: OraclePriceHistory,
        ssl_a_vault_a: TokenAccount,
        ssl_a_vault_b: TokenAccount,
        ssl_b_vault_a: TokenAccount,
        ssl_b_vault_b: TokenAccount,
    ) -> Result<Self, GfxSslSdkError> {
        if PoolRegistry::address(pool_registry.seed) != pair.pool_registry {
            return Err(GfxSslSdkError::InconsistentInitializationData);
        }
        let ssl_a = pool_registry
            .find_pool(pair.mints.0)
            .map_err(|_| GfxSslSdkError::InconsistentInitializationData)?;
        let ssl_b = pool_registry
            .find_pool(pair.mints.1)
            .map_err(|_| GfxSslSdkError::InconsistentInitializationData)?;

        let pair_pubkey = Pair::address(pair.pool_registry, pair.mints.0, pair.mints.1);

        let ssl_a_signer = SSLPool::signer_address(pair.pool_registry, pair.mints.0);
        let ssl_b_signer = SSLPool::signer_address(pair.pool_registry, pair.mints.1);

        // Calculate PDAs of GFX accounts
        let ssl_a_vault_a_addr = get_associated_token_address(&ssl_a_signer, &pair.mints.0);
        let ssl_a_vault_b_addr = get_associated_token_address(&ssl_a_signer, &pair.mints.1);
        let ssl_b_vault_a_addr = get_associated_token_address(&ssl_b_signer, &pair.mints.0);
        let ssl_b_vault_b_addr = get_associated_token_address(&ssl_b_signer, &pair.mints.1);

        let mut pair_data = vec![];
        pair.try_serialize(&mut pair_data).unwrap();
        let pair_data: A8Bytes<{ size_of::<Pair>() + DISCRIMINANT }> =
            pair_data.try_into().unwrap();

        Ok(Self {
            pool_registry_address: PoolRegistry::address(pair.pool_registry),
            pool_registry: Some(pool_registry),
            ssl_a_mint: pair.mints.0,
            ssl_b_mint: pair.mints.1,
            mint_a_price_history_address: Some(ssl_a.oracle_price_histories[0]),
            mint_b_price_history_address: Some(ssl_b.oracle_price_histories[0]),
            mint_a_price_history: Some(price_history_a),
            mint_b_price_history: Some(price_history_b),
            ssl_a_vault_a: ssl_a_vault_a_addr,
            ssl_a_vault_a_data: Some(ssl_a_vault_a),
            ssl_a_vault_b: ssl_a_vault_b_addr,
            ssl_a_vault_b_data: Some(ssl_a_vault_b),
            ssl_b_vault_a: ssl_b_vault_a_addr,
            ssl_b_vault_a_data: Some(ssl_b_vault_a),
            ssl_b_vault_b: ssl_b_vault_b_addr,
            ssl_b_vault_b_data: Some(ssl_b_vault_b),
            pair_pubkey,
            pair_data: Some(pair_data),
            pair: Some(pair),
        })
    }
}

impl Amm for GfxAmm {
    fn from_keyed_account(keyed_account: &KeyedAccount) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        Self::from_keyed_account(&keyed_account)
    }

    /// Human-readable name for the Amm pair.
    fn label(&self) -> String {
        "GooseFX".to_string()
    }

    fn program_id(&self) -> Pubkey {
        todo!()
    }

    /// Get a pubkey to represent the Amm as a whole.
    fn key(&self) -> Pubkey {
        self.pair_pubkey
    }

    /// Returns mints offered by this Amm for swap.
    fn get_reserve_mints(&self) -> Vec<Pubkey> {
        vec![self.ssl_a_mint, self.ssl_b_mint]
    }

    /// Returns pubkeys of all the accounts required
    /// for providing accurate quotes and swap instructions.
    fn get_accounts_to_update(&self) -> Vec<Pubkey> {
        let mut accounts = vec![
            self.pool_registry_address,
            self.pair_pubkey,
            self.ssl_a_vault_a,
            self.ssl_a_vault_b,
            self.ssl_b_vault_a,
            self.ssl_b_vault_b,
        ];
        if let Some(addr) = self.mint_a_price_history_address {
            accounts.push(addr);
        }
        if let Some(addr) = self.mint_b_price_history_address {
            accounts.push(addr);
        }
        accounts
    }

    /// Update the account state contained in self.
    fn update(&mut self, account_map: &AccountMap) -> Result<(), Error> {
        let update_token_account = |act: &mut Option<TokenAccount>, data: &mut &[u8]| {
            let token_account = TokenAccount::try_deserialize(data)?;
            *act = Some(token_account);
            Ok::<_, Error>(())
        };
        for (pubkey, account) in account_map {
            let pubkey = pubkey.clone();
            let data = account.data.clone();
            if pubkey == self.pool_registry_address {
                let pool_registry =
                    PoolRegistry::try_deserialize(&mut data.as_slice()).map_err(|_| {
                        GfxSslSdkError::DeserializeFailure(pubkey, "PoolRegistry".to_string())
                    })?;
                let ssl_a: &SSLPool = pool_registry
                    .find_pool(self.ssl_a_mint)
                    .map_err(|_| GfxSslSdkError::PoolNotFound(self.ssl_a_mint))?;
                self.mint_a_price_history_address = Some(ssl_a.oracle_price_histories[0]);
                let ssl_b: &SSLPool = pool_registry
                    .find_pool(self.ssl_b_mint)
                    .map_err(|_| GfxSslSdkError::PoolNotFound(self.ssl_b_mint))?;
                self.mint_b_price_history_address = Some(ssl_b.oracle_price_histories[0]);
                self.pool_registry = Some(pool_registry);
            } else if pubkey == self.pair_pubkey {
                let pair = Pair::try_deserialize(&mut data.as_slice())
                    .map_err(|_| GfxSslSdkError::DeserializeFailure(pubkey, "Pair".to_string()))?;
                self.pair = Some(pair);
            } else if pubkey == self.ssl_a_vault_a {
                update_token_account(&mut self.ssl_a_vault_a_data, &mut data.as_slice())?;
            } else if pubkey == self.ssl_a_vault_b {
                update_token_account(&mut self.ssl_a_vault_b_data, &mut data.as_slice())?;
            } else if pubkey == self.ssl_b_vault_a {
                update_token_account(&mut self.ssl_b_vault_a_data, &mut data.as_slice())?;
            } else if pubkey == self.ssl_b_vault_b {
                update_token_account(&mut self.ssl_b_vault_b_data, &mut data.as_slice())?;
            } else {
                // Assume it's an oracle price history
                if let Some(addr) = self.mint_a_price_history_address {
                    if pubkey == addr {
                        let price_history = OraclePriceHistory::try_deserialize(
                            &mut data.as_slice(),
                        )
                        .map_err(|_| {
                            GfxSslSdkError::DeserializeFailure(
                                pubkey,
                                "OraclePriceHistory".to_string(),
                            )
                        })?;
                        self.mint_a_price_history = Some(price_history);
                    }
                }
                if let Some(addr) = self.mint_b_price_history_address {
                    if pubkey == addr {
                        let price_history = OraclePriceHistory::try_deserialize(
                            &mut data.as_slice(),
                        )
                        .map_err(|_| {
                            GfxSslSdkError::DeserializeFailure(
                                pubkey,
                                "OraclePriceHistory".to_string(),
                            )
                        })?;
                        self.mint_b_price_history = Some(price_history);
                    }
                }
            }
        }
        Ok(())
    }

    /// Get a GooseFX SSL swap quote
    fn quote(&self, quote_params: &QuoteParams) -> Result<Quote, Error> {
        if self.pair.is_none()
            || self.pool_registry.is_none()
            || self.mint_a_price_history_address.is_none()
            || self.mint_b_price_history_address.is_none()
            || self.ssl_a_vault_a_data.is_none()
            || self.ssl_a_vault_b_data.is_none()
            || self.ssl_b_vault_a_data.is_none()
            || self.ssl_b_vault_b_data.is_none()
        {
            return Err(GfxSslSdkError::RequiredAccountUpdate.into());
        }

        if self.mint_a_price_history.is_none() || self.mint_b_price_history.is_none() {
            return Err(GfxSslSdkError::RequiredAccountUpdate.into());
        }

        // Orient each side of the pair as "in" our "out".
        // Keep a boolean flag that helps keep track of whether to flip
        // other arguments later in this function
        let mut is_reversed = false;
        if quote_params.input_mint == self.ssl_b_mint && quote_params.output_mint == self.ssl_a_mint
        {
            is_reversed = true;
        } else if quote_params.input_mint != self.ssl_a_mint
            || quote_params.output_mint != self.ssl_b_mint
        {
            return Err(GfxSslSdkError::UnexpectedMints.into());
        }

        let (
            mint_in_pubkey,
            mint_out_pubkey,
            mint_in_price_history,
            mint_out_price_history,
            ssl_in_mint_in,
            ssl_in_mint_out,
            ssl_out_mint_in,
            ssl_out_mint_out,
        ) = if !is_reversed {
            (
                self.ssl_a_mint,
                self.ssl_b_mint,
                self.mint_a_price_history.unwrap(),
                self.mint_b_price_history.unwrap(),
                self.ssl_a_vault_a_data.clone().unwrap(),
                self.ssl_a_vault_b_data.clone().unwrap(),
                self.ssl_b_vault_a_data.clone().unwrap(),
                self.ssl_b_vault_b_data.clone().unwrap(),
            )
        } else {
            (
                self.ssl_b_mint,
                self.ssl_a_mint,
                self.mint_b_price_history.unwrap(),
                self.mint_a_price_history.unwrap(),
                self.ssl_b_vault_b_data.clone().unwrap(),
                self.ssl_b_vault_a_data.clone().unwrap(),
                self.ssl_a_vault_b_data.clone().unwrap(),
                self.ssl_a_vault_a_data.clone().unwrap(),
            )
        };

        match unsafe {
            quote(
                self.pool_registry.unwrap(),
                &self.pair_data.unwrap(),
                mint_in_pubkey,
                mint_out_pubkey,
                mint_in_price_history,
                mint_out_price_history,
                ssl_in_mint_in.amount,
                ssl_in_mint_out.amount,
                ssl_out_mint_in.amount,
                ssl_out_mint_out.amount,
                quote_params.amount,
            )
        } {
            QuoteResult::Ok(swap_result) => {
                let fee_pct = if !is_reversed {
                    self.pair.as_ref().unwrap().fee_rates.0
                } else {
                    self.pair.as_ref().unwrap().fee_rates.1
                };
                let fee_pct = Decimal::new(fee_pct.into(), 4);

                let quote = Quote {
                    not_enough_liquidity: false,
                    min_in_amount: None,
                    min_out_amount: None,
                    in_amount: swap_result.amount_in,
                    out_amount: swap_result.amount_out,
                    fee_amount: swap_result.fees_paid,
                    fee_mint: mint_out_pubkey,
                    fee_pct,
                };
                Ok(quote)
            }
            QuoteResult::Error(err) => unsafe {
                let c_str = CString::from_raw(err);
                let rust_str = c_str.to_str().expect("bad string encoding");
                Err(anyhow!("{}", rust_str))
            },
        }
    }

    /// Get account metas for a GFX swap instruction,
    /// and marker denoting a [SwapLeg::Swap], and a [Swap::GooseFX].
    fn get_swap_and_account_metas(
        &self,
        swap_params: &SwapParams,
    ) -> anyhow::Result<SwapAndAccountMetas> {
        // We need these accounts to be updated in order to create swap account metas
        if self.pair.is_none()
            || self.mint_a_price_history.is_none()
            || self.mint_b_price_history.is_none()
        {
            return Err(GfxSslSdkError::RequiredAccountUpdate.into());
        }

        let (mint_in, mint_out, input_token_price_history, output_token_price_history) =
            if swap_params.source_mint == self.ssl_a_mint {
                (
                    self.ssl_a_mint,
                    self.ssl_b_mint,
                    self.mint_a_price_history.unwrap(),
                    self.mint_b_price_history.unwrap(),
                )
            } else {
                (
                    self.ssl_b_mint,
                    self.ssl_a_mint,
                    self.mint_b_price_history.unwrap(),
                    self.mint_a_price_history.unwrap(),
                )
            };
        let pair = self.pair.as_ref().unwrap();
        let fee_destination = if pair.mints.0 == mint_out {
            pair.fee_collector.0
        } else {
            pair.fee_collector.1
        };

        Ok(SwapAndAccountMetas {
            swap: Swap::GooseFX,
            account_metas: get_account_metas_for_swap(
                self.pool_registry_address,
                swap_params.token_transfer_authority,
                mint_in,
                mint_out,
                input_token_price_history.oracle_address,
                output_token_price_history.oracle_address,
                fee_destination,
            ),
        })
    }

    /// Clone this object in a [Box].
    fn clone_amm(&self) -> Box<dyn Amm + Send + Sync> {
        Box::new(self.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instructions::swap;
    use solana_sdk::account::Account;

    #[test]
    fn swap_account_metas() {
        let admin = Pubkey::new_unique();
        let user_wallet = Pubkey::new_unique();
        let mint_a = Pubkey::new_unique();
        let mint_b = Pubkey::new_unique();
        let oracle_a = Pubkey::new_unique();
        let oracle_b = Pubkey::new_unique();
        let fee_destination = Pubkey::new_unique();
        let false_fee_destination = Pubkey::new_unique();
        let pool_registry = PoolRegistry::address(admin);
        // First test where mint_a = mint_in, and mint_b = mint_out
        let ix = swap(
            0,
            0,
            pool_registry,
            user_wallet,
            mint_a,
            mint_b,
            oracle_a,
            oracle_b,
            fee_destination,
        );

        let mut pair = Pair::default();
        pair.pool_registry = pool_registry;
        pair.mints = (mint_a, mint_b);
        pair.fee_collector = (false_fee_destination, fee_destination);
        let mut price_history_a = OraclePriceHistory::default();
        price_history_a.oracle_address = oracle_a;
        let mut price_history_b = OraclePriceHistory::default();
        price_history_b.oracle_address = oracle_b;

        let pair_pubkey = Pair::address(pool_registry, mint_a, mint_b);
        let mut data = vec![];
        pair.try_serialize(&mut data).unwrap();
        let account = Account {
            lamports: 0,
            data,
            owner: Default::default(),
            executable: false,
            rent_epoch: 0,
        };
        let keyed_account = KeyedAccount {
            key: pair_pubkey,
            account,
            params: None,
        };

        let mut gfx_amm = GfxAmm::from_keyed_account(&keyed_account).unwrap();
        gfx_amm.pair = Some(pair);
        gfx_amm.mint_a_price_history_address =
            Some(OraclePriceHistory::address(&pool_registry, &oracle_a));
        gfx_amm.mint_a_price_history = Some(price_history_a.clone());
        gfx_amm.mint_b_price_history_address =
            Some(OraclePriceHistory::address(&pool_registry, &oracle_b));
        gfx_amm.mint_b_price_history = Some(price_history_b.clone());
        let account_metas = gfx_amm
            .get_swap_and_account_metas(&SwapParams {
                source_mint: mint_a,
                destination_mint: mint_b,
                source_token_account: get_associated_token_address(&user_wallet, &mint_a),
                destination_token_account: get_associated_token_address(&user_wallet, &mint_b),
                open_order_address: None,
                quote_mint_to_referrer: None,
                in_amount: 0,
                out_amount: 0,
                token_transfer_authority: Default::default(),
                jupiter_program_id: &Default::default(),
            })
            .unwrap()
            .account_metas;
        assert_eq!(account_metas, ix.accounts);

        let ix = swap(
            0,
            0,
            pool_registry,
            user_wallet,
            mint_b,
            mint_a,
            oracle_b,
            oracle_a,
            fee_destination,
        );

        let mut pair = Pair::default();
        pair.pool_registry = pool_registry;
        pair.mints = (mint_a, mint_b);
        pair.fee_collector = (fee_destination, false_fee_destination);
        let pair_pubkey = Pair::address(pool_registry, mint_a, mint_b);
        let mut data = vec![];
        pair.try_serialize(&mut data).unwrap();
        let account = Account {
            lamports: 0,
            data,
            owner: Default::default(),
            executable: false,
            rent_epoch: 0,
        };

        let keyed_account = KeyedAccount {
            key: pair_pubkey,
            account,
            params: None,
        };
        let mut gfx_amm = GfxAmm::from_keyed_account(&keyed_account).unwrap();
        gfx_amm.pair = Some(pair);
        gfx_amm.mint_a_price_history_address =
            Some(OraclePriceHistory::address(&pool_registry, &oracle_a));
        gfx_amm.mint_a_price_history = Some(price_history_a);
        gfx_amm.mint_b_price_history_address =
            Some(OraclePriceHistory::address(&pool_registry, &oracle_b));
        gfx_amm.mint_b_price_history = Some(price_history_b);
        let account_metas = gfx_amm
            .get_swap_and_account_metas(&SwapParams {
                source_mint: mint_b,
                destination_mint: mint_a,
                source_token_account: get_associated_token_address(&user_wallet, &mint_b),
                destination_token_account: get_associated_token_address(&user_wallet, &mint_a),
                token_transfer_authority: user_wallet,
                open_order_address: None,
                quote_mint_to_referrer: None,
                in_amount: 0,
                out_amount: 0,
                jupiter_program_id: &Default::default(),
            })
            .unwrap()
            .account_metas;
        assert_eq!(account_metas, ix.accounts);
    }
}
