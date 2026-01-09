use std::collections::HashMap;
use std::path::Path;

use color_eyre::eyre::{eyre, Result};
use serde::Deserialize;

const FOUNDRY_CONFIG: &str = "foundry.toml";

/// Foundry configuration file structure (foundry.toml)
/// We only parse the sections we need
#[derive(Debug, Clone, Deserialize)]
pub struct FoundryConfig {
    #[serde(default)]
    pub rpc_endpoints: HashMap<String, RpcEndpoint>,
    #[serde(default)]
    pub etherscan: HashMap<String, EtherscanConfig>,
}

/// RPC endpoint can be a string or an object with url field
#[derive(Debug, Clone, Deserialize)]
#[serde(untagged)]
pub enum RpcEndpoint {
    Url(String),
    Object { url: String },
}

impl RpcEndpoint {
    pub fn url(&self) -> &str {
        match self {
            RpcEndpoint::Url(url) => url,
            RpcEndpoint::Object { url } => url,
        }
    }
}

/// Etherscan config for a network
#[derive(Debug, Clone, Deserialize)]
pub struct EtherscanConfig {
    /// API key for contract verification (parsed but not yet used)
    #[serde(default)]
    #[allow(dead_code)]
    pub key: Option<String>,
    #[serde(default)]
    pub url: Option<String>,
}

impl FoundryConfig {
    /// Load configuration from foundry.toml in the current directory
    pub fn load() -> Result<Self> {
        Self::load_from(Path::new(FOUNDRY_CONFIG))
    }

    /// Load configuration from a specific path
    pub fn load_from(path: &Path) -> Result<Self> {
        let content = std::fs::read_to_string(path)
            .map_err(|_| eyre!("Could not find foundry.toml. Is this a Foundry project?"))?;

        let config: FoundryConfig = toml::from_str(&content)?;
        Ok(config)
    }

    /// Get a network configuration by name, resolving environment variables
    /// Note: chain_id must be fetched from RPC separately
    pub fn get_network(&self, name: &str) -> Result<NetworkConfig> {
        let rpc_endpoint = self.rpc_endpoints.get(name).ok_or_else(|| {
            eyre!(
                "Network '{}' not found in foundry.toml [rpc_endpoints]",
                name
            )
        })?;

        let rpc_url = resolve_env_var(rpc_endpoint.url())?;

        let explorer_url = self
            .etherscan
            .get(name)
            .and_then(|e| e.url.as_ref())
            .map(|u| resolve_env_var(u))
            .transpose()?;

        Ok(NetworkConfig {
            name: name.to_string(),
            rpc_url,
            explorer_url,
        })
    }

    /// Check if foundry.toml exists
    pub fn exists() -> bool {
        Path::new(FOUNDRY_CONFIG).exists()
    }

    /// Get all network names defined in foundry.toml
    pub fn network_names(&self) -> Vec<&str> {
        self.rpc_endpoints.keys().map(|s| s.as_str()).collect()
    }
}

/// Network configuration extracted from foundry.toml
/// chain_id is not included here - it should be fetched from RPC
#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub name: String,
    pub rpc_url: String,
    pub explorer_url: Option<String>,
}

/// Resolve environment variable references in a string
/// Supports ${VAR_NAME} syntax
fn resolve_env_var(value: &str) -> Result<String> {
    if value.starts_with("${") && value.ends_with('}') {
        let var_name = &value[2..value.len() - 1];
        std::env::var(var_name).map_err(|_| eyre!("Environment variable '{}' not set", var_name))
    } else {
        Ok(value.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_foundry_config() {
        let toml_content = r#"
[rpc_endpoints]
mainnet = "https://eth.llamarpc.com"
tempo-testnet = "${TEMPO_TESTNET_RPC_URL}"

[etherscan]
mainnet = { key = "${ETHERSCAN_API_KEY}", url = "https://api.etherscan.io/api" }
tempo-testnet = { key = "${API_KEY}", url = "https://testnet.tempotestnetscan.io/api" }
"#;

        let config: FoundryConfig = toml::from_str(toml_content).unwrap();

        assert!(config.rpc_endpoints.contains_key("mainnet"));
        assert!(config.rpc_endpoints.contains_key("tempo-testnet"));
        assert_eq!(
            config.rpc_endpoints.get("mainnet").unwrap().url(),
            "https://eth.llamarpc.com"
        );
    }

    #[test]
    fn test_parse_minimal_foundry_config() {
        let toml_content = r#"
[profile.default]
src = "src"
"#;

        let config: FoundryConfig = toml::from_str(toml_content).unwrap();

        assert!(config.rpc_endpoints.is_empty());
        assert!(config.etherscan.is_empty());
    }

    #[test]
    fn test_get_network_with_explorer() {
        std::env::set_var("TEST_RPC_URL", "https://rpc.test.xyz");

        let toml_content = r#"
[rpc_endpoints]
testnet = "${TEST_RPC_URL}"

[etherscan]
testnet = { url = "https://explorer.test.xyz/api" }
"#;

        let config: FoundryConfig = toml::from_str(toml_content).unwrap();
        let network = config.get_network("testnet").unwrap();

        assert_eq!(network.name, "testnet");
        assert_eq!(network.rpc_url, "https://rpc.test.xyz");
        assert_eq!(
            network.explorer_url,
            Some("https://explorer.test.xyz/api".to_string())
        );

        std::env::remove_var("TEST_RPC_URL");
    }

    #[test]
    fn test_get_network_without_explorer() {
        let toml_content = r#"
[rpc_endpoints]
local = "http://localhost:8545"
"#;

        let config: FoundryConfig = toml::from_str(toml_content).unwrap();
        let network = config.get_network("local").unwrap();

        assert_eq!(network.name, "local");
        assert_eq!(network.rpc_url, "http://localhost:8545");
        assert!(network.explorer_url.is_none());
    }

    #[test]
    fn test_get_network_not_found() {
        let toml_content = r#"
[rpc_endpoints]
mainnet = "https://eth.rpc"
"#;

        let config: FoundryConfig = toml::from_str(toml_content).unwrap();
        let result = config.get_network("nonexistent");

        assert!(result.is_err());
    }

    #[test]
    fn test_resolve_env_var() {
        std::env::set_var("TEST_VAR_123", "resolved_value");

        let result = resolve_env_var("${TEST_VAR_123}").unwrap();
        assert_eq!(result, "resolved_value");

        std::env::remove_var("TEST_VAR_123");
    }

    #[test]
    fn test_resolve_env_var_literal() {
        let result = resolve_env_var("https://literal.url").unwrap();
        assert_eq!(result, "https://literal.url");
    }

    #[test]
    fn test_resolve_env_var_missing() {
        let result = resolve_env_var("${NONEXISTENT_VAR_99999}");
        assert!(result.is_err());
    }

    #[test]
    fn test_rpc_endpoint_object_format() {
        let toml_content = r#"
[rpc_endpoints]
mainnet = { url = "https://eth.rpc.xyz" }
"#;

        let config: FoundryConfig = toml::from_str(toml_content).unwrap();
        assert_eq!(
            config.rpc_endpoints.get("mainnet").unwrap().url(),
            "https://eth.rpc.xyz"
        );
    }
}
