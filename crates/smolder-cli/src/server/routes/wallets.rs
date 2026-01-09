use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use serde::Deserialize;
use smolder_core::{encrypt_private_key, Error};
use smolder_db::{NewWallet, Wallet, WalletRepository};

use crate::server::error::ApiError;
use crate::server::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/wallets", get(list))
        .route("/wallets", post(create))
        .route("/wallets/{name}", get(get_by_name))
        .route("/wallets/{name}", delete(remove))
}

async fn list(State(state): State<AppState>) -> Result<Json<Vec<Wallet>>, ApiError> {
    let wallets = WalletRepository::list(state.db()).await?;
    Ok(Json(wallets))
}

async fn get_by_name(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<Wallet>, ApiError> {
    let wallet = WalletRepository::get_by_name(state.db(), &name).await?;

    wallet
        .map(Json)
        .ok_or_else(|| ApiError::from(Error::WalletNotFound(name)))
}

#[derive(Debug, Deserialize)]
struct CreateWalletRequest {
    name: String,
    private_key: String,
}

async fn create(
    State(state): State<AppState>,
    Json(payload): Json<CreateWalletRequest>,
) -> Result<Json<Wallet>, ApiError> {
    // Normalize private key
    let private_key = if payload.private_key.starts_with("0x") {
        payload.private_key.clone()
    } else {
        format!("0x{}", payload.private_key)
    };

    // Parse and validate private key, get address
    let signer: alloy::signers::local::PrivateKeySigner = private_key
        .parse()
        .map_err(|e| ApiError::from(Error::invalid_param("private_key", format!("{}", e))))?;

    let address = format!("{:?}", signer.address());

    // Check if wallet name already exists
    if WalletRepository::get_by_name(state.db(), &payload.name)
        .await?
        .is_some()
    {
        return Err(ApiError::conflict(format!(
            "Wallet '{}' already exists",
            payload.name
        )));
    }

    // Check if address already exists
    if WalletRepository::get_by_address(state.db(), &address)
        .await?
        .is_some()
    {
        return Err(ApiError::conflict(format!(
            "A wallet with address {} already exists",
            address
        )));
    }

    // Encrypt private key
    let encrypted_key = encrypt_private_key(&private_key)?;

    // Store wallet with encrypted key in database
    let new_wallet = NewWallet {
        name: payload.name,
        address,
        encrypted_key,
    };

    let wallet = WalletRepository::create(state.db(), &new_wallet).await?;
    Ok(Json(wallet))
}

async fn remove(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<StatusCode, ApiError> {
    // Check if wallet exists
    WalletRepository::get_by_name(state.db(), &name)
        .await?
        .ok_or_else(|| ApiError::from(Error::WalletNotFound(name.clone())))?;

    // Delete from database
    WalletRepository::delete(state.db(), &name).await?;
    Ok(StatusCode::NO_CONTENT)
}
