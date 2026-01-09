//! WalletRepository implementation for SQLite

use async_trait::async_trait;
use smolder_core::{Result, WalletId};

use crate::models::{NewWallet, Wallet, WalletWithKey};
use crate::traits::WalletRepository;
use crate::Database;

#[async_trait]
impl WalletRepository for Database {
    async fn list(&self) -> Result<Vec<Wallet>> {
        let wallets = sqlx::query_as::<_, Wallet>(
            "SELECT id, name, address, created_at FROM wallets ORDER BY name",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(wallets)
    }

    async fn get_by_name(&self, name: &str) -> Result<Option<Wallet>> {
        let wallet = sqlx::query_as::<_, Wallet>(
            "SELECT id, name, address, created_at FROM wallets WHERE name = ?",
        )
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;
        Ok(wallet)
    }

    async fn get_with_key(&self, name: &str) -> Result<Option<WalletWithKey>> {
        let wallet = sqlx::query_as::<_, WalletWithKey>("SELECT * FROM wallets WHERE name = ?")
            .bind(name)
            .fetch_optional(&self.pool)
            .await?;
        Ok(wallet)
    }

    async fn get_by_id(&self, id: WalletId) -> Result<Option<Wallet>> {
        let wallet = sqlx::query_as::<_, Wallet>(
            "SELECT id, name, address, created_at FROM wallets WHERE id = ?",
        )
        .bind(id.0)
        .fetch_optional(&self.pool)
        .await?;
        Ok(wallet)
    }

    async fn get_by_address(&self, address: &str) -> Result<Option<Wallet>> {
        let wallet = sqlx::query_as::<_, Wallet>(
            "SELECT id, name, address, created_at FROM wallets WHERE address = ?",
        )
        .bind(address)
        .fetch_optional(&self.pool)
        .await?;
        Ok(wallet)
    }

    async fn create(&self, wallet: &NewWallet) -> Result<Wallet> {
        let id = sqlx::query_scalar::<_, i64>(
            "INSERT INTO wallets (name, address, encrypted_key) VALUES (?, ?, ?) RETURNING id",
        )
        .bind(&wallet.name)
        .bind(&wallet.address)
        .bind(&wallet.encrypted_key)
        .fetch_one(&self.pool)
        .await?;

        WalletRepository::get_by_id(self, WalletId(id))
            .await?
            .ok_or_else(|| smolder_core::Error::WalletNotFound(wallet.name.clone()))
    }

    async fn delete(&self, name: &str) -> Result<()> {
        sqlx::query("DELETE FROM wallets WHERE name = ?")
            .bind(name)
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
