//! Type definitions for forge artifacts and broadcast outputs

use serde::{Deserialize, Serialize};
use smolder_core::ConstructorInfo;

// =============================================================================
// Broadcast Types
// =============================================================================

/// Represents the broadcast output from forge script
#[derive(Debug, Deserialize)]
pub struct BroadcastOutput {
    pub transactions: Vec<BroadcastTransaction>,
    pub receipts: Vec<BroadcastReceipt>,
}

/// A transaction from the broadcast output
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

impl BroadcastTransaction {
    /// Check if this is a CREATE transaction
    pub fn is_create(&self) -> bool {
        self.transaction_type == "CREATE"
    }

    /// Check if this transaction has complete deployment info
    pub fn has_deployment_info(&self) -> bool {
        self.contract_name.is_some() && self.contract_address.is_some()
    }
}

/// Transaction data within a broadcast transaction
#[derive(Debug, Deserialize)]
pub struct TransactionData {
    pub from: String,
    #[allow(dead_code)]
    pub data: Option<String>,
}

/// A receipt from the broadcast output
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BroadcastReceipt {
    pub transaction_hash: String,
    pub block_number: String,
    #[allow(dead_code)]
    pub contract_address: Option<String>,
}

// =============================================================================
// Artifact Types
// =============================================================================

/// Represents a contract artifact from forge build output
#[derive(Debug, Deserialize)]
pub struct ContractArtifact {
    pub abi: serde_json::Value,
    pub bytecode: BytecodeObject,
    #[serde(rename = "deployedBytecode")]
    #[allow(dead_code)]
    pub deployed_bytecode: BytecodeObject,
}

/// Bytecode object within an artifact
#[derive(Debug, Deserialize)]
pub struct BytecodeObject {
    pub object: String,
}

impl BytecodeObject {
    /// Check if this bytecode is valid (non-empty)
    pub fn is_valid(&self) -> bool {
        !self.object.is_empty() && self.object != "0x"
    }

    /// Get the bytecode without 0x prefix
    pub fn without_prefix(&self) -> &str {
        self.object.trim_start_matches("0x")
    }
}

/// Extended contract artifact with AST for source path detection
#[derive(Debug, Deserialize)]
pub struct ContractArtifactFull {
    pub abi: serde_json::Value,
    pub bytecode: BytecodeObject,
    #[serde(default)]
    #[allow(dead_code)]
    pub ast: Option<serde_json::Value>,
}

// =============================================================================
// Artifact Info Types
// =============================================================================

/// Information about a compiled artifact (for listing)
#[derive(Debug, Clone, Serialize)]
pub struct ArtifactInfo {
    pub name: String,
    pub source_path: String,
    pub has_constructor: bool,
    pub has_bytecode: bool,
}

/// Detailed artifact information for deployment
#[derive(Debug, Clone, Serialize)]
pub struct ArtifactDetails {
    pub name: String,
    pub source_path: String,
    pub abi: serde_json::Value,
    pub constructor: Option<ConstructorInfo>,
    pub has_bytecode: bool,
}

// =============================================================================
// Deployment Types
// =============================================================================

/// Parsed deployment information from broadcast + artifacts
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
