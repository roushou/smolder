use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use smolder_core::Error;
use smolder_db::{Network, NetworkRepository};

use crate::server::error::ApiError;
use crate::server::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/networks", get(list))
        .route("/networks/{name}", get(get_by_name))
}

async fn list(State(state): State<AppState>) -> Result<Json<Vec<Network>>, ApiError> {
    let networks = NetworkRepository::list(state.db()).await?;
    Ok(Json(networks))
}

async fn get_by_name(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<Network>, ApiError> {
    let network = NetworkRepository::get_by_name(state.db(), &name).await?;

    network
        .map(Json)
        .ok_or_else(|| ApiError::from(Error::NetworkNotFound(name)))
}
