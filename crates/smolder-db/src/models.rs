//! Database entity models
//!
//! This module contains all the entity structs used for database operations,
//! including both read models (with `FromRow`) and write models (New* structs).

use serde::{Deserialize, Serialize};
use smolder_core::types::{
    CallType, ChainId, ContractId, DeploymentId, NetworkId, TransactionStatus, WalletId,
};
use sqlx::FromRow;

/// Network configuration stored in database
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Network {
    pub id: NetworkId,
    pub name: String,
    pub chain_id: ChainId,
    pub rpc_url: String,
    pub explorer_url: Option<String>,
    pub created_at: String,
}

/// Contract definition (source-level)
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Contract {
    pub id: ContractId,
    pub name: String,
    pub source_path: String,
    pub abi: String, // JSON string
    pub bytecode_hash: String,
    pub created_at: String,
}

/// Deployment instance on a chain
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Deployment {
    pub id: DeploymentId,
    pub contract_id: ContractId,
    pub network_id: NetworkId,
    pub address: String,
    pub deployer: String,
    pub tx_hash: String,
    pub block_number: Option<i64>,
    pub constructor_args: Option<String>, // JSON string
    pub version: i64,
    pub deployed_at: String,
    pub is_current: bool,
}

/// Joined view of deployment with contract and network info
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct DeploymentView {
    pub id: DeploymentId,
    pub contract_name: String,
    pub network_name: String,
    pub chain_id: ChainId,
    pub address: String,
    pub deployer: String,
    pub tx_hash: String,
    pub block_number: Option<i64>,
    pub version: i64,
    pub deployed_at: String,
    pub is_current: bool,
    pub abi: String,
}

/// Input for creating a new network
#[derive(Debug, Clone)]
pub struct NewNetwork {
    pub name: String,
    pub chain_id: ChainId,
    pub rpc_url: String,
    pub explorer_url: Option<String>,
}

/// Input for creating a new contract
#[derive(Debug, Clone)]
pub struct NewContract {
    pub name: String,
    pub source_path: String,
    pub abi: String,
    pub bytecode_hash: String,
}

/// Input for creating a new deployment
#[derive(Debug, Clone)]
pub struct NewDeployment {
    pub contract_id: ContractId,
    pub network_id: NetworkId,
    pub address: String,
    pub deployer: String,
    pub tx_hash: String,
    pub block_number: Option<i64>,
    pub constructor_args: Option<String>,
}

/// Wallet metadata (for listing without key)
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct Wallet {
    pub id: WalletId,
    pub name: String,
    pub address: String,
    pub created_at: String,
}

/// Wallet with encrypted private key (for internal use)
#[derive(Debug, Clone, FromRow)]
pub struct WalletWithKey {
    pub id: WalletId,
    pub name: String,
    pub address: String,
    pub encrypted_key: Vec<u8>,
    pub created_at: String,
}

/// Input for creating a new wallet
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewWallet {
    pub name: String,
    pub address: String,
    pub encrypted_key: Vec<u8>,
}

/// Call history entry
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct CallHistory {
    pub id: i64,
    pub deployment_id: DeploymentId,
    pub wallet_id: Option<WalletId>,
    pub function_name: String,
    pub function_signature: String,
    pub input_params: String,   // JSON
    pub call_type: CallType,    // Read or Write
    pub result: Option<String>, // JSON for read results
    pub tx_hash: Option<String>,
    pub block_number: Option<i64>,
    pub gas_used: Option<i64>,
    pub gas_price: Option<String>,
    pub status: Option<TransactionStatus>, // Pending, Success, Failed, Reverted
    pub error_message: Option<String>,
    pub created_at: String,
    pub confirmed_at: Option<String>,
}

/// Joined view of call history with deployment and wallet info
#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
pub struct CallHistoryView {
    pub id: i64,
    pub deployment_id: DeploymentId,
    pub contract_name: String,
    pub network_name: String,
    pub contract_address: String,
    pub wallet_name: Option<String>,
    pub function_name: String,
    pub function_signature: String,
    pub input_params: String,
    pub call_type: CallType,
    pub result: Option<String>,
    pub tx_hash: Option<String>,
    pub block_number: Option<i64>,
    pub gas_used: Option<i64>,
    pub gas_price: Option<String>,
    pub status: Option<TransactionStatus>,
    pub error_message: Option<String>,
    pub created_at: String,
    pub confirmed_at: Option<String>,
}

/// Input for creating a new call history record
#[derive(Debug, Clone)]
pub struct NewCallHistory {
    pub deployment_id: DeploymentId,
    pub wallet_id: Option<WalletId>,
    pub function_name: String,
    pub function_signature: String,
    pub input_params: String,
    pub call_type: CallType,
}

/// Update for call history after execution
#[derive(Debug, Clone)]
pub struct CallHistoryUpdate {
    pub result: Option<String>,
    pub tx_hash: Option<String>,
    pub block_number: Option<i64>,
    pub gas_used: Option<i64>,
    pub gas_price: Option<String>,
    pub status: TransactionStatus,
    pub error_message: Option<String>,
}
