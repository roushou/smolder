mod rpc;

use alloy::dyn_abi::{FunctionExt, JsonAbiExt};
use alloy::json_abi::{Function, StateMutability};
use alloy::primitives::{Address, Bytes, U256};
use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use smolder_core::{
    decrypt_private_key, json_to_sol_value, sol_value_to_json, Abi, Error, FunctionInfo,
};
use smolder_db::{
    CallHistoryFilter, CallHistoryRepository, CallHistoryUpdate, CallHistoryView, CallType,
    DeploymentId, DeploymentRepository, DeploymentView, Network, NetworkRepository, NewCallHistory,
    TransactionStatus, WalletId, WalletRepository, WalletWithKey,
};

use crate::server::error::ApiError;
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
    read: Vec<FunctionInfo>,
    write: Vec<FunctionInfo>,
}

async fn get_functions(
    State(state): State<AppState>,
    Path(id): Path<i64>,
) -> Result<Json<FunctionsResponse>, ApiError> {
    // Get deployment with ABI
    let deployment = get_deployment_by_id(&state, id).await?;

    // Parse and categorize functions
    let abi = Abi::parse(&deployment.abi).map_err(|e| ApiError::internal(e.to_string()))?;
    let parsed = abi.functions();

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
) -> Result<Json<CallResponse>, ApiError> {
    let deployment = get_deployment_by_id(&state, id).await?;
    let network = get_network_by_name(&state, &deployment.network_name).await?;

    // Get function from ABI
    let abi = Abi::parse(&deployment.abi).map_err(|e| ApiError::internal(e.to_string()))?;
    let function = abi
        .function(&payload.function_name)
        .cloned()
        .ok_or_else(|| {
            ApiError::not_found(format!("Function '{}' not found", payload.function_name))
        })?;

    // Verify it's a read function
    if !matches!(
        function.state_mutability,
        StateMutability::View | StateMutability::Pure
    ) {
        return Err(ApiError::bad_request(format!(
            "Function '{}' is not a read function. Use /send for write operations.",
            payload.function_name
        )));
    }

    let call_data = encode_function_call(&function, &payload.params).map_err(ApiError::from)?;

    // Execute eth_call
    let contract_address: Address = deployment
        .address
        .parse()
        .map_err(|e| ApiError::internal(format!("Invalid address: {}", e)))?;

    let result = rpc::execute_eth_call(&network.rpc_url, contract_address, call_data)
        .await
        .map_err(ApiError::from)?;

    let decoded = decode_function_result(&function, &result).map_err(ApiError::from)?;

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
) -> Result<Json<SendResponse>, ApiError> {
    let deployment = get_deployment_by_id(&state, id).await?;
    let network = get_network_by_name(&state, &deployment.network_name).await?;
    let wallet = get_wallet_by_name(&state, &payload.wallet_name).await?;

    // Get function from ABI
    let abi = Abi::parse(&deployment.abi).map_err(|e| ApiError::internal(e.to_string()))?;
    let function = abi
        .function(&payload.function_name)
        .cloned()
        .ok_or_else(|| {
            ApiError::not_found(format!("Function '{}' not found", payload.function_name))
        })?;

    // Verify it's a write function
    if matches!(
        function.state_mutability,
        StateMutability::View | StateMutability::Pure
    ) {
        return Err(ApiError::bad_request(format!(
            "Function '{}' is a read function. Use /call for read operations.",
            payload.function_name
        )));
    }

    let call_data = encode_function_call(&function, &payload.params).map_err(ApiError::from)?;

    // Parse value if provided
    let value = match &payload.value {
        Some(v) => Some(
            v.parse::<U256>()
                .map_err(|e| ApiError::bad_request(format!("Invalid value: {}", e)))?,
        ),
        None => None,
    };

    let history_id = record_call_history(
        &state,
        deployment.id,
        Some(wallet.id),
        &payload.function_name,
        &function.signature(),
        &payload.params,
        CallType::Write,
    )
    .await?;

    let private_key = decrypt_private_key(&wallet.encrypted_key)
        .map_err(|e| ApiError::internal(e.to_string()))?;

    // Execute transaction
    let contract_address: Address = deployment
        .address
        .parse()
        .map_err(|e| ApiError::internal(format!("Invalid address: {}", e)))?;

    let tx_hash = rpc::execute_transaction(
        &network.rpc_url,
        &private_key,
        contract_address,
        call_data,
        value,
    )
    .await
    .map_err(|e| {
        // Update history with error
        let state_clone = state.clone();
        let error_msg = e.to_string();
        tokio::spawn(async move {
            let _ = update_call_history_error(&state_clone, history_id, &error_msg).await;
        });
        ApiError::from(e)
    })?;

    // Update history with pending tx
    update_call_history_tx(&state, history_id, &tx_hash, TransactionStatus::Pending).await?;

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
) -> Result<Json<Vec<CallHistoryView>>, ApiError> {
    let filter = CallHistoryFilter {
        deployment_id: Some(DeploymentId(id)),
        limit: Some(100),
    };

    let history = CallHistoryRepository::list_views(state.db(), filter).await?;

    Ok(Json(history))
}

