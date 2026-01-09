use crate::types::DeploymentId;
use thiserror::Error;

/// Result type alias using the crate's Error type
pub type Result<T> = std::result::Result<T, Error>;

#[derive(Error, Debug)]
pub enum Error {
    // =========================================================================
    // External errors (with automatic From implementations)
    // =========================================================================
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Hex decode error: {0}")]
    HexDecode(#[from] hex::FromHexError),

    // =========================================================================
    // Entity not found errors
    // =========================================================================
    #[error("Network not found: {0}")]
    NetworkNotFound(String),

    #[error("Contract not found: {0}")]
    ContractNotFound(String),

    #[error("Deployment not found: {0}")]
    DeploymentNotFound(String),

    #[error("Deployment not found by ID: {0}")]
    DeploymentNotFoundById(DeploymentId),

    #[error("Wallet not found: {0}")]
    WalletNotFound(String),

    #[error("Function '{function}' not found in contract '{contract}'")]
    FunctionNotFound { contract: String, function: String },

    #[error("Artifact not found: {0}")]
    ArtifactNotFound(String),

    // =========================================================================
    // ABI errors
    // =========================================================================
    #[error("ABI parse error: {0}")]
    AbiParse(String),

    #[error("ABI encoding error: {0}")]
    AbiEncode(String),

    #[error("ABI decoding error: {0}")]
    AbiDecode(String),

    // =========================================================================
    // RPC/Chain errors
    // =========================================================================
    #[error("RPC error: {0}")]
    Rpc(String),

    #[error("RPC error on chain {chain_id}: {message}")]
    RpcWithChain { chain_id: u64, message: String },

    #[error("Transaction failed: {0}")]
    TransactionFailed(String),

    #[error("Transaction reverted: {reason}")]
    TransactionReverted {
        reason: String,
        tx_hash: Option<String>,
    },

    // =========================================================================
    // Validation errors
    // =========================================================================
    #[error("Invalid parameter '{name}': {reason}")]
    InvalidParameter { name: String, reason: String },

    #[error("Validation error: {0}")]
    Validation(String),

    // =========================================================================
    // Cryptography errors
    // =========================================================================
    #[error("Keyring error: {0}")]
    Keyring(String),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("Decryption error: {0}")]
    Decryption(String),

    // =========================================================================
    // IO errors
    // =========================================================================
    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("IO error: {0}")]
    Io(String),

    // =========================================================================
    // Configuration errors
    // =========================================================================
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Environment variable '{name}' not set")]
    EnvVarNotSet { name: String },
}

impl Error {
    /// Returns true if this is a "not found" type error
    pub fn is_not_found(&self) -> bool {
        matches!(
            self,
            Error::NetworkNotFound(_)
                | Error::ContractNotFound(_)
                | Error::DeploymentNotFound(_)
                | Error::DeploymentNotFoundById(_)
                | Error::WalletNotFound(_)
                | Error::FunctionNotFound { .. }
                | Error::ArtifactNotFound(_)
                | Error::FileNotFound(_)
        )
    }

    /// Returns true if this is a database error
    pub fn is_database(&self) -> bool {
        matches!(self, Error::Database(_))
    }

    /// Returns true if this is a validation error
    pub fn is_validation(&self) -> bool {
        matches!(self, Error::InvalidParameter { .. } | Error::Validation(_))
    }

    /// Returns an error code suitable for API responses
    pub fn code(&self) -> &'static str {
        match self {
            Error::Database(_) => "DATABASE_ERROR",
            Error::Serialization(_) => "SERIALIZATION_ERROR",
            Error::HexDecode(_) => "HEX_DECODE_ERROR",
            Error::NetworkNotFound(_) => "NETWORK_NOT_FOUND",
            Error::ContractNotFound(_) => "CONTRACT_NOT_FOUND",
            Error::DeploymentNotFound(_) | Error::DeploymentNotFoundById(_) => {
                "DEPLOYMENT_NOT_FOUND"
            }
            Error::WalletNotFound(_) => "WALLET_NOT_FOUND",
            Error::FunctionNotFound { .. } => "FUNCTION_NOT_FOUND",
            Error::ArtifactNotFound(_) => "ARTIFACT_NOT_FOUND",
            Error::AbiParse(_) => "ABI_PARSE_ERROR",
            Error::AbiEncode(_) => "ABI_ENCODE_ERROR",
            Error::AbiDecode(_) => "ABI_DECODE_ERROR",
            Error::Rpc(_) | Error::RpcWithChain { .. } => "RPC_ERROR",
            Error::TransactionFailed(_) => "TRANSACTION_FAILED",
            Error::TransactionReverted { .. } => "TRANSACTION_REVERTED",
            Error::InvalidParameter { .. } => "INVALID_PARAMETER",
            Error::Validation(_) => "VALIDATION_ERROR",
            Error::Keyring(_) => "KEYRING_ERROR",
            Error::Encryption(_) => "ENCRYPTION_ERROR",
            Error::Decryption(_) => "DECRYPTION_ERROR",
            Error::FileNotFound(_) => "FILE_NOT_FOUND",
            Error::Io(_) => "IO_ERROR",
            Error::Config(_) => "CONFIG_ERROR",
            Error::EnvVarNotSet { .. } => "ENV_VAR_NOT_SET",
        }
    }
}

// Convenience constructors for common error patterns
impl Error {
    pub fn function_not_found(contract: impl Into<String>, function: impl Into<String>) -> Self {
        Error::FunctionNotFound {
            contract: contract.into(),
            function: function.into(),
        }
    }

    pub fn invalid_param(name: impl Into<String>, reason: impl Into<String>) -> Self {
        Error::InvalidParameter {
            name: name.into(),
            reason: reason.into(),
        }
    }

    pub fn rpc_error(chain_id: u64, message: impl Into<String>) -> Self {
        Error::RpcWithChain {
            chain_id,
            message: message.into(),
        }
    }
}
