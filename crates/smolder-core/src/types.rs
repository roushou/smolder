use serde::{Deserialize, Serialize};

/// Re-export alloy types for convenience
pub use alloy::primitives::{Address, B256};

/// Network identifier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkId(pub i64);

/// Contract identifier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractId(pub i64);

/// Deployment identifier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeploymentId(pub i64);

/// Chain ID wrapper
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ChainId(pub u64);

impl From<u64> for ChainId {
    fn from(value: u64) -> Self {
        Self(value)
    }
}

impl From<ChainId> for u64 {
    fn from(value: ChainId) -> Self {
        value.0
    }
}
