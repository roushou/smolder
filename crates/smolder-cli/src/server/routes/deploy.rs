use alloy::dyn_abi::{DynSolType, DynSolValue};
use alloy::hex;
use alloy::network::EthereumWallet;
use alloy::primitives::{keccak256, Address, Bytes, U256};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::rpc::types::TransactionRequest;
use alloy::signers::local::PrivateKeySigner;
use axum::{extract::State, http::StatusCode, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use smolder_core::{decrypt_private_key, Network, WalletWithKey};

use crate::forge;
use crate::server::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/deploy", post(deploy_contract))
}

#[derive(Deserialize)]
struct DeployRequest {
    artifact_name: String,
    network_name: String,
    wallet_name: String,
    #[serde(default)]
    constructor_args: Vec<serde_json::Value>,
    #[serde(default)]
    value: Option<String>,
}

#[derive(Serialize)]
struct DeployResponse {
    tx_hash: String,
    contract_address: Option<String>,
    deployment_id: Option<i64>,
}

async fn deploy_contract(
    State(state): State<AppState>,
    Json(payload): Json<DeployRequest>,
) -> Result<Json<DeployResponse>, (StatusCode, String)> {
    // Get artifact details
    let artifact = forge::get_artifact_details(&payload.artifact_name).map_err(|e| {
        if e.to_string().contains("Could not find artifact") {
            (
                StatusCode::NOT_FOUND,
                format!("Artifact '{}' not found", payload.artifact_name),
            )
        } else {
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        }
    })?;

    if !artifact.has_bytecode {
        return Err((
            StatusCode::BAD_REQUEST,
            format!(
                "Artifact '{}' has no bytecode (may be an interface or abstract contract)",
                payload.artifact_name
            ),
        ));
    }

    // Get bytecode
    let bytecode = forge::get_artifact_bytecode(&payload.artifact_name)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Get network
    let network = get_network_by_name(&state, &payload.network_name).await?;

    // Get wallet with encrypted key (validates wallet exists)
    let wallet = get_wallet_with_key(&state, &payload.wallet_name).await?;

    // Encode constructor args if any
    let encoded_args = if let Some(constructor) = &artifact.constructor {
        if payload.constructor_args.len() != constructor.inputs.len() {
            return Err((
                StatusCode::BAD_REQUEST,
                format!(
                    "Expected {} constructor arguments, got {}",
                    constructor.inputs.len(),
                    payload.constructor_args.len()
                ),
            ));
        }

        encode_constructor_args(&constructor.inputs, &payload.constructor_args)
            .map_err(|e| (StatusCode::BAD_REQUEST, e))?
    } else if !payload.constructor_args.is_empty() {
        return Err((
            StatusCode::BAD_REQUEST,
            "Contract has no constructor but arguments were provided".to_string(),
        ));
    } else {
        Vec::new()
    };

    // Parse value if provided
    let value = match &payload.value {
        Some(v) if !v.is_empty() => {
            // Check if constructor is payable
            if let Some(constructor) = &artifact.constructor {
                if constructor.state_mutability != "payable" {
                    return Err((
                        StatusCode::BAD_REQUEST,
                        "Cannot send value to non-payable constructor".to_string(),
                    ));
                }
            } else {
                return Err((
                    StatusCode::BAD_REQUEST,
                    "Cannot send value to contract without payable constructor".to_string(),
                ));
            }
            Some(
                v.parse::<U256>()
                    .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid value: {}", e)))?,
            )
        }
        _ => None,
    };

    // Decrypt private key from wallet
    let private_key = decrypt_private_key(&wallet.encrypted_key)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Combine bytecode and encoded args
    let bytecode_bytes =
        hex::decode(&bytecode).map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;
    let mut deploy_data = bytecode_bytes.clone();
    deploy_data.extend_from_slice(&encoded_args);

    // Deploy
    let (tx_hash, contract_address) = execute_deploy(
        &network.rpc_url,
        &private_key,
        Bytes::from(deploy_data),
        value,
    )
    .await
    .map_err(|e| (StatusCode::BAD_GATEWAY, e))?;

    // Record deployment in database
    let deployment_id = if let Some(ref address) = contract_address {
        // Compute bytecode hash
        let bytecode_hash = format!("{:x}", keccak256(&bytecode_bytes));

        // Get or create contract in registry
        let contract_id =
            get_or_create_contract(&state, &payload.artifact_name, &artifact, &bytecode_hash)
                .await?;

        // Record deployment
        let deployment_id = record_deployment(
            &state,
            contract_id,
            network.id,
            address,
            &wallet.address,
            &tx_hash,
        )
        .await?;

        Some(deployment_id)
    } else {
        None
    };

    Ok(Json(DeployResponse {
        tx_hash,
        contract_address,
        deployment_id,
    }))
}

