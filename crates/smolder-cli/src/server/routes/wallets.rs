use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use serde::Deserialize;
use smolder_core::encrypt_private_key;
use smolder_db::{NewWallet, Wallet, WalletRepository};

use crate::server::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/wallets", get(list))
        .route("/wallets", post(create))
        .route("/wallets/{name}", get(get_by_name))
        .route("/wallets/{name}", delete(remove))
}

async fn list(State(state): State<AppState>) -> Result<Json<Vec<Wallet>>, (StatusCode, String)> {
    let wallets = WalletRepository::list(state.db())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(wallets))
}

async fn get_by_name(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<Wallet>, (StatusCode, String)> {
    let wallet = WalletRepository::get_by_name(state.db(), &name)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    match wallet {
        Some(w) => Ok(Json(w)),
        None => Err((
            StatusCode::NOT_FOUND,
            format!("Wallet '{}' not found", name),
        )),
    }
}

#[derive(Debug, Deserialize)]
struct CreateWalletRequest {
    name: String,
    private_key: String,
}

async fn create(
    State(state): State<AppState>,
    Json(payload): Json<CreateWalletRequest>,
) -> Result<Json<Wallet>, (StatusCode, String)> {
    // Normalize private key
    let private_key = if payload.private_key.starts_with("0x") {
        payload.private_key.clone()
    } else {
        format!("0x{}", payload.private_key)
    };

    // Parse and validate private key, get address
    let signer: alloy::signers::local::PrivateKeySigner = private_key.parse().map_err(|e| {
        (
            StatusCode::BAD_REQUEST,
            format!("Invalid private key: {}", e),
        )
    })?;

    let address = format!("{:?}", signer.address());

    // Check if wallet name already exists
    let existing = WalletRepository::get_by_name(state.db(), &payload.name)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if existing.is_some() {
        return Err((
            StatusCode::CONFLICT,
            format!("Wallet '{}' already exists", payload.name),
        ));
    }

    // Check if address already exists
    let existing_addr = WalletRepository::get_by_address(state.db(), &address)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if existing_addr.is_some() {
        return Err((
            StatusCode::CONFLICT,
            format!("A wallet with address {} already exists", address),
        ));
    }

    // Encrypt private key
    let encrypted_key = encrypt_private_key(&private_key)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Store wallet with encrypted key in database
    let new_wallet = NewWallet {
        name: payload.name,
        address,
        encrypted_key,
    };

    let wallet = WalletRepository::create(state.db(), &new_wallet)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(wallet))
}

async fn remove(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<StatusCode, (StatusCode, String)> {
    // Check if wallet exists
    let wallet = WalletRepository::get_by_name(state.db(), &name)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    if wallet.is_none() {
        return Err((
            StatusCode::NOT_FOUND,
            format!("Wallet '{}' not found", name),
        ));
    }

    // Delete from database
    WalletRepository::delete(state.db(), &name)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(StatusCode::NO_CONTENT)
}
