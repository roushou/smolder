//! SQLite database implementation for Smolder
//!
//! This crate provides the [`Database`] struct which implements all repository
//! traits, backed by SQLite.
//!
//! # Usage
//!
//! Use the repository traits for all database operations:
//!
//! ```rust,ignore
//! use smolder_db::{Database, NetworkRepository, NewNetwork};
//!
//! let db = Database::connect().await?;
//! let network = NetworkRepository::upsert(&db, &NewNetwork { ... }).await?;
//! ```

pub mod models;
mod repositories;
mod schema;
pub mod traits;

// Re-export models for convenience
pub use models::*;

// Re-export traits for convenience
pub use traits::*;

// Re-export types from smolder-core for convenience
pub use smolder_core::types::{
    CallType, ChainId, ContractId, DeploymentId, NetworkId, TransactionStatus, WalletId,
};

use smolder_core::Result;
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::str::FromStr;

const DEFAULT_DB_FILE: &str = "smolder.db";

/// SQLite database connection and repository implementation
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// Connect to the default database file (smolder.db)
    pub async fn connect() -> Result<Self> {
        Self::connect_to(DEFAULT_DB_FILE).await
    }

    /// Connect to a specific database file
    pub async fn connect_to(path: &str) -> Result<Self> {
        let options = SqliteConnectOptions::from_str(path)
            .map_err(smolder_core::Error::Database)?
            .create_if_missing(true)
            .foreign_keys(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(5)
            .connect_with(options)
            .await?;

        Ok(Self { pool })
    }

    /// Initialize the database schema
    pub async fn init_schema(&self) -> Result<()> {
        schema::init_schema(&self.pool).await?;
        Ok(())
    }

    /// Get a reference to the underlying connection pool
    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{NewContract, NewDeployment, NewNetwork};
    use crate::traits::{
        ContractRepository, DeploymentFilter, DeploymentRepository, NetworkRepository,
    };

    async fn setup_test_db() -> Database {
        let db = Database::connect_to(":memory:").await.unwrap();
        db.init_schema().await.unwrap();
        db
    }

    #[tokio::test]
    async fn test_upsert_and_get_network() {
        let db = setup_test_db().await;

        let network = NewNetwork {
            name: "tempo-testnet".to_string(),
            chain_id: ChainId(240240),
            rpc_url: "https://rpc.testnet.tempo.xyz".to_string(),
            explorer_url: Some("https://testnet.tempotestnetscan.io".to_string()),
        };

        let created = NetworkRepository::upsert(&db, &network).await.unwrap();
        assert!(created.id.0 > 0);

        let fetched = NetworkRepository::get_by_name(&db, "tempo-testnet")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(fetched.name, "tempo-testnet");
        assert_eq!(fetched.chain_id, ChainId(240240));
        assert_eq!(fetched.rpc_url, "https://rpc.testnet.tempo.xyz");
    }

    #[tokio::test]
    async fn test_upsert_network_updates_existing() {
        let db = setup_test_db().await;

        let network1 = NewNetwork {
            name: "tempo".to_string(),
            chain_id: ChainId(100),
            rpc_url: "https://old.rpc".to_string(),
            explorer_url: None,
        };

        let created1 = NetworkRepository::upsert(&db, &network1).await.unwrap();

        let network2 = NewNetwork {
            name: "tempo".to_string(),
            chain_id: ChainId(200),
            rpc_url: "https://new.rpc".to_string(),
            explorer_url: Some("https://explorer.xyz".to_string()),
        };

        let created2 = NetworkRepository::upsert(&db, &network2).await.unwrap();

        // Should return same ID (upsert)
        assert_eq!(created1.id, created2.id);

        let fetched = NetworkRepository::get_by_name(&db, "tempo")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(fetched.chain_id, ChainId(200));
        assert_eq!(fetched.rpc_url, "https://new.rpc");
    }

    #[tokio::test]
    async fn test_list_networks() {
        let db = setup_test_db().await;

        NetworkRepository::upsert(
            &db,
            &NewNetwork {
                name: "alpha".to_string(),
                chain_id: ChainId(1),
                rpc_url: "https://alpha".to_string(),
                explorer_url: None,
            },
        )
        .await
        .unwrap();

        NetworkRepository::upsert(
            &db,
            &NewNetwork {
                name: "beta".to_string(),
                chain_id: ChainId(2),
                rpc_url: "https://beta".to_string(),
                explorer_url: None,
            },
        )
        .await
        .unwrap();

        let networks = NetworkRepository::list(&db).await.unwrap();
        assert_eq!(networks.len(), 2);
        assert_eq!(networks[0].name, "alpha");
        assert_eq!(networks[1].name, "beta");
    }

    #[tokio::test]
    async fn test_upsert_and_get_contract() {
        let db = setup_test_db().await;

        let contract = NewContract {
            name: "MyToken".to_string(),
            source_path: "src/MyToken.sol:MyToken".to_string(),
            abi: r#"[{"type":"function","name":"transfer"}]"#.to_string(),
            bytecode_hash: "0xabc123".to_string(),
        };

        let created = ContractRepository::upsert(&db, &contract).await.unwrap();
        assert!(created.id.0 > 0);

        let fetched = ContractRepository::get_by_name(&db, "MyToken")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(fetched.name, "MyToken");
        assert_eq!(fetched.source_path, "src/MyToken.sol:MyToken");
    }

    #[tokio::test]
    async fn test_create_deployment_increments_version() {
        let db = setup_test_db().await;

        let network = NetworkRepository::upsert(
            &db,
            &NewNetwork {
                name: "testnet".to_string(),
                chain_id: ChainId(1),
                rpc_url: "https://rpc".to_string(),
                explorer_url: None,
            },
        )
        .await
        .unwrap();

        let contract = ContractRepository::upsert(
            &db,
            &NewContract {
                name: "Token".to_string(),
                source_path: "src/Token.sol".to_string(),
                abi: "[]".to_string(),
                bytecode_hash: "0x123".to_string(),
            },
        )
        .await
        .unwrap();

        // First deployment
        DeploymentRepository::create(
            &db,
            &NewDeployment {
                contract_id: contract.id,
                network_id: network.id,
                address: "0xaaa".to_string(),
                deployer: "0xddd".to_string(),
                tx_hash: "0x111".to_string(),
                block_number: Some(100),
                constructor_args: None,
            },
        )
        .await
        .unwrap();

        // Second deployment
        DeploymentRepository::create(
            &db,
            &NewDeployment {
                contract_id: contract.id,
                network_id: network.id,
                address: "0xbbb".to_string(),
                deployer: "0xddd".to_string(),
                tx_hash: "0x222".to_string(),
                block_number: Some(200),
                constructor_args: None,
            },
        )
        .await
        .unwrap();

        let current = DeploymentRepository::get_current(&db, "Token", "testnet")
            .await
            .unwrap()
            .unwrap();

        assert_eq!(current.address, "0xbbb");
        assert_eq!(current.version, 2);
        assert!(current.is_current);
    }

    #[tokio::test]
    async fn test_list_deployments_filters_by_network() {
        let db = setup_test_db().await;

        let net1 = NetworkRepository::upsert(
            &db,
            &NewNetwork {
                name: "net1".to_string(),
                chain_id: ChainId(1),
                rpc_url: "https://net1".to_string(),
                explorer_url: None,
            },
        )
        .await
        .unwrap();

        let net2 = NetworkRepository::upsert(
            &db,
            &NewNetwork {
                name: "net2".to_string(),
                chain_id: ChainId(2),
                rpc_url: "https://net2".to_string(),
                explorer_url: None,
            },
        )
        .await
        .unwrap();

        let contract = ContractRepository::upsert(
            &db,
            &NewContract {
                name: "Token".to_string(),
                source_path: "src/Token.sol".to_string(),
                abi: "[]".to_string(),
                bytecode_hash: "0x123".to_string(),
            },
        )
        .await
        .unwrap();

        DeploymentRepository::create(
            &db,
            &NewDeployment {
                contract_id: contract.id,
                network_id: net1.id,
                address: "0x111".to_string(),
                deployer: "0xddd".to_string(),
                tx_hash: "0xaaa".to_string(),
                block_number: None,
                constructor_args: None,
            },
        )
        .await
        .unwrap();

        DeploymentRepository::create(
            &db,
            &NewDeployment {
                contract_id: contract.id,
                network_id: net2.id,
                address: "0x222".to_string(),
                deployer: "0xddd".to_string(),
                tx_hash: "0xbbb".to_string(),
                block_number: None,
                constructor_args: None,
            },
        )
        .await
        .unwrap();

        // List all current
        let all = DeploymentRepository::list(&db, DeploymentFilter::current())
            .await
            .unwrap();
        assert_eq!(all.len(), 2);

        // Filter by net1
        let net1_only = DeploymentRepository::list(&db, DeploymentFilter::for_network("net1"))
            .await
            .unwrap();
        assert_eq!(net1_only.len(), 1);
        assert_eq!(net1_only[0].network_name, "net1");

        // Filter by net2
        let net2_only = DeploymentRepository::list(&db, DeploymentFilter::for_network("net2"))
            .await
            .unwrap();
        assert_eq!(net2_only.len(), 1);
        assert_eq!(net2_only[0].network_name, "net2");
    }

    #[tokio::test]
    async fn test_get_current_deployment_not_found() {
        let db = setup_test_db().await;

        let result = DeploymentRepository::get_current(&db, "NonExistent", "testnet")
            .await
            .unwrap();

        assert!(result.is_none());
    }
}
