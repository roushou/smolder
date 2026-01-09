use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use serde::Serialize;
use smolder_core::Error;
use smolder_db::ContractRepository;

use crate::forge::{ArtifactDetails, ArtifactInfo};
use crate::server::error::ApiError;
use crate::server::AppState;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/artifacts", get(list))
        .route("/artifacts/{name}", get(get_by_name))
}

#[derive(Serialize)]
struct ArtifactListItem {
    #[serde(flatten)]
    info: ArtifactInfo,
    in_registry: bool,
}

async fn list(State(state): State<AppState>) -> Result<Json<Vec<ArtifactListItem>>, ApiError> {
    // Get all artifacts from out/ directory
    let artifacts = state
        .artifacts()
        .list()
        .map_err(|e| ApiError::internal(e.to_string()))?;

    // Get contracts in registry to mark which artifacts are tracked
    let contracts = ContractRepository::list(state.db()).await?;

    let registry_names: Vec<String> = contracts.into_iter().map(|c| c.name).collect();

    let items: Vec<ArtifactListItem> = artifacts
        .into_iter()
        .map(|info| {
            let in_registry = registry_names.contains(&info.name);
            ArtifactListItem { info, in_registry }
        })
        .collect();

    Ok(Json(items))
}

#[derive(Serialize)]
struct ArtifactDetailsResponse {
    #[serde(flatten)]
    details: ArtifactDetails,
    in_registry: bool,
}

async fn get_by_name(
    State(state): State<AppState>,
    Path(name): Path<String>,
) -> Result<Json<ArtifactDetailsResponse>, ApiError> {
    let details = state.artifacts().get_details(&name).map_err(|e| {
        if e.to_string().contains("Could not find artifact") {
            ApiError::from(Error::ArtifactNotFound(name.clone()))
        } else {
            ApiError::internal(e.to_string())
        }
    })?;

    // Check if in registry
    let contract = ContractRepository::get_by_name(state.db(), &name).await?;
    let in_registry = contract.is_some();

    Ok(Json(ArtifactDetailsResponse {
        details,
        in_registry,
    }))
}
