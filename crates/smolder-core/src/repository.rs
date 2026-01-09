//! Repository traits for data access abstraction
//!
//! These traits define the interface for database operations, enabling:
//! - Separation of business logic from data access
//! - Easy testing with mock implementations
//! - Potential for different storage backends (SQLite, Postgres, etc.)

use async_trait::async_trait;

use crate::error::Result;
use crate::models::{
    CallHistory, CallHistoryUpdate, CallHistoryView, Contract, Deployment, DeploymentView, Network,
    NewCallHistory, NewContract, NewDeployment, NewNetwork, NewWallet, Wallet, WalletWithKey,
};
use crate::types::{ChainId, ContractId, DeploymentId, NetworkId, WalletId};

// =============================================================================
// Filter Types
// =============================================================================

/// Filter for listing deployments
#[derive(Debug, Default, Clone)]
pub struct DeploymentFilter {
    /// Filter by network name
    pub network: Option<String>,
    /// Filter by contract name
    pub contract: Option<String>,
    /// Only include current (latest) deployments
    pub current_only: bool,
}

impl DeploymentFilter {
    /// Create a filter for a specific network
    pub fn for_network(network: impl Into<String>) -> Self {
        Self {
            network: Some(network.into()),
            current_only: true,
            ..Default::default()
        }
    }

    /// Create a filter for current deployments only
    pub fn current() -> Self {
        Self {
            current_only: true,
            ..Default::default()
        }
    }
}

/// Filter for listing call history
#[derive(Debug, Default, Clone)]
pub struct CallHistoryFilter {
    /// Filter by deployment ID
    pub deployment_id: Option<DeploymentId>,
    /// Limit number of results
    pub limit: Option<u32>,
}

// =============================================================================
// Repository Traits
// =============================================================================

/// Repository for network operations
#[async_trait]
pub trait NetworkRepository: Send + Sync {
    /// List all networks
    async fn list(&self) -> Result<Vec<Network>>;

    /// Get a network by name
    async fn get_by_name(&self, name: &str) -> Result<Option<Network>>;

    /// Get a network by ID
    async fn get_by_id(&self, id: NetworkId) -> Result<Option<Network>>;

    /// Get a network by chain ID
    async fn get_by_chain_id(&self, chain_id: ChainId) -> Result<Option<Network>>;

    /// Insert or update a network
    async fn upsert(&self, network: &NewNetwork) -> Result<Network>;
}

/// Repository for contract operations
#[async_trait]
pub trait ContractRepository: Send + Sync {
    /// List all contracts
    async fn list(&self) -> Result<Vec<Contract>>;

    /// Get a contract by name
    async fn get_by_name(&self, name: &str) -> Result<Option<Contract>>;

    /// Get a contract by ID
    async fn get_by_id(&self, id: ContractId) -> Result<Option<Contract>>;

    /// Insert or update a contract
    async fn upsert(&self, contract: &NewContract) -> Result<Contract>;
}

/// Repository for deployment operations
#[async_trait]
pub trait DeploymentRepository: Send + Sync {
    /// List deployments with optional filtering
    async fn list(&self, filter: DeploymentFilter) -> Result<Vec<DeploymentView>>;

    /// Get the current deployment for a contract on a network
    async fn get_current(&self, contract: &str, network: &str) -> Result<Option<Deployment>>;

    /// Get a deployment by ID
    async fn get_by_id(&self, id: DeploymentId) -> Result<Option<Deployment>>;

    /// Get a deployment view by ID (includes contract and network info)
    async fn get_view_by_id(&self, id: DeploymentId) -> Result<Option<DeploymentView>>;

    /// Check if a deployment exists by transaction hash
    async fn exists_by_tx_hash(&self, tx_hash: &str) -> Result<bool>;

    /// Create a new deployment (handles versioning automatically)
    async fn create(&self, deployment: &NewDeployment) -> Result<Deployment>;

    /// Get all deployments for export (regardless of current status)
    async fn list_for_export(&self, network: Option<&str>) -> Result<Vec<DeploymentView>>;
}

/// Repository for wallet operations
#[async_trait]
pub trait WalletRepository: Send + Sync {
    /// List all wallets (without encrypted keys)
    async fn list(&self) -> Result<Vec<Wallet>>;

    /// Get a wallet by name (without encrypted key)
    async fn get_by_name(&self, name: &str) -> Result<Option<Wallet>>;

    /// Get a wallet by name with encrypted key (for internal use)
    async fn get_with_key(&self, name: &str) -> Result<Option<WalletWithKey>>;

    /// Get a wallet by ID
    async fn get_by_id(&self, id: WalletId) -> Result<Option<Wallet>>;

    /// Get a wallet by address
    async fn get_by_address(&self, address: &str) -> Result<Option<Wallet>>;

    /// Create a new wallet
    async fn create(&self, wallet: &NewWallet) -> Result<Wallet>;

    /// Delete a wallet by name
    async fn delete(&self, name: &str) -> Result<()>;
}

/// Repository for call history operations
#[async_trait]
pub trait CallHistoryRepository: Send + Sync {
    /// List call history with optional filtering
    async fn list(&self, filter: CallHistoryFilter) -> Result<Vec<CallHistory>>;

    /// List call history with full view (joined with deployment, contract, network, wallet)
    async fn list_views(&self, filter: CallHistoryFilter) -> Result<Vec<CallHistoryView>>;

    /// Get a call history entry by ID
    async fn get_by_id(&self, id: i64) -> Result<Option<CallHistory>>;

    /// Create a new call history entry
    async fn create(&self, entry: &NewCallHistory) -> Result<CallHistory>;

    /// Update a call history entry after execution
    async fn update(&self, id: i64, update: &CallHistoryUpdate) -> Result<()>;
}

// =============================================================================
// Aggregate Repository (for convenience)
// =============================================================================

/// Combined repository providing access to all entity repositories
///
/// This is useful for handlers that need access to multiple repositories
pub trait Repositories: Send + Sync {
    /// Access the network repository
    fn networks(&self) -> &dyn NetworkRepository;

    /// Access the contract repository
    fn contracts(&self) -> &dyn ContractRepository;

    /// Access the deployment repository
    fn deployments(&self) -> &dyn DeploymentRepository;

    /// Access the wallet repository
    fn wallets(&self) -> &dyn WalletRepository;

    /// Access the call history repository
    fn call_history(&self) -> &dyn CallHistoryRepository;
}
