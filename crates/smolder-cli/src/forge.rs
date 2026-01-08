use alloy::hex;
use alloy::primitives::keccak256;
use color_eyre::eyre::{eyre, Result};
use serde::Deserialize;
use std::path::Path;

/// Represents the broadcast output from forge script
#[derive(Debug, Deserialize)]
pub struct BroadcastOutput {
    pub transactions: Vec<BroadcastTransaction>,
    pub receipts: Vec<BroadcastReceipt>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BroadcastTransaction {
    pub hash: String,
    pub transaction_type: String,
    pub contract_name: Option<String>,
    pub contract_address: Option<String>,
    pub arguments: Option<Vec<serde_json::Value>>,
    pub transaction: TransactionData,
}

#[derive(Debug, Deserialize)]
pub struct TransactionData {
    pub from: String,
    pub data: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BroadcastReceipt {
    pub transaction_hash: String,
    pub block_number: String,
    pub contract_address: Option<String>,
}

/// Represents a contract artifact from forge build output
#[derive(Debug, Deserialize)]
pub struct ContractArtifact {
    pub abi: serde_json::Value,
    pub bytecode: BytecodeObject,
    #[serde(rename = "deployedBytecode")]
    pub deployed_bytecode: BytecodeObject,
}

#[derive(Debug, Deserialize)]
pub struct BytecodeObject {
    pub object: String,
}

/// Parsed deployment information
#[derive(Debug)]
pub struct ParsedDeployment {
    pub contract_name: String,
    pub address: String,
    pub deployer: String,
    pub tx_hash: String,
    pub block_number: Option<i64>,
    pub constructor_args: Option<String>,
    pub abi: String,
    pub bytecode_hash: String,
    pub source_path: String,
}

/// Find and parse the broadcast output for a given script and chain ID
pub fn parse_broadcast(script_path: &str, chain_id: u64) -> Result<BroadcastOutput> {
    // Strip :ContractName suffix if present (e.g., "script/Deploy.s.sol:Deploy" -> "script/Deploy.s.sol")
    let script_file = script_path.split(':').next().unwrap_or(script_path);

    // Extract script name from path (e.g., "script/Deploy.s.sol" -> "Deploy.s.sol")
    let script_name = Path::new(script_file)
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| eyre!("Invalid script path"))?;

    let broadcast_path = format!("broadcast/{}/{}/run-latest.json", script_name, chain_id);

    let content = std::fs::read_to_string(&broadcast_path).map_err(|_| {
        eyre!(
            "Could not find broadcast output at {}. Make sure the script was run with --broadcast.",
            broadcast_path
        )
    })?;

    let output: BroadcastOutput = serde_json::from_str(&content)?;
    Ok(output)
}

/// Load a contract artifact from the forge output directory
pub fn load_artifact(contract_name: &str) -> Result<ContractArtifact> {
    // Try common artifact locations
    let possible_paths = [
        format!("out/{}.sol/{}.json", contract_name, contract_name),
        format!("out/{}/{}.json", contract_name, contract_name),
    ];

    for path in &possible_paths {
        if let Ok(content) = std::fs::read_to_string(path) {
            let artifact: ContractArtifact = serde_json::from_str(&content)?;
            return Ok(artifact);
        }
    }

    Err(eyre!(
        "Could not find artifact for contract '{}'. Make sure `forge build` was run.",
        contract_name
    ))
}

/// Extract deployment information from broadcast output and artifacts
pub fn extract_deployments(broadcast: &BroadcastOutput) -> Result<Vec<ParsedDeployment>> {
    let mut deployments = Vec::new();

    for tx in &broadcast.transactions {
        // Only process CREATE transactions
        if tx.transaction_type != "CREATE" {
            continue;
        }

        let contract_name = match &tx.contract_name {
            Some(name) => name.clone(),
            None => continue,
        };

        let address = match &tx.contract_address {
            Some(addr) => addr.clone(),
            None => continue,
        };

        // Load artifact for this contract
        let artifact = load_artifact(&contract_name)?;

        // Find matching receipt for block number
        let block_number = broadcast
            .receipts
            .iter()
            .find(|r| r.transaction_hash == tx.hash)
            .and_then(|r| {
                // Parse hex block number
                let hex = r.block_number.trim_start_matches("0x");
                i64::from_str_radix(hex, 16).ok()
            });

        // Compute bytecode hash
        let bytecode = artifact.bytecode.object.trim_start_matches("0x");
        let bytecode_bytes = hex::decode(bytecode).unwrap_or_default();
        let bytecode_hash = format!("{:x}", keccak256(&bytecode_bytes));

        // Serialize constructor args if present
        let constructor_args = tx
            .arguments
            .as_ref()
            .map(serde_json::to_string)
            .transpose()?;

        let source_path = format!("src/{}.sol:{}", contract_name, contract_name);
        deployments.push(ParsedDeployment {
            contract_name,
            address,
            deployer: tx.transaction.from.clone(),
            tx_hash: tx.hash.clone(),
            block_number,
            constructor_args,
            abi: serde_json::to_string(&artifact.abi)?,
            bytecode_hash,
            source_path,
        });
    }

    Ok(deployments)
}

#[cfg(test)]
mod tests {
    use super::*;

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
        assert_eq!(tx.transaction_type, "CREATE");
        assert_eq!(tx.contract_name, Some("MyToken".to_string()));
        assert_eq!(
            tx.contract_address,
            Some("0xabcdef1234567890abcdef1234567890abcdef12".to_string())
        );
        assert_eq!(
            tx.transaction.from,
            "0x1111111111111111111111111111111111111111"
        );