async fn get_network_by_name(
    state: &AppState,
    name: &str,
) -> Result<Network, (StatusCode, String)> {
    let network = sqlx::query_as::<_, Network>("SELECT * FROM networks WHERE name = ?")
        .bind(name)
        .fetch_optional(state.pool.as_ref())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    network.ok_or((
        StatusCode::NOT_FOUND,
        format!("Network '{}' not found", name),
    ))
}

async fn get_wallet_with_key(
    state: &AppState,
    name: &str,
) -> Result<WalletWithKey, (StatusCode, String)> {
    let wallet = sqlx::query_as::<_, WalletWithKey>("SELECT * FROM wallets WHERE name = ?")
        .bind(name)
        .fetch_optional(state.pool.as_ref())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    wallet.ok_or((
        StatusCode::NOT_FOUND,
        format!("Wallet '{}' not found", name),
    ))
}

fn encode_constructor_args(
    inputs: &[forge::ConstructorInput],
    args: &[serde_json::Value],
) -> Result<Vec<u8>, String> {
    let mut sol_values = Vec::new();

    for (i, (input, value)) in inputs.iter().zip(args.iter()).enumerate() {
        let sol_value = json_to_sol_value(&input.param_type, value)
            .map_err(|e| format!("Argument {}: {}", i, e))?;
        sol_values.push(sol_value);
    }

    // Encode the values as a tuple
    let tuple = DynSolValue::Tuple(sol_values);
    Ok(tuple.abi_encode_params())
}

fn json_to_sol_value(type_str: &str, value: &serde_json::Value) -> Result<DynSolValue, String> {
    let sol_type: DynSolType = type_str
        .parse()
        .map_err(|e| format!("Unknown type '{}': {}", type_str, e))?;

    match sol_type {
        DynSolType::Address => {
            let addr_str = value.as_str().ok_or("Expected string for address")?;
            let addr: Address = addr_str
                .parse()
                .map_err(|e| format!("Invalid address '{}': {}", addr_str, e))?;
            Ok(DynSolValue::Address(addr))
        }
        DynSolType::Bool => {
            let b = value.as_bool().ok_or("Expected boolean")?;
            Ok(DynSolValue::Bool(b))
        }
        DynSolType::Uint(bits) => {
            let n = parse_uint(value)?;
            Ok(DynSolValue::Uint(n, bits))
        }
        DynSolType::Int(bits) => {
            let n = parse_int(value)?;
            Ok(DynSolValue::Int(n, bits))
        }
        DynSolType::Bytes => {
            let hex_str = value.as_str().ok_or("Expected hex string for bytes")?;
            let bytes: Bytes = hex_str.parse().map_err(|e| format!("Invalid hex: {}", e))?;
            Ok(DynSolValue::Bytes(bytes.to_vec()))
        }
        DynSolType::String => {
            let s = value.as_str().ok_or("Expected string")?;
            Ok(DynSolValue::String(s.to_string()))
        }
        DynSolType::FixedBytes(size) => {
            let hex_str = value.as_str().ok_or("Expected hex string")?;
            let bytes: Bytes = hex_str.parse().map_err(|e| format!("Invalid hex: {}", e))?;
            if bytes.len() != size {
                return Err(format!("Expected {} bytes, got {}", size, bytes.len()));
            }
            Ok(DynSolValue::FixedBytes(
                alloy::primitives::FixedBytes::from_slice(&bytes),
                size,
            ))
        }
        DynSolType::Array(inner) => {
            let arr = value.as_array().ok_or("Expected array")?;
            let inner_str = inner.to_string();
            let values: Result<Vec<_>, _> = arr
                .iter()
                .map(|v| json_to_sol_value(&inner_str, v))
                .collect();
            Ok(DynSolValue::Array(values?))
        }
        _ => Err(format!("Unsupported type: {}", type_str)),
    }
}

