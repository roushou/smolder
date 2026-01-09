use alloy::network::EthereumWallet;
use alloy::primitives::{Address, Bytes, U256};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::rpc::types::TransactionRequest;
use alloy::signers::local::PrivateKeySigner;
use smolder_core::Error;

pub async fn execute_eth_call(rpc_url: &str, to: Address, data: Bytes) -> Result<Bytes, Error> {
    let url: reqwest::Url = rpc_url
        .parse()
        .map_err(|e| Error::invalid_param("rpc_url", format!("Invalid RPC URL: {}", e)))?;
    let provider = ProviderBuilder::new().connect_http(url);

    let tx = TransactionRequest::default().to(to).input(data.into());

    let result: Bytes = provider
        .call(tx)
        .await
        .map_err(|e| Error::Rpc(format!("RPC call failed: {}", e)))?;

    Ok(result)
}

pub async fn execute_transaction(
    rpc_url: &str,
    private_key: &str,
    to: Address,
    data: Bytes,
    value: Option<U256>,
) -> Result<String, Error> {
    let signer: PrivateKeySigner = private_key
        .parse()
        .map_err(|e| Error::invalid_param("private_key", format!("Invalid: {}", e)))?;

    let wallet = EthereumWallet::from(signer);

    let url: reqwest::Url = rpc_url
        .parse()
        .map_err(|e| Error::invalid_param("rpc_url", format!("Invalid RPC URL: {}", e)))?;
    let provider = ProviderBuilder::new().wallet(wallet).connect_http(url);

    let mut tx = TransactionRequest::default().to(to).input(data.into());

    if let Some(v) = value {
        tx = tx.value(v);
    }

    let pending = provider
        .send_transaction(tx)
        .await
        .map_err(|e| Error::TransactionFailed(format!("{}", e)))?;

    Ok(format!("{:?}", pending.tx_hash()))
}
