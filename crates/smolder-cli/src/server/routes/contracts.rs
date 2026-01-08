use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use smolder_core::Contract;

use crate::server::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/contracts", get(list))
        .route("/contracts/{name}", get(get_by_name))
}

async fn list(State(state): State<AppState>) -> Result<Json<Vec<Contract>>, (StatusCode, String)> {
    let contracts = sqlx::query_as::<_, Contract>("SELECT * FROM contracts ORDER BY name")
        .fetch_all(state.pool.as_ref())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(contracts))
}

async fn get_by_name(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<Contract>, (StatusCode, String)> {
    let contract = sqlx::query_as::<_, Contract>(
        "SELECT * FROM contracts WHERE name = ? ORDER BY created_at DESC LIMIT 1",
    )
    .bind(&name)
    .fetch_optional(state.pool.as_ref())
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
