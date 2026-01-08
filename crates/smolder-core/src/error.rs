use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("Network not found: {0}")]
    NetworkNotFound(String),

    #[error("Contract not found: {0}")]
    ContractNotFound(String),

    #[error("Deployment not found: {0}")]
    DeploymentNotFound(String),

    #[error("Wallet not found: {0}")]
    WalletNotFound(String),

    #[error("Keyring error: {0}")]
    Keyring(String),

    #[error("ABI error: {0}")]
    Abi(String),

    #[error("RPC error: {0}")]
    Rpc(String),

    #[error("Transaction failed: {0}")]
    TransactionFailed(String),

    #[error("Invalid parameter: {0}")]
    InvalidParameter(String),

    #[error("Function not found: {0}")]
    FunctionNotFound(String),
}
