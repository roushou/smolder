//! Repository trait implementations for SQLite
//!
//! Each repository is implemented in its own module for better organization.

mod call_history;
mod contract;
mod deployment;
mod network;
mod wallet;

use crate::traits::{
    CallHistoryRepository, ContractRepository, DeploymentRepository, NetworkRepository,
    Repositories, WalletRepository,
};
use crate::Database;

impl Repositories for Database {
    fn networks(&self) -> &dyn NetworkRepository {
        self
    }

    fn contracts(&self) -> &dyn ContractRepository {
        self
    }

    fn deployments(&self) -> &dyn DeploymentRepository {
        self
    }

    fn wallets(&self) -> &dyn WalletRepository {
        self
    }

    fn call_history(&self) -> &dyn CallHistoryRepository {
        self
    }
}