fn parse_uint(value: &serde_json::Value) -> Result<U256, String> {
    match value {
        serde_json::Value::Number(n) => {
            if let Some(u) = n.as_u64() {
                Ok(U256::from(u))
            } else if let Some(i) = n.as_i64() {
                if i >= 0 {
                    Ok(U256::from(i as u64))
                } else {
                    Err("Negative number not allowed for uint".to_string())
                }
            } else {
                Err("Number too large".to_string())
            }
        }
        serde_json::Value::String(s) => s
            .parse::<U256>()
            .map_err(|e| format!("Invalid uint: {}", e)),
        _ => Err("Expected number or string for uint".to_string()),
    }
}

fn parse_int(value: &serde_json::Value) -> Result<alloy::primitives::I256, String> {
    match value {
        serde_json::Value::Number(n) => {
            if let Some(i) = n.as_i64() {
                Ok(alloy::primitives::I256::try_from(i).unwrap())
            } else {
                Err("Number out of range".to_string())
            }
        }
        serde_json::Value::String(s) => s
            .parse::<alloy::primitives::I256>()
            .map_err(|e| format!("Invalid int: {}", e)),
        _ => Err("Expected number or string for int".to_string()),
    }
}

async fn execute_deploy(
    rpc_url: &str,
    private_key: &str,
    data: Bytes,
    value: Option<U256>,
) -> Result<(String, Option<String>), String> {
    let signer: PrivateKeySigner = private_key
        .parse()
        .map_err(|e| format!("Invalid private key: {}", e))?;

    let wallet = EthereumWallet::from(signer);

    let url: reqwest::Url = rpc_url
        .parse()
        .map_err(|e| format!("Invalid RPC URL: {}", e))?;
    let provider = ProviderBuilder::new().wallet(wallet).connect_http(url);

    // CREATE transaction - no 'to' address
    let mut tx = TransactionRequest::default().input(data.into());

    if let Some(v) = value {
        tx = tx.value(v);
    }

    let pending = provider
        .send_transaction(tx)
        .await
        .map_err(|e| format!("Failed to send deployment transaction: {}", e))?;

    let tx_hash = format!("{:?}", pending.tx_hash());

    // Wait for receipt to get contract address
    let receipt = pending
        .get_receipt()
        .await
        .map_err(|e| format!("Failed to get transaction receipt: {}", e))?;

    let contract_address = receipt.contract_address.map(|a| format!("{:?}", a));

    Ok((tx_hash, contract_address))
}

async fn get_or_create_contract(
    state: &AppState,
    name: &str,
    artifact: &forge::ArtifactDetails,
    bytecode_hash: &str,
) -> Result<i64, (StatusCode, String)> {
    // Check if contract already exists
    let existing: Option<i64> =
        sqlx::query_scalar("SELECT id FROM contracts WHERE name = ? AND bytecode_hash = ?")
            .bind(name)
            .bind(bytecode_hash)
            .fetch_optional(state.pool.as_ref())
            .await
            .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if let Some(id) = existing {
        return Ok(id);
    }

    // Create new contract
    let abi_json = serde_json::to_string(&artifact.abi)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let id = sqlx::query_scalar::<_, i64>(
        "INSERT INTO contracts (name, source_path, abi, bytecode_hash) VALUES (?, ?, ?, ?) RETURNING id",
    )
    .bind(name)
    .bind(&artifact.source_path)
    .bind(&abi_json)
    .bind(bytecode_hash)
    .fetch_one(state.pool.as_ref())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(id)
}

async fn record_deployment(
    state: &AppState,
    contract_id: i64,
    network_id: i64,
    address: &str,
    deployer: &str,
    tx_hash: &str,
) -> Result<i64, (StatusCode, String)> {
    // Get next version for this contract/network
    let next_version: i64 = sqlx::query_scalar(
        "SELECT COALESCE(MAX(version), 0) + 1 FROM deployments WHERE contract_id = ? AND network_id = ?",
    )
    .bind(contract_id)
    .bind(network_id)
    .fetch_one(state.pool.as_ref())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Mark previous deployments as not current
    sqlx::query("UPDATE deployments SET is_current = 0 WHERE contract_id = ? AND network_id = ?")
        .bind(contract_id)
        .bind(network_id)
        .execute(state.pool.as_ref())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Insert new deployment
    let id = sqlx::query_scalar::<_, i64>(
        r#"
        INSERT INTO deployments (contract_id, network_id, address, deployer, tx_hash, version, is_current)
        VALUES (?, ?, ?, ?, ?, ?, 1)
        RETURNING id
        "#,
    )
    .bind(contract_id)
    .bind(network_id)
    .bind(address)
    .bind(deployer)
    .bind(tx_hash)
    .bind(next_version)
    .fetch_one(state.pool.as_ref())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(id)
}
