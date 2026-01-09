use serde::{Deserialize, Serialize};
use sqlx::Type;
use std::fmt;

/// Re-export alloy types for convenience
pub use alloy::primitives::{Address, B256};

// =============================================================================
// Domain Enums
// =============================================================================

/// Type of contract call (read-only or state-changing)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(rename_all = "lowercase")]
pub enum CallType {
    Read,
    Write,
}

impl fmt::Display for CallType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CallType::Read => write!(f, "read"),
            CallType::Write => write!(f, "write"),
        }
    }
}

impl CallType {
    pub fn as_str(&self) -> &'static str {
        match self {
            CallType::Read => "read",
            CallType::Write => "write",
        }
    }
}

/// Status of a transaction
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[serde(rename_all = "lowercase")]
#[sqlx(rename_all = "lowercase")]
pub enum TransactionStatus {
    Pending,
    Success,
    Failed,
    Reverted,
}

impl fmt::Display for TransactionStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TransactionStatus::Pending => write!(f, "pending"),
            TransactionStatus::Success => write!(f, "success"),
            TransactionStatus::Failed => write!(f, "failed"),
            TransactionStatus::Reverted => write!(f, "reverted"),
        }
    }
}

impl TransactionStatus {
    pub fn as_str(&self) -> &'static str {
        match self {
            TransactionStatus::Pending => "pending",
            TransactionStatus::Success => "success",
            TransactionStatus::Failed => "failed",
            TransactionStatus::Reverted => "reverted",
        }
    }
}

/// State mutability of a contract function
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StateMutability {
    Pure,
    View,
    NonPayable,
    Payable,
}

impl StateMutability {
    /// Returns true if this function does not modify state
    pub fn is_read_only(&self) -> bool {
        matches!(self, StateMutability::Pure | StateMutability::View)
    }

    /// Returns true if this function can receive ETH
    pub fn is_payable(&self) -> bool {
        matches!(self, StateMutability::Payable)
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            StateMutability::Pure => "pure",
            StateMutability::View => "view",
            StateMutability::NonPayable => "nonpayable",
            StateMutability::Payable => "payable",
        }
    }
}

impl fmt::Display for StateMutability {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

// =============================================================================
// ID Newtypes
// =============================================================================

/// Network identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[sqlx(transparent)]
pub struct NetworkId(pub i64);

impl fmt::Display for NetworkId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<i64> for NetworkId {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl From<NetworkId> for i64 {
    fn from(value: NetworkId) -> Self {
        value.0
    }
}

/// Contract identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[sqlx(transparent)]
pub struct ContractId(pub i64);

impl fmt::Display for ContractId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<i64> for ContractId {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl From<ContractId> for i64 {
    fn from(value: ContractId) -> Self {
        value.0
    }
}

/// Deployment identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[sqlx(transparent)]
pub struct DeploymentId(pub i64);

impl fmt::Display for DeploymentId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<i64> for DeploymentId {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl From<DeploymentId> for i64 {
    fn from(value: DeploymentId) -> Self {
        value.0
    }
}

/// Wallet identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[sqlx(transparent)]
pub struct WalletId(pub i64);

impl fmt::Display for WalletId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<i64> for WalletId {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl From<WalletId> for i64 {
    fn from(value: WalletId) -> Self {
        value.0
    }
}

/// Chain ID wrapper
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Type)]
#[sqlx(transparent)]
pub struct ChainId(pub i64);

impl fmt::Display for ChainId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl From<u64> for ChainId {
    fn from(value: u64) -> Self {
        Self(value as i64)
    }
}

impl From<ChainId> for u64 {
    fn from(value: ChainId) -> Self {
        value.0 as u64
    }
}

impl From<i64> for ChainId {
    fn from(value: i64) -> Self {
        Self(value)
    }
}

impl From<ChainId> for i64 {
    fn from(value: ChainId) -> Self {
        value.0
    }
}
