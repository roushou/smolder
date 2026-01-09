//! CallHistoryRepository implementation for SQLite

use async_trait::async_trait;
use smolder_core::Result;
use sqlx::QueryBuilder;

use crate::models::{CallHistory, CallHistoryUpdate, CallHistoryView, NewCallHistory};
use crate::traits::{CallHistoryFilter, CallHistoryRepository};
use crate::Database;

const CALL_HISTORY_VIEW_SELECT: &str = r#"
    SELECT
        h.id, h.deployment_id, c.name as contract_name, n.name as network_name,
        d.address as contract_address, w.name as wallet_name, h.function_name,
        h.function_signature, h.input_params, h.call_type, h.result, h.tx_hash,
        h.block_number, h.gas_used, h.gas_price, h.status, h.error_message,
        h.created_at, h.confirmed_at
    FROM call_history h
    JOIN deployments d ON h.deployment_id = d.id
    JOIN contracts c ON d.contract_id = c.id
    JOIN networks n ON d.network_id = n.id
    LEFT JOIN wallets w ON h.wallet_id = w.id
"#;

#[async_trait]
impl CallHistoryRepository for Database {
    async fn list(&self, filter: CallHistoryFilter) -> Result<Vec<CallHistory>> {
        let mut builder: QueryBuilder<sqlx::Sqlite> =
            QueryBuilder::new("SELECT * FROM call_history");

        if let Some(id) = filter.deployment_id {
            builder.push(" WHERE deployment_id = ");
            builder.push_bind(id.0);
        }

        builder.push(" ORDER BY created_at DESC");

        if let Some(limit) = filter.limit {
            builder.push(" LIMIT ");
            builder.push_bind(limit as i64);
        }

        let history = builder
            .build_query_as::<CallHistory>()
            .fetch_all(&self.pool)
            .await?;
        Ok(history)
    }

    async fn list_views(&self, filter: CallHistoryFilter) -> Result<Vec<CallHistoryView>> {
        let mut builder: QueryBuilder<sqlx::Sqlite> = QueryBuilder::new(CALL_HISTORY_VIEW_SELECT);

        if let Some(id) = filter.deployment_id {
            builder.push(" WHERE h.deployment_id = ");
            builder.push_bind(id.0);
        }

        builder.push(" ORDER BY h.created_at DESC");

        if let Some(limit) = filter.limit {
            builder.push(" LIMIT ");
            builder.push_bind(limit as i64);
        }

        let history = builder
            .build_query_as::<CallHistoryView>()
            .fetch_all(&self.pool)
            .await?;
        Ok(history)
    }

    async fn get_by_id(&self, id: i64) -> Result<Option<CallHistory>> {
        let entry = sqlx::query_as::<_, CallHistory>("SELECT * FROM call_history WHERE id = ?")
            .bind(id)
            .fetch_optional(&self.pool)
            .await?;
        Ok(entry)
    }

    async fn create(&self, entry: &NewCallHistory) -> Result<CallHistory> {
        let id = sqlx::query_scalar::<_, i64>(
            r#"
            INSERT INTO call_history (deployment_id, wallet_id, function_name, function_signature, input_params, call_type)
            VALUES (?, ?, ?, ?, ?, ?)
            RETURNING id
            "#,
        )
        .bind(entry.deployment_id)
        .bind(entry.wallet_id)
        .bind(&entry.function_name)
        .bind(&entry.function_signature)
        .bind(&entry.input_params)
        .bind(entry.call_type)
        .fetch_one(&self.pool)
        .await?;

        CallHistoryRepository::get_by_id(self, id)
            .await?
            .ok_or_else(|| smolder_core::Error::Validation("Failed to create call history".into()))
    }

    async fn update(&self, id: i64, update: &CallHistoryUpdate) -> Result<()> {
        sqlx::query(
            r#"
            UPDATE call_history SET
                result = ?,
                tx_hash = ?,
                block_number = ?,
                gas_used = ?,
                gas_price = ?,
                status = ?,
                error_message = ?,
                confirmed_at = datetime('now')
            WHERE id = ?
            "#,
        )
        .bind(&update.result)
        .bind(&update.tx_hash)
        .bind(update.block_number)
        .bind(update.gas_used)
        .bind(&update.gas_price)
        .bind(update.status)
        .bind(&update.error_message)
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
