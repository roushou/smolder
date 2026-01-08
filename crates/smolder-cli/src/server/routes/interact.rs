use alloy::dyn_abi::{DynSolType, DynSolValue, FunctionExt, JsonAbiExt};
use alloy::json_abi::{Function, StateMutability};
use alloy::network::EthereumWallet;
use alloy::primitives::{Address, Bytes, U256};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::rpc::types::TransactionRequest;
use alloy::signers::local::PrivateKeySigner;
use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use smolder_core::{abi, keyring, CallHistoryView, DeploymentView, Network, Wallet};

use crate::server::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/deployments/{id}/functions", get(get_functions))
        .route("/deployments/{id}/call", post(execute_call))
        .route("/deployments/{id}/send", post(execute_send))
        .route("/deployments/{id}/history", get(get_history))
}

// ================================
// GET /deployments/:id/functions
// ================================

#[derive(Serialize)]
struct FunctionsResponse {
    read: Vec<abi::FunctionInfo>,
    write: Vec<abi::FunctionInfo>,
}

async fn get_functions(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<FunctionsResponse>, (StatusCode, String)> {
    // Get deployment with ABI
    let deployment = get_deployment_by_id(&state, id).await?;

    // Parse and categorize functions
    let parsed = abi::categorize_functions(&deployment.abi)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(FunctionsResponse {
        read: parsed.read,
        write: parsed.write,
    }))
}

// ================================
// POST /deployments/:id/call
// ================================

#[derive(Deserialize)]
struct CallRequest {
    function_name: String,
    params: Vec<serde_json::Value>,
}

#[derive(Serialize)]
struct CallResponse {
    result: serde_json::Value,
}

async fn execute_call(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<CallRequest>,
) -> Result<Json<CallResponse>, (StatusCode, String)> {
    let deployment = get_deployment_by_id(&state, id).await?;
    let network = get_network_by_name(&state, &deployment.network_name).await?;

    // Get function from ABI
    let function = abi::get_function(&deployment.abi, &payload.function_name)
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;

    // Verify it's a read function
    if !matches!(
        function.state_mutability,
        StateMutability::View | StateMutability::Pure
    ) {
        return Err((
            StatusCode::BAD_REQUEST,
            format!(
                "Function '{}' is not a read function. Use /send for write operations.",
                payload.function_name
            ),
        ));
    }

    let call_data = encode_function_call(&function, &payload.params)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    // Execute eth_call
    let contract_address: Address = deployment.address.parse().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Invalid address: {}", e),
        )
    })?;

    let result = execute_eth_call(&network.rpc_url, contract_address, call_data)
        .await
        .map_err(|e| (StatusCode::BAD_GATEWAY, e))?;

    let decoded = decode_function_result(&function, &result)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e))?;

    Ok(Json(CallResponse { result: decoded }))
}

// ================================
// POST /deployments/:id/send
// ================================

#[derive(Deserialize)]
struct SendRequest {
    function_name: String,
    params: Vec<serde_json::Value>,
    wallet_name: String,
    #[serde(default)]
    value: Option<String>,
}

#[derive(Serialize)]
struct SendResponse {
    tx_hash: String,
    history_id: i64,
}

