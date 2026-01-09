use sqlx::SqlitePool;

use crate::error::Error;

/// SQL schema for initializing the database
pub const SCHEMA: &str = r#"
-- Networks configuration
CREATE TABLE IF NOT EXISTS networks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT UNIQUE NOT NULL,
    chain_id INTEGER NOT NULL,
    rpc_url TEXT NOT NULL,
    explorer_url TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Contract definitions (source-level)
CREATE TABLE IF NOT EXISTS contracts (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT NOT NULL,
    source_path TEXT NOT NULL,
    abi JSON NOT NULL,
    bytecode_hash TEXT NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    UNIQUE(name, bytecode_hash)
);

-- Deployments (instances on chains)
CREATE TABLE IF NOT EXISTS deployments (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    contract_id INTEGER NOT NULL REFERENCES contracts(id),
    network_id INTEGER NOT NULL REFERENCES networks(id),
    address TEXT NOT NULL,
    deployer TEXT NOT NULL,
    tx_hash TEXT NOT NULL,
    block_number INTEGER,
    constructor_args JSON,
    version INTEGER NOT NULL DEFAULT 1,
    deployed_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    is_current BOOLEAN DEFAULT TRUE,
    UNIQUE(network_id, address)
);

-- Index for common queries
CREATE INDEX IF NOT EXISTS idx_deployments_contract_network ON deployments(contract_id, network_id);
CREATE INDEX IF NOT EXISTS idx_deployments_current ON deployments(is_current) WHERE is_current = TRUE;

-- Wallets with encrypted private keys
CREATE TABLE IF NOT EXISTS wallets (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    name TEXT UNIQUE NOT NULL,
    address TEXT UNIQUE NOT NULL,
    encrypted_key BLOB NOT NULL,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP
);

-- Call history for contract interactions
CREATE TABLE IF NOT EXISTS call_history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    deployment_id INTEGER NOT NULL REFERENCES deployments(id),
    wallet_id INTEGER REFERENCES wallets(id),
    function_name TEXT NOT NULL,
    function_signature TEXT NOT NULL,
    input_params JSON NOT NULL,
    call_type TEXT NOT NULL CHECK (call_type IN ('read', 'write')),
    result JSON,
    tx_hash TEXT,
    block_number INTEGER,
    gas_used INTEGER,
    gas_price TEXT,
    status TEXT CHECK (status IN ('pending', 'success', 'failed', 'reverted')),
    error_message TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    confirmed_at DATETIME
);

CREATE INDEX IF NOT EXISTS idx_call_history_deployment ON call_history(deployment_id);
CREATE INDEX IF NOT EXISTS idx_call_history_wallet ON call_history(wallet_id);
"#;

/// Initialize the database schema
pub async fn init_schema(pool: &SqlitePool) -> Result<(), Error> {
    sqlx::raw_sql(SCHEMA).execute(pool).await?;
    Ok(())
}
