use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use smolder_core::Error;
use smolder_db::{Contract, ContractRepository};

use crate::server::error::ApiError;
use crate::server::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/contracts", get(list))
        .route("/contracts/{name}", get(get_by_name))
}

async fn list(State(state): State<AppState>) -> Result<Json<Vec<Contract>>, ApiError> {
    let contracts = ContractRepository::list(state.db()).await?;
    Ok(Json(contracts))
}

async fn get_by_name(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<Contract>, ApiError> {
    let contract = ContractRepository::get_by_name(state.db(), &name).await?;

    contract
        .map(Json)
        .ok_or_else(|| ApiError::from(Error::ContractNotFound(name)))
}
