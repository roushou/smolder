//! CallHistoryRepository implementation for SQLite

use async_trait::async_trait;
use smolder_core::Result;

use crate::models::{CallHistory, CallHistoryUpdate, CallHistoryView, NewCallHistory};
use crate::traits::{CallHistoryFilter, CallHistoryRepository};
use crate::Database;

#[async_trait]
impl CallHistoryRepository for Database {
    async fn list(&self, filter: CallHistoryFilter) -> Result<Vec<CallHistory>> {
        let history = match (filter.deployment_id, filter.limit) {
            (Some(id), Some(limit)) => {
                sqlx::query_as::<_, CallHistory>(
                    "SELECT * FROM call_history WHERE deployment_id = ? ORDER BY created_at DESC LIMIT ?",
                )
                .bind(id.0)
                .bind(limit as i64)
                .fetch_all(&self.pool)
                .await?
            }
            (Some(id), None) => {
                sqlx::query_as::<_, CallHistory>(
                    "SELECT * FROM call_history WHERE deployment_id = ? ORDER BY created_at DESC",
                )
                .bind(id.0)
                .fetch_all(&self.pool)
                .await?
            }
            (None, Some(limit)) => {
                sqlx::query_as::<_, CallHistory>(
                    "SELECT * FROM call_history ORDER BY created_at DESC LIMIT ?",
                )
                .bind(limit as i64)
                .fetch_all(&self.pool)
                .await?
            }
            (None, None) => {
                sqlx::query_as::<_, CallHistory>(
                    "SELECT * FROM call_history ORDER BY created_at DESC",
                )
                .fetch_all(&self.pool)
                .await?
            }
        };
        Ok(history)
    }

    async fn list_views(&self, filter: CallHistoryFilter) -> Result<Vec<CallHistoryView>> {
        let base_query = r#"
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

        let history = match (filter.deployment_id, filter.limit) {
            (Some(id), Some(limit)) => {
                let query = format!(
                    "{} WHERE h.deployment_id = ? ORDER BY h.created_at DESC LIMIT ?",
                    base_query
                );
                sqlx::query_as::<_, CallHistoryView>(&query)
                    .bind(id.0)
                    .bind(limit as i64)
                    .fetch_all(&self.pool)
                    .await?
            }
            (Some(id), None) => {
                let query = format!(
                    "{} WHERE h.deployment_id = ? ORDER BY h.created_at DESC",
                    base_query
                );
                sqlx::query_as::<_, CallHistoryView>(&query)
                    .bind(id.0)
                    .fetch_all(&self.pool)
                    .await?
            }
            (None, Some(limit)) => {
                let query = format!("{} ORDER BY h.created_at DESC LIMIT ?", base_query);
                sqlx::query_as::<_, CallHistoryView>(&query)
                    .bind(limit as i64)
                    .fetch_all(&self.pool)
                    .await?
            }
            (None, None) => {
                let query = format!("{} ORDER BY h.created_at DESC", base_query);
                sqlx::query_as::<_, CallHistoryView>(&query)
                    .fetch_all(&self.pool)
                    .await?
            }
        };
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
        .bind(&entry.call_type)
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
        .bind(&update.status)
        .bind(&update.error_message)
        .bind(id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }
}
