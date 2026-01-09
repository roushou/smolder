//! NetworkRepository implementation for SQLite

use async_trait::async_trait;
use smolder_core::{ChainId, NetworkId, Result};

use crate::models::{Network, NewNetwork};
use crate::traits::NetworkRepository;
use crate::Database;

#[async_trait]
impl NetworkRepository for Database {
    async fn list(&self) -> Result<Vec<Network>> {
        let networks = sqlx::query_as::<_, Network>("SELECT * FROM networks ORDER BY name")
            .fetch_all(&self.pool)
            .await?;
        Ok(networks)
    }

    async fn get_by_name(&self, name: &str) -> Result<Option<Network>> {
        let network = sqlx::query_as::<_, Network>("SELECT * FROM networks WHERE name = ?")
            .bind(name)
            .fetch_optional(&self.pool)
            .await?;
        Ok(network)
    }

    async fn get_by_id(&self, id: NetworkId) -> Result<Option<Network>> {
        let network = sqlx::query_as::<_, Network>("SELECT * FROM networks WHERE id = ?")
            .bind(id.0)
            .fetch_optional(&self.pool)
            .await?;
        Ok(network)
    }

    async fn get_by_chain_id(&self, chain_id: ChainId) -> Result<Option<Network>> {
        let network = sqlx::query_as::<_, Network>("SELECT * FROM networks WHERE chain_id = ?")
            .bind(chain_id.0)
            .fetch_optional(&self.pool)
            .await?;
        Ok(network)
    }

    async fn upsert(&self, network: &NewNetwork) -> Result<Network> {
        let id = sqlx::query_scalar::<_, i64>(
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

        NetworkRepository::get_by_id(self, NetworkId(id))
            .await?
            .ok_or_else(|| smolder_core::Error::NetworkNotFound(network.name.clone()))
    }
}