        let receipt = &output.receipts[0];
        assert_eq!(receipt.block_number, "0x10");
    }

    #[test]
    fn test_parse_contract_artifact() {
        let json = r#"{
            "abi": [
                {
                    "type": "function",
                    "name": "transfer",
                    "inputs": [
                        {"name": "to", "type": "address"},
                        {"name": "amount", "type": "uint256"}
                    ],
                    "outputs": [{"type": "bool"}]
                }
            ],
            "bytecode": {
                "object": "0x6080604052348015600f57600080fd5b50"
            },
            "deployedBytecode": {
                "object": "0x6080604052"
            }
        }"#;

        let artifact: ContractArtifact = serde_json::from_str(json).unwrap();

        assert!(artifact.abi.is_array());
        assert_eq!(artifact.abi.as_array().unwrap().len(), 1);
        assert!(artifact
            .bytecode
            .object
            .starts_with("0x6080604052348015600f57600080fd5b50"));
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

        // First is CALL, second is CREATE
        assert_eq!(output.transactions[0].transaction_type, "CALL");
        assert_eq!(output.transactions[1].transaction_type, "CREATE");
        assert_eq!(
            output.transactions[1].contract_name,
            Some("Token".to_string())
        );
    }

    #[test]
    fn test_parse_hex_block_number() {
        // Test hex parsing logic used in extract_deployments
        let hex_str = "0x1a2b3c";
        let hex = hex_str.trim_start_matches("0x");
        let block_number = i64::from_str_radix(hex, 16).unwrap();

        assert_eq!(block_number, 1715004);
    }

    #[test]
    fn test_bytecode_hash_computation() {
        let bytecode = "6080604052";
        let bytecode_bytes = hex::decode(bytecode).unwrap();
        let hash = format!("{:x}", keccak256(&bytecode_bytes));

        // Hash should be 64 hex chars (32 bytes)
        assert_eq!(hash.len(), 64);
        // Should be deterministic
        let hash2 = format!("{:x}", keccak256(&bytecode_bytes));
        assert_eq!(hash, hash2);
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
        // Test the logic for stripping :ContractName from script path
        let with_contract = "script/Deploy.s.sol:Deploy";
        let stripped = with_contract.split(':').next().unwrap();
        assert_eq!(stripped, "script/Deploy.s.sol");

        let without_contract = "script/Deploy.s.sol";
        let stripped = without_contract.split(':').next().unwrap();
        assert_eq!(stripped, "script/Deploy.s.sol");

        // Test extracting filename
        let script_name = Path::new(stripped).file_name().unwrap().to_str().unwrap();
        assert_eq!(script_name, "Deploy.s.sol");
    }
}
