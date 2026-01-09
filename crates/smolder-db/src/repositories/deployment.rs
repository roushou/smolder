//! DeploymentRepository implementation for SQLite

use async_trait::async_trait;
use smolder_core::{DeploymentId, Result};
use sqlx::QueryBuilder;

use crate::models::{Deployment, DeploymentView, NewDeployment};
use crate::traits::{DeploymentFilter, DeploymentRepository};
use crate::Database;

const DEPLOYMENT_VIEW_SELECT: &str = r#"
    SELECT
        d.id, c.name as contract_name, n.name as network_name, n.chain_id,
        d.address, d.deployer, d.tx_hash, d.block_number, d.version,
        d.deployed_at, d.is_current, c.abi
    FROM deployments d
    JOIN contracts c ON d.contract_id = c.id
    JOIN networks n ON d.network_id = n.id
"#;

#[async_trait]
impl DeploymentRepository for Database {
    async fn list(&self, filter: DeploymentFilter) -> Result<Vec<DeploymentView>> {
        let mut builder: QueryBuilder<sqlx::Sqlite> = QueryBuilder::new(DEPLOYMENT_VIEW_SELECT);

        let mut has_where = false;
        if let Some(ref network) = filter.network {
            builder.push(" WHERE n.name = ");
            builder.push_bind(network);
            has_where = true;
        }
        if filter.current_only {
            builder.push(if has_where { " AND " } else { " WHERE " });
            builder.push("d.is_current = TRUE");
        }

        builder.push(" ORDER BY n.name, c.name");
        if !filter.current_only {
            builder.push(", d.version DESC");
        }

        let deployments = builder
            .build_query_as::<DeploymentView>()
            .fetch_all(&self.pool)
            .await?;
        Ok(deployments)
    }

    async fn get_current(&self, contract: &str, network: &str) -> Result<Option<Deployment>> {
        let deployment = sqlx::query_as::<_, Deployment>(
            r#"
            SELECT d.*
            FROM deployments d
            JOIN contracts c ON d.contract_id = c.id
            JOIN networks n ON d.network_id = n.id
            WHERE c.name = ? AND n.name = ? AND d.is_current = TRUE
            "#,
        )
        .bind(contract)
        .bind(network)
        .fetch_optional(&self.pool)
        .await?;
        Ok(deployment)
    }

    async fn get_by_id(&self, id: DeploymentId) -> Result<Option<Deployment>> {
        let deployment = sqlx::query_as::<_, Deployment>("SELECT * FROM deployments WHERE id = ?")
            .bind(id.0)
            .fetch_optional(&self.pool)
            .await?;
        Ok(deployment)
    }

    async fn get_view_by_id(&self, id: DeploymentId) -> Result<Option<DeploymentView>> {
        let mut builder: QueryBuilder<sqlx::Sqlite> = QueryBuilder::new(DEPLOYMENT_VIEW_SELECT);
        builder.push(" WHERE d.id = ");
        builder.push_bind(id.0);

        let deployment = builder
            .build_query_as::<DeploymentView>()
            .fetch_optional(&self.pool)
            .await?;
        Ok(deployment)
    }

    async fn exists_by_tx_hash(&self, tx_hash: &str) -> Result<bool> {
        let exists: bool =
            sqlx::query_scalar("SELECT EXISTS(SELECT 1 FROM deployments WHERE tx_hash = ?)")
                .bind(tx_hash)
                .fetch_one(&self.pool)
                .await?;
        Ok(exists)
    }

    async fn create(&self, deployment: &NewDeployment) -> Result<Deployment> {
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

        DeploymentRepository::get_by_id(self, DeploymentId(id))
            .await?
            .ok_or_else(|| smolder_core::Error::DeploymentNotFoundById(DeploymentId(id)))
    }

    async fn list_for_export(&self, network: Option<&str>) -> Result<Vec<DeploymentView>> {
        let filter = match network {
            Some(n) => DeploymentFilter::for_network(n),
            None => DeploymentFilter::current(),
        };
        DeploymentRepository::list(self, filter).await
    }

    async fn list_versions(&self, contract: &str, network: &str) -> Result<Vec<DeploymentView>> {
        let query = format!(
            "{} WHERE c.name = ? AND n.name = ? ORDER BY d.version DESC",
            DEPLOYMENT_VIEW_SELECT
        );

        let deployments = sqlx::query_as::<_, DeploymentView>(&query)
            .bind(contract)
            .bind(network)
            .fetch_all(&self.pool)
            .await?;
        Ok(deployments)
    }
}
