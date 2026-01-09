use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use smolder_db::{Contract, ContractRepository};

use crate::server::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/contracts", get(list))
        .route("/contracts/{name}", get(get_by_name))
}

async fn list(State(state): State<AppState>) -> Result<Json<Vec<Contract>>, (StatusCode, String)> {
    let contracts = ContractRepository::list(state.db())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(contracts))
}

async fn get_by_name(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<Contract>, (StatusCode, String)> {
    let contract = ContractRepository::get_by_name(state.db(), &name)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    match contract {
        Some(c) => Ok(Json(c)),
        None => Err((
            StatusCode::NOT_FOUND,
            format!("Contract '{}' not found", name),
        )),
    }
}