// ================================
// Helper functions
// ================================

async fn get_deployment_by_id(state: &AppState, id: i64) -> Result<DeploymentView, ApiError> {
    let deployment = DeploymentRepository::get_view_by_id(state.db(), DeploymentId(id)).await?;

    deployment.ok_or_else(|| ApiError::from(Error::DeploymentNotFound(format!("id {}", id))))
}

async fn get_network_by_name(state: &AppState, name: &str) -> Result<Network, ApiError> {
    let network = NetworkRepository::get_by_name(state.db(), name).await?;

    network.ok_or_else(|| ApiError::from(Error::NetworkNotFound(name.to_string())))
}

async fn get_wallet_by_name(state: &AppState, name: &str) -> Result<WalletWithKey, ApiError> {
    let wallet = WalletRepository::get_with_key(state.db(), name).await?;

    wallet.ok_or_else(|| ApiError::from(Error::WalletNotFound(name.to_string())))
}

fn encode_function_call(function: &Function, params: &[serde_json::Value]) -> Result<Bytes, Error> {
    if params.len() != function.inputs.len() {
        return Err(Error::AbiEncode(format!(
            "Expected {} parameters, got {}",
            function.inputs.len(),
            params.len()
        )));
    }

    let mut sol_values = Vec::new();
    for (i, (param, value)) in function.inputs.iter().zip(params.iter()).enumerate() {
        let sol_value = json_to_sol_value(&param.ty.to_string(), value)
            .map_err(|e| Error::AbiEncode(format!("Parameter {}: {}", i, e)))?;
        sol_values.push(sol_value);
    }

    let encoded = function
        .abi_encode_input(&sol_values)
        .map_err(|e| Error::AbiEncode(format!("Failed to encode function call: {}", e)))?;

    Ok(Bytes::from(encoded))
}

fn decode_function_result(function: &Function, data: &Bytes) -> Result<serde_json::Value, Error> {
    if function.outputs.is_empty() {
        return Ok(serde_json::Value::Null);
    }

    let decoded = function
        .abi_decode_output(data)
        .map_err(|e| Error::AbiDecode(format!("Failed to decode result: {}", e)))?;

    let result: Vec<serde_json::Value> = decoded.iter().map(sol_value_to_json).collect();

    if result.len() == 1 {
        Ok(result.into_iter().next().unwrap())
    } else {
        Ok(serde_json::Value::Array(result))
    }
}

async fn record_call_history(
    state: &AppState,
    deployment_id: DeploymentId,
    wallet_id: Option<WalletId>,
    function_name: &str,
    function_signature: &str,
    params: &[serde_json::Value],
    call_type: CallType,
) -> Result<i64, ApiError> {
    let params_json = serde_json::to_string(params)?;

    let entry = NewCallHistory {
        deployment_id,
        wallet_id,
        function_name: function_name.to_string(),
        function_signature: function_signature.to_string(),
        input_params: params_json,
        call_type,
    };

    let history = CallHistoryRepository::create(state.db(), &entry).await?;

    Ok(history.id)
}

async fn update_call_history_tx(
    state: &AppState,
    id: i64,
    tx_hash: &str,
    status: TransactionStatus,
) -> Result<(), ApiError> {
    let update = CallHistoryUpdate {
        result: None,
        tx_hash: Some(tx_hash.to_string()),
        block_number: None,
        gas_used: None,
        gas_price: None,
        status,
        error_message: None,
    };

    CallHistoryRepository::update(state.db(), id, &update).await?;

    Ok(())
}

async fn update_call_history_error(state: &AppState, id: i64, error: &str) -> Result<(), ApiError> {
    let update = CallHistoryUpdate {
        result: None,
        tx_hash: None,
        block_number: None,
        gas_used: None,
        gas_price: None,
        status: TransactionStatus::Failed,
        error_message: Some(error.to_string()),
    };

    CallHistoryRepository::update(state.db(), id, &update).await?;

    Ok(())
}