async fn execute_send(
    State(state): State<AppState>,
    Path(id): Path<i64>,
    Json(payload): Json<SendRequest>,
) -> Result<Json<SendResponse>, (StatusCode, String)> {
    let deployment = get_deployment_by_id(&state, id).await?;
    let network = get_network_by_name(&state, &deployment.network_name).await?;
    let wallet = get_wallet_by_name(&state, &payload.wallet_name).await?;

    let function = abi::get_function(&deployment.abi, &payload.function_name)
        .map_err(|e| (StatusCode::NOT_FOUND, e.to_string()))?;

    // Verify it's a write function
    if matches!(
        function.state_mutability,
        StateMutability::View | StateMutability::Pure
    ) {
        return Err((
            StatusCode::BAD_REQUEST,
            format!(
                "Function '{}' is a read function. Use /call for read operations.",
                payload.function_name
            ),
        ));
    }

    let call_data = encode_function_call(&function, &payload.params)
        .map_err(|e| (StatusCode::BAD_REQUEST, e))?;

    // Parse value if provided
    let value = match &payload.value {
        Some(v) => Some(
            v.parse::<U256>()
                .map_err(|e| (StatusCode::BAD_REQUEST, format!("Invalid value: {}", e)))?,
        ),
        None => None,
    };

    let history_id = record_call_history(
        &state,
        id,
        Some(wallet.id),
        &payload.function_name,
        &function.signature(),
        &payload.params,
        "write",
    )
    .await?;

    let private_key = keyring::get_private_key(&payload.wallet_name)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Execute transaction
    let contract_address: Address = deployment.address.parse().map_err(|e| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Invalid address: {}", e),
        )
    })?;

    let tx_hash = execute_transaction(
        &network.rpc_url,
        &private_key,
        contract_address,
        call_data,
        value,
    )
    .await
    .map_err(|e| {
        // Update history with error
        let _ = update_call_history_error(&state, history_id, &e);
        (StatusCode::BAD_GATEWAY, e)
    })?;

    // Update history with pending tx
    update_call_history_tx(&state, history_id, &tx_hash, "pending").await?;

    Ok(Json(SendResponse {
        tx_hash,
        history_id,
    }))
}

// ================================
// GET /deployments/:id/history
// ================================

async fn get_history(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<Vec<CallHistoryView>>, (StatusCode, String)> {
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
        LIMIT 100
        "#,
    )
    .bind(id)
    .fetch_all(state.pool.as_ref())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(history))
}

// ================================
// Helper functions
// ================================

async fn get_deployment_by_id(
    state: &AppState,
    id: i64,
) -> Result<DeploymentView, (StatusCode, String)> {
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
        WHERE d.id = ?
        "#,
    )
    .bind(id)
    .fetch_optional(state.pool.as_ref())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    deployment.ok_or((
        StatusCode::NOT_FOUND,
        format!("Deployment {} not found", id),
    ))
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

async fn get_wallet_by_name(state: &AppState, name: &str) -> Result<Wallet, (StatusCode, String)> {
    let wallet = sqlx::query_as::<_, Wallet>("SELECT * FROM wallets WHERE name = ?")
        .bind(name)
        .fetch_optional(state.pool.as_ref())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    wallet.ok_or((
        StatusCode::NOT_FOUND,
        format!("Wallet '{}' not found", name),
    ))
}

