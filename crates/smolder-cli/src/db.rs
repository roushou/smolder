use color_eyre::eyre::Result;
use smolder_core::{
    schema, CallHistory, CallHistoryView, Contract, DeploymentView, Network, NewCallHistory,
    NewContract, NewDeployment, NewNetwork, NewWallet, Wallet,
};
use sqlx::sqlite::{SqliteConnectOptions, SqlitePool, SqlitePoolOptions};
use std::str::FromStr;

const DB_FILE: &str = "smolder.db";

/// Database connection wrapper
pub struct Database {
    pool: SqlitePool,
}

impl Database {
    /// Connect to the database, creating it if it doesn't exist
    pub async fn connect() -> Result<Self> {
        Self::connect_to(DB_FILE).await
    }

    /// Connect to a specific database file
    pub async fn connect_to(path: &str) -> Result<Self> {
        let options = SqliteConnectOptions::from_str(path)?
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

    // ---- Network operations ----

    /// Insert or update a network
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

    /// Get a network by name
    pub async fn get_network(&self, name: &str) -> Result<Option<Network>> {
        let network = sqlx::query_as::<_, Network>("SELECT * FROM networks WHERE name = ?")
            .bind(name)
            .fetch_optional(&self.pool)
            .await?;

        Ok(network)
    }

    /// Get all networks
    pub async fn list_networks(&self) -> Result<Vec<Network>> {
        let networks = sqlx::query_as::<_, Network>("SELECT * FROM networks ORDER BY name")
            .fetch_all(&self.pool)
            .await?;

        Ok(networks)
    }

    // ---- Contract operations ----

    /// Insert a contract or get existing one with same name and bytecode hash
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

    /// Get a contract by name
    pub async fn get_contract(&self, name: &str) -> Result<Option<Contract>> {
        let contract = sqlx::query_as::<_, Contract>(
            "SELECT * FROM contracts WHERE name = ? ORDER BY created_at DESC LIMIT 1",
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(contract)
    }

    // ---- Deployment operations ----

    /// Create a new deployment, handling version increment and is_current flag
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

    /// Get the current deployment for a contract on a network
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
        let query = match network {
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

        Ok(query)
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

    /// Get all current deployments grouped for export
    pub async fn get_all_deployments_for_export(&self) -> Result<Vec<DeploymentView>> {
        let deployments = sqlx::query_as::<_, DeploymentView>(
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
        .await?;

        Ok(deployments)
    }

    // ---- Wallet operations ----

    /// Create a new wallet
    pub async fn create_wallet(&self, wallet: &NewWallet) -> Result<i64> {
        let id = sqlx::query_scalar::<_, i64>(
            "INSERT INTO wallets (name, address) VALUES (?, ?) RETURNING id",
        )
        .bind(&wallet.name)
        .bind(&wallet.address)
        .fetch_one(&self.pool)
        .await?;

        Ok(id)
    }

    /// Get a wallet by name
    pub async fn get_wallet(&self, name: &str) -> Result<Option<Wallet>> {
        let wallet = sqlx::query_as::<_, Wallet>("SELECT * FROM wallets WHERE name = ?")
            .bind(name)
            .fetch_optional(&self.pool)
            .await?;

        Ok(wallet)
    }

    /// Get a wallet by address
    pub async fn get_wallet_by_address(&self, address: &str) -> Result<Option<Wallet>> {
        let wallet = sqlx::query_as::<_, Wallet>("SELECT * FROM wallets WHERE address = ?")
            .bind(address)
            .fetch_optional(&self.pool)
            .await?;

        Ok(wallet)
    }

    /// List all wallets
    pub async fn list_wallets(&self) -> Result<Vec<Wallet>> {
        let wallets = sqlx::query_as::<_, Wallet>("SELECT * FROM wallets ORDER BY name")
            .fetch_all(&self.pool)
            .await?;

        Ok(wallets)
    }

    /// Delete a wallet by name
    pub async fn delete_wallet(&self, name: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM wallets WHERE name = ?")
            .bind(name)
            .execute(&self.pool)
            .await?;

        Ok(result.rows_affected() > 0)
    }

    // ---- Call history operations ----

    /// Create a new call history entry
    pub async fn create_call_history(&self, call: &NewCallHistory) -> Result<i64> {
        let id = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO call_history (deployment_id, wallet_id, function_name, function_signature, input_params, call_type)
            VALUES (?, ?, ?, ?, ?, ?)
            RETURNING id
            "#,
        )
        .bind(call.deployment_id)
        .bind(call.wallet_id)
        .bind(&call.function_name)
        .bind(&call.function_signature)
        .bind(&call.input_params)
        .bind(&call.call_type)
        .fetch_one(&self.pool)
        .await?;

        Ok(id)
    }

    /// Update call history with result (for read calls)
    pub async fn update_call_history_result(&self, id: i64, result: &str) -> Result<()> {
        sqlx::query("UPDATE call_history SET result = ?, status = 'success' WHERE id = ?")
            .bind(result)
            .bind(id)
            .execute(&self.pool)
            .await?;

        Ok(())
    }

    /// Update call history with transaction result (for write calls)
    pub async fn update_call_history_tx(
        &self,
        id: i64,
        tx_hash: &str,
        status: &str,
        block_number: Option<i64>,
        gas_used: Option<i64>,
        gas_price: Option<&str>,
        error_message: Option<&str>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE call_history SET
                tx_hash = ?,
                status = ?,
                block_number = ?,
                gas_used = ?,
                gas_price = ?,
                error_message = ?,
                confirmed_at = CASE WHEN ? IN ('success', 'failed', 'reverted') THEN CURRENT_TIMESTAMP ELSE NULL END
            WHERE id = ?
            "#,
        )
        .bind(tx_hash)
        .bind(status)
        .bind(block_number)
        .bind(gas_used)
        .bind(gas_price)
        .bind(error_message)
        .bind(status)
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    /// Get call history for a deployment
    pub async fn get_deployment_call_history(
        &self,
        deployment_id: i64,
    ) -> Result<Vec<CallHistoryView>> {
        let history = sqlx::query_as::<_, CallHistoryView>(
            r#"
            SELECT
                h.id,
                h.deployment_id,
                c.name as contract_name,
                n.name as network_name,
                d.address as contract_address,
                w.name as wallet_name,
                h.function_name,
                h.function_signature,
                h.input_params,
                h.call_type,
                h.result,
                h.tx_hash,
                h.block_number,
                h.gas_used,
                h.gas_price,
                h.status,
                h.error_message,
                h.created_at,
                h.confirmed_at
            FROM call_history h
            JOIN deployments d ON h.deployment_id = d.id
            JOIN contracts c ON d.contract_id = c.id
            JOIN networks n ON d.network_id = n.id
            LEFT JOIN wallets w ON h.wallet_id = w.id
            WHERE h.deployment_id = ?
            ORDER BY h.created_at DESC
            "#,
        )
        .bind(deployment_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(history)
    }

    /// Get a single call history entry by id
    pub async fn get_call_history(&self, id: i64) -> Result<Option<CallHistory>> {
        let history = sqlx::query_as::<_, CallHistory>("SELECT * FROM call_history WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;

        Ok(history)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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

        let id = db.upsert_network(&network).await.unwrap();
        assert!(id > 0);

        let fetched = db.get_network("tempo-testnet").await.unwrap().unwrap();
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

        let id1 = db.upsert_network(&network1).await.unwrap();

        let network2 = NewNetwork {
            name: "tempo".to_string(),
            chain_id: 200,
            rpc_url: "https://new.rpc".to_string(),
            explorer_url: Some("https://explorer.xyz".to_string()),
        };

        let id2 = db.upsert_network(&network2).await.unwrap();

        // Should return same ID (upsert)
        assert_eq!(id1, id2);

        let fetched = db.get_network("tempo").await.unwrap().unwrap();
        assert_eq!(fetched.chain_id, 200);
        assert_eq!(fetched.rpc_url, "https://new.rpc");
    }

    #[tokio::test]
    async fn test_list_networks() {
        let db = setup_test_db().await;

        db.upsert_network(&NewNetwork {
            name: "alpha".to_string(),
            chain_id: 1,
            rpc_url: "https://alpha".to_string(),
            explorer_url: None,
        })
        .await
        .unwrap();

        db.upsert_network(&NewNetwork {
            name: "beta".to_string(),
            chain_id: 2,
            rpc_url: "https://beta".to_string(),
            explorer_url: None,
        })
        .await
        .unwrap();

        let networks = db.list_networks().await.unwrap();
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

        let id = db.upsert_contract(&contract).await.unwrap();
        assert!(id > 0);

        let fetched = db.get_contract("MyToken").await.unwrap().unwrap();
        assert_eq!(fetched.name, "MyToken");
        assert_eq!(fetched.source_path, "src/MyToken.sol:MyToken");
    }

    #[tokio::test]
    async fn test_create_deployment_increments_version() {
        let db = setup_test_db().await;

        let network_id = db
            .upsert_network(&NewNetwork {
                name: "testnet".to_string(),
                chain_id: 1,
                rpc_url: "https://rpc".to_string(),
                explorer_url: None,
            })
            .await
            .unwrap();

        let contract_id = db
            .upsert_contract(&NewContract {
                name: "Token".to_string(),
                source_path: "src/Token.sol".to_string(),
                abi: "[]".to_string(),
                bytecode_hash: "0x123".to_string(),
            })
            .await
            .unwrap();

        // First deployment
        db.create_deployment(&NewDeployment {
            contract_id,
            network_id,
            address: "0xaaa".to_string(),
            deployer: "0xddd".to_string(),
            tx_hash: "0x111".to_string(),
            block_number: Some(100),
            constructor_args: None,
        })
        .await
        .unwrap();

        // Second deployment
        db.create_deployment(&NewDeployment {
            contract_id,
            network_id,
            address: "0xbbb".to_string(),
            deployer: "0xddd".to_string(),
            tx_hash: "0x222".to_string(),
            block_number: Some(200),
            constructor_args: None,
        })
        .await
        .unwrap();

        let current = db
            .get_current_deployment("Token", "testnet")
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

        let net1 = db
            .upsert_network(&NewNetwork {
                name: "net1".to_string(),
                chain_id: 1,
                rpc_url: "https://net1".to_string(),
                explorer_url: None,
            })
            .await
            .unwrap();

        let net2 = db
            .upsert_network(&NewNetwork {
                name: "net2".to_string(),
                chain_id: 2,
                rpc_url: "https://net2".to_string(),
                explorer_url: None,
            })
            .await
            .unwrap();

        let contract_id = db
            .upsert_contract(&NewContract {
                name: "Token".to_string(),
                source_path: "src/Token.sol".to_string(),
                abi: "[]".to_string(),
                bytecode_hash: "0x123".to_string(),
            })
            .await
            .unwrap();

        db.create_deployment(&NewDeployment {
            contract_id,
            network_id: net1,
            address: "0x111".to_string(),
            deployer: "0xddd".to_string(),
            tx_hash: "0xaaa".to_string(),
            block_number: None,
            constructor_args: None,
        })
        .await
        .unwrap();

        db.create_deployment(&NewDeployment {
            contract_id,
            network_id: net2,
            address: "0x222".to_string(),
            deployer: "0xddd".to_string(),
            tx_hash: "0xbbb".to_string(),
            block_number: None,
            constructor_args: None,
        })
        .await
        .unwrap();

        // List all
        let all = db.list_deployments(None).await.unwrap();
        assert_eq!(all.len(), 2);

        // Filter by net1
        let net1_only = db.list_deployments(Some("net1")).await.unwrap();
        assert_eq!(net1_only.len(), 1);
        assert_eq!(net1_only[0].network_name, "net1");

        // Filter by net2
        let net2_only = db.list_deployments(Some("net2")).await.unwrap();
        assert_eq!(net2_only.len(), 1);
        assert_eq!(net2_only[0].network_name, "net2");
    }

    #[tokio::test]
    async fn test_get_current_deployment_not_found() {
        let db = setup_test_db().await;

        let result = db
            .get_current_deployment("NonExistent", "testnet")
            .await
            .unwrap();

        assert!(result.is_none());
    }
}
