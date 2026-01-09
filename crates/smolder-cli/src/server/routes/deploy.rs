use alloy::dyn_abi::DynSolValue;
use alloy::hex;
use alloy::network::{EthereumWallet, TransactionBuilder};
use alloy::primitives::{keccak256, Bytes, U256};
use alloy::providers::{Provider, ProviderBuilder};
use alloy::rpc::types::TransactionRequest;
use alloy::signers::local::PrivateKeySigner;
use axum::{extract::State, routing::post, Json, Router};
use serde::{Deserialize, Serialize};
use smolder_core::{decrypt_private_key, json_to_sol_value, Error, ParamInfo};
use smolder_db::{
    ContractRepository, DeploymentId, DeploymentRepository, NetworkRepository, NewContract,
    NewDeployment, WalletRepository,
};

use crate::server::error::ApiError;
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
    deployment_id: Option<DeploymentId>,
}

async fn deploy_contract(
    State(state): State<AppState>,
    Json(payload): Json<DeployRequest>,
) -> Result<Json<DeployResponse>, ApiError> {
    // Get artifact details
    let artifact = state
        .artifacts()
        .get_details(&payload.artifact_name)
        .map_err(|e| {
            if e.to_string().contains("Could not find artifact") {
                ApiError::from(Error::ArtifactNotFound(payload.artifact_name.clone()))
            } else {
                ApiError::internal(e.to_string())
            }
        })?;

    if !artifact.has_bytecode {
        return Err(ApiError::bad_request(format!(
            "Artifact '{}' has no bytecode (may be an interface or abstract contract)",
            payload.artifact_name
        )));
    }

    // Get bytecode
    let bytecode = state
        .artifacts()
        .get_bytecode(&payload.artifact_name)
        .map_err(|e| ApiError::internal(e.to_string()))?;

    // Get network using repository
    let network = NetworkRepository::get_by_name(state.db(), &payload.network_name)
        .await?
        .ok_or_else(|| ApiError::from(Error::NetworkNotFound(payload.network_name.clone())))?;

    // Get wallet with encrypted key using repository
    let wallet = WalletRepository::get_with_key(state.db(), &payload.wallet_name)
        .await?
        .ok_or_else(|| ApiError::from(Error::WalletNotFound(payload.wallet_name.clone())))?;

    // Encode constructor args if any
    let encoded_args = if let Some(constructor) = &artifact.constructor {
        if payload.constructor_args.len() != constructor.inputs.len() {
            return Err(ApiError::bad_request(format!(
                "Expected {} constructor arguments, got {}",
                constructor.inputs.len(),
                payload.constructor_args.len()
            )));
        }

        encode_constructor_args(&constructor.inputs, &payload.constructor_args)
            .map_err(ApiError::from)?
    } else if !payload.constructor_args.is_empty() {
        return Err(ApiError::bad_request(
            "Contract has no constructor but arguments were provided",
        ));
    } else {
        Vec::new()
    };

    // Parse value if provided
    let value = match &payload.value {
        Some(v) if !v.is_empty() => {
            // Check if constructor is payable
            if let Some(constructor) = &artifact.constructor {
                if !constructor.is_payable() {
                    return Err(ApiError::bad_request(
                        "Cannot send value to non-payable constructor",
                    ));
                }
            } else {
                return Err(ApiError::bad_request(
                    "Cannot send value to contract without payable constructor",
                ));
            }
            Some(
                v.parse::<U256>()
                    .map_err(|e| ApiError::bad_request(format!("Invalid value: {}", e)))?,
            )
        }
        _ => None,
    };

    // Decrypt private key from wallet
    let private_key = decrypt_private_key(&wallet.encrypted_key)
        .map_err(|e| ApiError::internal(e.to_string()))?;

    // Combine bytecode and encoded args
    let bytecode_bytes = hex::decode(&bytecode).map_err(|e| ApiError::internal(e.to_string()))?;
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
    .map_err(ApiError::from)?;

    // Record deployment in database
    let deployment_id = if let Some(ref address) = contract_address {
        // Compute bytecode hash
        let bytecode_hash = format!("{:x}", keccak256(&bytecode_bytes));

        // Get or create contract in registry
        let abi_json = serde_json::to_string(&artifact.abi)?;

        let new_contract = NewContract {
            name: payload.artifact_name.clone(),
            source_path: artifact.source_path.clone(),
            abi: abi_json,
            bytecode_hash,
        };

        let contract = ContractRepository::upsert(state.db(), &new_contract).await?;

        // Record deployment
        let new_deployment = NewDeployment {
            contract_id: contract.id,
            network_id: network.id,
            address: address.clone(),
            deployer: wallet.address.clone(),
            tx_hash: tx_hash.clone(),
            block_number: None,
            constructor_args: None,
        };

        let deployment = DeploymentRepository::create(state.db(), &new_deployment).await?;

        Some(deployment.id)
    } else {
        None
    };

    Ok(Json(DeployResponse {
        tx_hash,
        contract_address,
        deployment_id,
    }))
}

fn encode_constructor_args(
    inputs: &[ParamInfo],
    args: &[serde_json::Value],
) -> Result<Vec<u8>, Error> {
    let mut sol_values = Vec::new();

    for (i, (input, value)) in inputs.iter().zip(args.iter()).enumerate() {
        let sol_value = json_to_sol_value(&input.param_type, value)
            .map_err(|e| Error::AbiEncode(format!("Argument {}: {}", i, e)))?;
        sol_values.push(sol_value);
    }

    let tuple = DynSolValue::Tuple(sol_values);
    Ok(tuple.abi_encode_params())
}

async fn execute_deploy(
    rpc_url: &str,
    private_key: &str,
    data: Bytes,
    value: Option<U256>,
) -> Result<(String, Option<String>), Error> {
    let signer: PrivateKeySigner = private_key
        .parse()
        .map_err(|e| Error::invalid_param("private_key", format!("Invalid: {}", e)))?;

    let wallet = EthereumWallet::from(signer);

    let url: reqwest::Url = rpc_url
        .parse()
        .map_err(|e| Error::invalid_param("rpc_url", format!("Invalid RPC URL: {}", e)))?;
    let provider = ProviderBuilder::new().wallet(wallet).connect_http(url);

    // CREATE transaction - use with_deploy_code to properly mark as deployment
    let mut tx = TransactionRequest::default().with_deploy_code(data);

    if let Some(v) = value {
        tx = tx.value(v);
    }

    let pending = provider
        .send_transaction(tx)
        .await
        .map_err(|e| Error::TransactionFailed(format!("Failed to send deployment: {}", e)))?;

    let tx_hash = format!("{:?}", pending.tx_hash());

    // Wait for receipt to get contract address
    let receipt = pending
        .get_receipt()
        .await
        .map_err(|e| Error::Rpc(format!("Failed to get transaction receipt: {}", e)))?;

    let contract_address = receipt.contract_address.map(|a| format!("{:?}", a));

    Ok((tx_hash, contract_address))
}