fn encode_function_call(
    function: &Function,
    params: &[serde_json::Value],
) -> Result<Bytes, String> {
    if params.len() != function.inputs.len() {
        return Err(format!(
            "Expected {} parameters, got {}",
            function.inputs.len(),
            params.len()
        ));
    }

    let mut sol_values = Vec::new();
    for (i, (param, value)) in function.inputs.iter().zip(params.iter()).enumerate() {
        let sol_value = json_to_sol_value(&param.ty.to_string(), value)
            .map_err(|e| format!("Parameter {}: {}", i, e))?;
        sol_values.push(sol_value);
    }

    let encoded = function
        .abi_encode_input(&sol_values)
        .map_err(|e| format!("Failed to encode function call: {}", e))?;

    Ok(Bytes::from(encoded))
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

fn decode_function_result(function: &Function, data: &Bytes) -> Result<serde_json::Value, String> {
    if function.outputs.is_empty() {
        return Ok(serde_json::Value::Null);
    }

    let decoded = function
        .abi_decode_output(data)
        .map_err(|e| format!("Failed to decode result: {}", e))?;

    // Convert to JSON
    let result: Vec<serde_json::Value> = decoded.iter().map(sol_value_to_json).collect();

    if result.len() == 1 {
        Ok(result.into_iter().next().unwrap())
    } else {
        Ok(serde_json::Value::Array(result))
    }
}

fn sol_value_to_json(value: &DynSolValue) -> serde_json::Value {
    match value {
        DynSolValue::Address(a) => serde_json::json!(format!("{:?}", a)),
        DynSolValue::Bool(b) => serde_json::json!(b),
        DynSolValue::Uint(n, _) => serde_json::json!(n.to_string()),
        DynSolValue::Int(n, _) => serde_json::json!(n.to_string()),
        DynSolValue::Bytes(b) => serde_json::json!(format!("0x{}", hex::encode(b))),
        DynSolValue::FixedBytes(b, _) => serde_json::json!(format!("0x{}", hex::encode(b))),
        DynSolValue::String(s) => serde_json::json!(s),
        DynSolValue::Array(arr) => {
            serde_json::Value::Array(arr.iter().map(sol_value_to_json).collect())
        }
        DynSolValue::Tuple(arr) => {
            serde_json::Value::Array(arr.iter().map(sol_value_to_json).collect())
        }
        _ => serde_json::Value::Null,
    }
}

async fn execute_eth_call(rpc_url: &str, to: Address, data: Bytes) -> Result<Bytes, String> {
    let url: reqwest::Url = rpc_url
        .parse()
        .map_err(|e| format!("Invalid RPC URL: {}", e))?;
    let provider = ProviderBuilder::new().connect_http(url);

    let tx = TransactionRequest::default().to(to).input(data.into());

    let result: Bytes = provider
        .call(tx)
        .await
        .map_err(|e| format!("RPC call failed: {}", e))?;

    Ok(result)
}

async fn execute_transaction(
    rpc_url: &str,
    private_key: &str,
    to: Address,
    data: Bytes,
    value: Option<U256>,
) -> Result<String, String> {
    let signer: PrivateKeySigner = private_key
        .parse()
        .map_err(|e| format!("Invalid private key: {}", e))?;

    let wallet = EthereumWallet::from(signer);

    let url: reqwest::Url = rpc_url
        .parse()
        .map_err(|e| format!("Invalid RPC URL: {}", e))?;
    let provider = ProviderBuilder::new().wallet(wallet).connect_http(url);

    let mut tx = TransactionRequest::default().to(to).input(data.into());

    if let Some(v) = value {
        tx = tx.value(v);
    }

    let pending = provider
        .send_transaction(tx)
        .await
        .map_err(|e| format!("Failed to send transaction: {}", e))?;

    Ok(format!("{:?}", pending.tx_hash()))
}

async fn record_call_history(
    state: &AppState,
    deployment_id: i64,
    wallet_id: Option<i64>,
    function_name: &str,
    function_signature: &str,
    params: &[serde_json::Value],
    call_type: &str,
) -> Result<i64, (StatusCode, String)> {
    let params_json = serde_json::to_string(params)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let id = sqlx::query_scalar::<_, i64>(
        r#"
        INSERT INTO call_history (deployment_id, wallet_id, function_name, function_signature, input_params, call_type)
        VALUES (?, ?, ?, ?, ?, ?)
        RETURNING id
        "#,
    )
    .bind(deployment_id)
    .bind(wallet_id)
    .bind(function_name)
    .bind(function_signature)
    .bind(&params_json)
    .bind(call_type)
    .fetch_one(state.pool.as_ref())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(id)
}

async fn update_call_history_tx(
    state: &AppState,
    id: i64,
    tx_hash: &str,
    status: &str,
) -> Result<(), (StatusCode, String)> {
    sqlx::query("UPDATE call_history SET tx_hash = ?, status = ? WHERE id = ?")
        .bind(tx_hash)
        .bind(status)
        .bind(id)
        .execute(state.pool.as_ref())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(())
}

fn update_call_history_error(state: &AppState, id: i64, error: &str) -> Result<(), String> {
    // This is called in error handlers, so we use blocking
    // In production, you'd want proper async handling
    let pool = state.pool.clone();
    let error = error.to_string();
    tokio::spawn(async move {
        let _ = sqlx::query(
            "UPDATE call_history SET status = 'failed', error_message = ? WHERE id = ?",
        )
        .bind(&error)
        .bind(id)
        .execute(pool.as_ref())
        .await;
    });
    Ok(())
}
