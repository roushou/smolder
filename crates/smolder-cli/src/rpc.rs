use alloy::providers::{Provider, ProviderBuilder};
use alloy::transports::http::reqwest::Url;
use color_eyre::eyre::Result;

/// Fetch the chain ID from an RPC endpoint
pub async fn get_chain_id(rpc_url: &str) -> Result<u64> {
    let url: Url = rpc_url.parse()?;
    let provider = ProviderBuilder::new().connect_http(url);
    let chain_id = provider.get_chain_id().await?;
    Ok(chain_id)
}
