use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::Serialize;

use crate::forge::{self, ArtifactDetails, ArtifactInfo};
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

async fn list(
    State(state): State<AppState>,
) -> Result<Json<Vec<ArtifactListItem>>, (StatusCode, String)> {
    // Get all artifacts from out/ directory
    let artifacts =
        forge::list_artifacts().map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    // Get contracts in registry to mark which artifacts are tracked
    let registry_contracts: Vec<String> = sqlx::query_scalar("SELECT name FROM contracts")
        .fetch_all(state.pool.as_ref())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    let items: Vec<ArtifactListItem> = artifacts
        .into_iter()
        .map(|info| {
            let in_registry = registry_contracts.contains(&info.name);
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
) -> Result<Json<ArtifactDetailsResponse>, (StatusCode, String)> {
    let details = forge::get_artifact_details(&name).map_err(|e| {
        if e.to_string().contains("Could not find artifact") {
            (
                StatusCode::NOT_FOUND,
                format!("Artifact '{}' not found", name),
            )
        } else {
            (StatusCode::INTERNAL_SERVER_ERROR, e.to_string())
        }
    })?;

    // Check if in registry
    let in_registry: bool = sqlx::query_scalar("SELECT COUNT(*) > 0 FROM contracts WHERE name = ?")
        .bind(&name)
        .fetch_one(state.pool.as_ref())
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(ArtifactDetailsResponse {
        details,
        in_registry,
    }))
}
