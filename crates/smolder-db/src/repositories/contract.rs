//! ContractRepository implementation for SQLite

use async_trait::async_trait;
use smolder_core::{ContractId, Result};

use crate::models::{Contract, NewContract};
use crate::traits::ContractRepository;
use crate::Database;

#[async_trait]
impl ContractRepository for Database {
    async fn list(&self) -> Result<Vec<Contract>> {
        let contracts = sqlx::query_as::<_, Contract>("SELECT * FROM contracts ORDER BY name")
            .fetch_all(&self.pool)
            .await?;
        Ok(contracts)
    }

    async fn get_by_name(&self, name: &str) -> Result<Option<Contract>> {
        let contract = sqlx::query_as::<_, Contract>(
            "SELECT * FROM contracts WHERE name = ? ORDER BY created_at DESC LIMIT 1",
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;
        Ok(contract)
    }

    async fn get_by_id(&self, id: ContractId) -> Result<Option<Contract>> {
        let contract = sqlx::query_as::<_, Contract>("SELECT * FROM contracts WHERE id = ?")
            .bind(id.0)
            .fetch_optional(&self.pool)
            .await?;
        Ok(contract)
    }

    async fn upsert(&self, contract: &NewContract) -> Result<Contract> {
        let id = sqlx::query_scalar::<_, i64>(
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

        ContractRepository::get_by_id(self, ContractId(id))
            .await?
            .ok_or_else(|| smolder_core::Error::ContractNotFound(contract.name.clone()))
    }
}
