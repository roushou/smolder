use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use smolder_core::repository::NetworkRepository;
use smolder_core::Network;

use crate::server::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/networks", get(list))
        .route("/networks/{name}", get(get_by_name))
}

async fn list(State(state): State<AppState>) -> Result<Json<Vec<Network>>, (StatusCode, String)> {
    let networks = NetworkRepository::list(state.db())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(networks))
}

async fn get_by_name(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<Network>, (StatusCode, String)> {
    let network = NetworkRepository::get_by_name(state.db(), &name)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    match network {
        Some(n) => Ok(Json(n)),
        None => Err((
            StatusCode::NOT_FOUND,
            format!("Network '{}' not found", name),
        )),
    }
}
