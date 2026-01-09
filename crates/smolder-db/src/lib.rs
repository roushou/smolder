//! SQLite database implementation for Smolder
//!
//! This crate provides the [`Database`] struct which implements all repository
//! traits from `smolder-core`, backed by SQLite.
//!
//! # Usage
//!
//! The Database struct can be used in two ways:
//!
//! 1. **Convenience methods** - Simple methods like `upsert_network`, `list_deployments`
//!    that return plain types (e.g., `i64` for IDs). Good for CLI commands.
//!
//! 2. **Repository traits** - Implement the repository traits from `smolder-core` for
//!    more structured access. Good for API servers.

pub mod models;
mod repositories;
mod schema;
pub mod traits;

// Re-export models for convenience
pub use models::*;

// Re-export traits for convenience
pub use traits::*;

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

    // =========================================================================
    // Convenience Methods
    // =========================================================================
    // These methods provide a simpler API for CLI commands, returning plain
    // types like i64 for IDs instead of full entity structs.

    /// Insert or update a network, returning the network ID
    pub async fn upsert_network(&self, network: &NewNetwork) -> Result<i64> {
        let result = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO networks (name, chain_id, rpc_url, explorer_url)
            VALUES (?, ?, ?, ?)
            ON CONFLICT(name) DO UPDATE SET
                chain_id = excluded.chain_id,
                rpc_url = excluded.rpc_url,
                explorer_url = excluded.explorer_url
            RETURNING id
            "#,
        )
        .bind(&network.name)
        .bind(network.chain_id)
        .bind(&network.rpc_url)
        .bind(&network.explorer_url)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    /// Insert or update a contract, returning the contract ID
    pub async fn upsert_contract(&self, contract: &NewContract) -> Result<i64> {
        let result = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO contracts (name, source_path, abi, bytecode_hash)
            VALUES (?, ?, ?, ?)
            ON CONFLICT(name, bytecode_hash) DO UPDATE SET
                source_path = excluded.source_path,
                abi = excluded.abi
            RETURNING id
            "#,
        )
        .bind(&contract.name)
        .bind(&contract.source_path)
        .bind(&contract.abi)
        .bind(&contract.bytecode_hash)
        .fetch_one(&self.pool)
        .await?;

        Ok(result)
    }

    /// Create a new deployment, returning the deployment ID
    pub async fn create_deployment(&self, deployment: &NewDeployment) -> Result<i64> {
        // Mark previous deployments as not current
        sqlx::query(
            "UPDATE deployments SET is_current = FALSE WHERE contract_id = ? AND network_id = ?",
        )
        .bind(deployment.contract_id)
        .bind(deployment.network_id)
        .execute(&self.pool)
        .await?;

        // Get next version number
        let max_version: Option<i64> = sqlx::query_scalar(
            "SELECT MAX(version) FROM deployments WHERE contract_id = ? AND network_id = ?",
        )
        .bind(deployment.contract_id)
        .bind(deployment.network_id)
        .fetch_one(&self.pool)
        .await?;

        let next_version = max_version.unwrap_or(0) + 1;

        // Insert new deployment
        let id = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO deployments (contract_id, network_id, address, deployer, tx_hash, block_number, constructor_args, version, is_current)
            VALUES (?, ?, ?, ?, ?, ?, ?, ?, TRUE)
            RETURNING id
            "#,
        )
        .bind(deployment.contract_id)
        .bind(deployment.network_id)
        .bind(&deployment.address)
        .bind(&deployment.deployer)
        .bind(&deployment.tx_hash)
        .bind(deployment.block_number)
        .bind(&deployment.constructor_args)
        .bind(next_version)
        .fetch_one(&self.pool)
        .await?;

        Ok(id)
    }

    /// Get the current deployment view for a contract on a network
    pub async fn get_current_deployment(
        &self,
        contract_name: &str,
        network_name: &str,
    ) -> Result<Option<DeploymentView>> {
        let deployment = sqlx::query_as::<_, DeploymentView>(
            r#"
            SELECT
                d.id,
                c.name as contract_name,
                n.name as network_name,
                n.chain_id,
                d.address,
                d.deployer,
                d.tx_hash,
                d.block_number,
                d.version,
                d.deployed_at,
                d.is_current,
                c.abi
            FROM deployments d
            JOIN contracts c ON d.contract_id = c.id
            JOIN networks n ON d.network_id = n.id
            WHERE c.name = ? AND n.name = ? AND d.is_current = TRUE
            "#,
        )
        .bind(contract_name)
        .bind(network_name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(deployment)
    }

    /// List all current deployments, optionally filtered by network
    pub async fn list_deployments(&self, network: Option<&str>) -> Result<Vec<DeploymentView>> {
        let deployments = match network {
            Some(net) => {
                sqlx::query_as::<_, DeploymentView>(
                    r#"
                SELECT
                    d.id,
                    c.name as contract_name,
                    n.name as network_name,
                    n.chain_id,
                    d.address,
                    d.deployer,
                    d.tx_hash,
                    d.block_number,
                    d.version,
                    d.deployed_at,
                    d.is_current,
                    c.abi
                FROM deployments d
                JOIN contracts c ON d.contract_id = c.id
                JOIN networks n ON d.network_id = n.id
                WHERE n.name = ? AND d.is_current = TRUE
                ORDER BY n.name, c.name
                "#,
                )
                .bind(net)
                .fetch_all(&self.pool)
                .await?
            }
            None => {
                sqlx::query_as::<_, DeploymentView>(
                    r#"
                SELECT
                    d.id,
                    c.name as contract_name,
                    n.name as network_name,
                    n.chain_id,
                    d.address,
                    d.deployer,
                    d.tx_hash,
                    d.block_number,
                    d.version,
                    d.deployed_at,
                    d.is_current,
                    c.abi
                FROM deployments d
                JOIN contracts c ON d.contract_id = c.id
                JOIN networks n ON d.network_id = n.id
                WHERE d.is_current = TRUE
                ORDER BY n.name, c.name
                "#,
                )
                .fetch_all(&self.pool)
                .await?
            }
        };

        Ok(deployments)
    }

    /// Check if a deployment exists by transaction hash
    pub async fn deployment_exists_by_tx_hash(&self, tx_hash: &str) -> Result<bool> {
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM deployments WHERE tx_hash = ?)")
                .bind(tx_hash)
                .fetch_one(&self.pool)
                .await?;

        Ok(exists)
    }

    /// Get all current deployments for export
    pub async fn get_all_deployments_for_export(&self) -> Result<Vec<DeploymentView>> {
        self.list_deployments(None).await
    }

    /// Create a new wallet with encrypted private key, returning the wallet ID
    pub async fn create_wallet(&self, wallet: &NewWallet) -> Result<i64> {
        let id = sqlx::query_scalar::<_, i64>(
            "INSERT INTO wallets (name, address, encrypted_key) VALUES (?, ?, ?) RETURNING id",
        )
        .bind(&wallet.name)
        .bind(&wallet.address)
        .bind(&wallet.encrypted_key)
        .fetch_one(&self.pool)
        .await?;

        Ok(id)
    }

    /// Get a wallet by name (without private key)
    pub async fn get_wallet(&self, name: &str) -> Result<Option<Wallet>> {
        let wallet = sqlx::query_as::<_, Wallet>(
            "SELECT id, name, address, created_at FROM wallets WHERE name = ?",
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(wallet)
    }

    /// Get a wallet by address (without private key)
    pub async fn get_wallet_by_address(&self, address: &str) -> Result<Option<Wallet>> {
        let wallet = sqlx::query_as::<_, Wallet>(
            "SELECT id, name, address, created_at FROM wallets WHERE address = ?",
        )
        .bind(address)
        .fetch_optional(&self.pool)
        .await?;

        Ok(wallet)
    }

    /// List all wallets (without private keys)
    pub async fn list_wallets(&self) -> Result<Vec<Wallet>> {
        let wallets = sqlx::query_as::<_, Wallet>(
            "SELECT id, name, address, created_at FROM wallets ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(wallets)
    }

    /// Delete a wallet by name, returning true if a wallet was deleted
    pub async fn delete_wallet(&self, name: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM wallets WHERE name = ?")
            .bind(name)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
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
            chain_id: 240240,
            rpc_url: "https://rpc.testnet.tempo.xyz".to_string(),
            explorer_url: Some("https://testnet.tempotestnetscan.io".to_string()),
        };

        let created = NetworkRepository::upsert(&db, &network).await.unwrap();
        assert!(created.id > 0);

        let fetched = NetworkRepository::get_by_name(&db, "tempo-testnet")
            .await
            .unwrap()
            .unwrap();
        assert_eq!(fetched.name, "tempo-testnet");
        assert_eq!(fetched.chain_id, 240240);
        assert_eq!(fetched.rpc_url, "https://rpc.testnet.tempo.xyz");
    }

    #[tokio::test]
    async fn test_upsert_network_updates_existing() {
        let db = setup_test_db().await;

        let network1 = NewNetwork {
            name: "tempo".to_string(),
            chain_id: 100,
            rpc_url: "https://old.rpc".to_string(),
            explorer_url: None,
        };

        let created1 = NetworkRepository::upsert(&db, &network1).await.unwrap();

        let network2 = NewNetwork {
            name: "tempo".to_string(),
            chain_id: 200,
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
        assert_eq!(fetched.chain_id, 200);
        assert_eq!(fetched.rpc_url, "https://new.rpc");
    }

    #[tokio::test]
    async fn test_list_networks() {
        let db = setup_test_db().await;

        NetworkRepository::upsert(
            &db,
            &NewNetwork {
                name: "alpha".to_string(),
                chain_id: 1,
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
                chain_id: 2,
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
        assert!(created.id > 0);

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
                chain_id: 1,
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
                chain_id: 1,
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
                chain_id: 2,
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
