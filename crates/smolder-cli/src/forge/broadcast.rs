//! Broadcast parsing trait and forge implementation

use alloy::hex;
use alloy::primitives::keccak256;
use color_eyre::eyre::{eyre, Result};
use std::path::{Path, PathBuf};
use std::sync::Arc;

use super::artifact::{ArtifactLoader, FileSystemArtifactLoader};
use super::types::{BroadcastOutput, ParsedDeployment};

/// Trait for parsing broadcast outputs from deployment scripts
pub trait BroadcastParser: Send + Sync {
    /// Parse the broadcast output for a given script and chain ID
    fn parse(&self, script_path: &str, chain_id: u64) -> Result<BroadcastOutput>;

    /// Extract deployment information from a broadcast output
    fn extract_deployments(&self, broadcast: &BroadcastOutput) -> Result<Vec<ParsedDeployment>>;
}

/// Broadcast parser for forge script outputs
#[derive(Clone)]
pub struct ForgeBroadcastParser {
    /// Directory containing broadcast outputs (typically "broadcast")
    broadcast_dir: PathBuf,
    /// Artifact loader for fetching contract ABIs and bytecode
    artifact_loader: Arc<dyn ArtifactLoader>,
}

impl ForgeBroadcastParser {
    /// Create a new parser with default paths relative to current directory
    pub fn new() -> Self {
        Self::with_paths(Path::new("."))
    }

    /// Create a new parser with paths relative to the given project root
    pub fn with_paths(project_root: &Path) -> Self {
        Self {
            broadcast_dir: project_root.join("broadcast"),
            artifact_loader: Arc::new(FileSystemArtifactLoader::with_paths(project_root)),
        }
    }

    /// Create a new parser with explicit broadcast directory and artifact loader
    #[allow(dead_code)]
    pub fn with_loader(broadcast_dir: PathBuf, artifact_loader: Arc<dyn ArtifactLoader>) -> Self {
        Self {
            broadcast_dir,
            artifact_loader,
        }
    }

    /// Extract deployment info from a single transaction
    fn extract_single_deployment(
        &self,
        tx: &super::types::BroadcastTransaction,
        broadcast: &BroadcastOutput,
    ) -> Result<ParsedDeployment> {
        let contract_name = tx.contract_name.as_ref().unwrap().clone();
        let address = tx.contract_address.as_ref().unwrap().clone();

        // Load artifact for this contract
        let artifact = self.artifact_loader.load(&contract_name)?;

        // Find matching receipt for block number
        let block_number = broadcast
            .receipts
            .iter()
            .find(|r| r.transaction_hash == tx.hash)
            .and_then(|r| parse_hex_block_number(&r.block_number));

        // Compute bytecode hash
        let bytecode = artifact.bytecode.without_prefix();
        let bytecode_bytes = hex::decode(bytecode).unwrap_or_default();
        let bytecode_hash = format!("{:x}", keccak256(&bytecode_bytes));

        // Serialize constructor args if present
        let constructor_args = tx
            .arguments
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;

        let source_path = format!("src/{}.sol:{}", contract_name, contract_name);

        Ok(ParsedDeployment {
            contract_name,
            address,
            deployer: tx.transaction.from.clone(),
            tx_hash: tx.hash.clone(),
            block_number,
            constructor_args,
            abi: serde_json::to_string(&artifact.abi)?,
            bytecode_hash,
            source_path,
        })
    }
}

impl Default for ForgeBroadcastParser {
    fn default() -> Self {
        Self::new()
    }
}

impl BroadcastParser for ForgeBroadcastParser {
    fn parse(&self, script_path: &str, chain_id: u64) -> Result<BroadcastOutput> {
        // Strip :ContractName suffix if present
        let script_file = script_path.split(':').next().unwrap_or(script_path);

        // Extract script name from path
        let script_name = Path::new(script_file)
            .file_name()
            .and_then(|n| n.to_str())
            .ok_or_else(|| eyre!("Invalid script path"))?;

        let broadcast_path = self
            .broadcast_dir
            .join(script_name)
            .join(chain_id.to_string())
            .join("run-latest.json");

        let content = std::fs::read_to_string(&broadcast_path).map_err(|_| {
            eyre!(
                "Could not find broadcast output at {}. Make sure the script was run with --broadcast.",
                broadcast_path.display()
            )
        })?;

        let output: BroadcastOutput = serde_json::from_str(&content)?;
        Ok(output)
    }

