use crate::error::{self, GfxSslSdkError};
use anchor_client::{
    anchor_lang::AccountDeserialize,
    solana_client::{nonblocking::rpc_client::RpcClient, rpc_client},
    solana_sdk::pubkey::Pubkey,
};

pub async fn get_state<T: AccountDeserialize>(
    address: &Pubkey,
    client: &RpcClient,
    type_name: &str,
) -> error::Result<T> {
    let data = client
        .get_account_data(address)
        .await
        .map_err(|_| GfxSslSdkError::AccountNotFound(address.clone()))?;
    let state = T::try_deserialize(&mut data.as_slice())
        .map_err(|_| GfxSslSdkError::DeserializeFailure(address.clone(), type_name.to_string()))?;
    Ok(state)
}

pub fn get_state_blocking<T: AccountDeserialize>(
    address: &Pubkey,
    client: &rpc_client::RpcClient,
    type_name: &str,
) -> error::Result<T> {
    let data = client
        .get_account_data(address)
        .map_err(|_| GfxSslSdkError::AccountNotFound(address.clone()))?;
    let state = T::try_deserialize(&mut data.as_slice())
        .map_err(|_| GfxSslSdkError::DeserializeFailure(address.clone(), type_name.to_string()))?;
    Ok(state)
}