    fn extract_deployments(&self, broadcast: &BroadcastOutput) -> Result<Vec<ParsedDeployment>> {
        broadcast
            .transactions
            .iter()
            .filter(|tx| tx.is_create() && tx.has_deployment_info())
            .map(|tx| self.extract_single_deployment(tx, broadcast))
            .collect()
    }
}

/// Parse a hex block number string to i64
fn parse_hex_block_number(hex_str: &str) -> Option<i64> {
    let hex = hex_str.trim_start_matches("0x");
    i64::from_str_radix(hex, 16).ok()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::forge::types::BroadcastOutput;

    #[test]
    fn test_parse_broadcast_output() {
        let json = r#"{
            "transactions": [
                {
                    "hash": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
                    "transactionType": "CREATE",
                    "contractName": "MyToken",
                    "contractAddress": "0xabcdef1234567890abcdef1234567890abcdef12",
                    "arguments": ["Test Token", "TT"],
                    "transaction": {
                        "from": "0x1111111111111111111111111111111111111111",
                        "data": "0x6080604052"
                    }
                }
            ],
            "receipts": [
                {
                    "transactionHash": "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
                    "blockNumber": "0x10",
                    "contractAddress": "0xabcdef1234567890abcdef1234567890abcdef12"
                }
            ]
        }"#;

        let output: BroadcastOutput = serde_json::from_str(json).unwrap();

        assert_eq!(output.transactions.len(), 1);
        assert_eq!(output.receipts.len(), 1);

        let tx = &output.transactions[0];
        assert!(tx.is_create());
        assert!(tx.has_deployment_info());
        assert_eq!(tx.contract_name, Some("MyToken".to_string()));
    }

    #[test]
    fn test_parse_broadcast_with_call_transaction() {
        let json = r#"{
            "transactions": [
                {
                    "hash": "0xaaaa",
                    "transactionType": "CALL",
                    "contractName": null,
                    "contractAddress": null,
                    "arguments": null,
                    "transaction": {
                        "from": "0x1111111111111111111111111111111111111111",
                        "data": null
                    }
                },
                {
                    "hash": "0xbbbb",
                    "transactionType": "CREATE",
                    "contractName": "Token",
                    "contractAddress": "0x2222222222222222222222222222222222222222",
                    "arguments": [],
                    "transaction": {
                        "from": "0x1111111111111111111111111111111111111111",
                        "data": "0x6080"
                    }
                }
            ],
            "receipts": []
        }"#;

        let output: BroadcastOutput = serde_json::from_str(json).unwrap();

        assert_eq!(output.transactions.len(), 2);
        assert!(!output.transactions[0].is_create());
        assert!(output.transactions[1].is_create());
    }

    #[test]
    fn test_parse_hex_block_number() {
        assert_eq!(parse_hex_block_number("0x1a2b3c"), Some(1715004));
        assert_eq!(parse_hex_block_number("1a2b3c"), Some(1715004));
        assert_eq!(parse_hex_block_number("0x10"), Some(16));
        assert_eq!(parse_hex_block_number("0x0"), Some(0));
    }

    #[test]
    fn test_empty_broadcast() {
        let json = r#"{
            "transactions": [],
            "receipts": []
        }"#;

        let output: BroadcastOutput = serde_json::from_str(json).unwrap();

        assert!(output.transactions.is_empty());
        assert!(output.receipts.is_empty());
    }

    #[test]
    fn test_script_path_stripping() {
        let with_contract = "script/Deploy.s.sol:Deploy";
        let stripped = with_contract.split(':').next().unwrap();
        assert_eq!(stripped, "script/Deploy.s.sol");

        let without_contract = "script/Deploy.s.sol";
        let stripped = without_contract.split(':').next().unwrap();
        assert_eq!(stripped, "script/Deploy.s.sol");
    }

    #[test]
    fn test_bytecode_hash_computation() {
        let bytecode = "6080604052";
        let bytecode_bytes = hex::decode(bytecode).unwrap();
        let hash = format!("{:x}", keccak256(&bytecode_bytes));

        assert_eq!(hash.len(), 64);
    }

    #[test]
    fn test_parser_with_custom_paths() {
        let loader = Arc::new(FileSystemArtifactLoader::with_paths(Path::new("/custom")));
        let parser = ForgeBroadcastParser::with_loader(PathBuf::from("/custom/broadcast"), loader);

        assert_eq!(parser.broadcast_dir, PathBuf::from("/custom/broadcast"));
    }
}
