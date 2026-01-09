use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use smolder_core::repository::{DeploymentFilter, DeploymentRepository};
use smolder_core::DeploymentView;

use crate::server::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/deployments", get(list)).route(
        "/deployments/{contract}/{network}",
        get(get_by_contract_and_network),
    )
}

#[derive(Deserialize, Default)]
pub struct ListQuery {
    pub network: Option<String>,
}

async fn list(
    State(state): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<DeploymentView>>, (StatusCode, String)> {
    let filter = match query.network {
        Some(ref network) => DeploymentFilter::for_network(network),
        None => DeploymentFilter::current(),
    };

    let deployments = DeploymentRepository::list(state.db(), filter)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(deployments))
}

async fn get_by_contract_and_network(
    State(state): State<AppState>,
    Path((contract, network)): Path<(String, String)>,
) -> Result<Json<DeploymentView>, (StatusCode, String)> {
    // First get the deployment
    let deployment = DeploymentRepository::get_current(state.db(), &contract, &network)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    match deployment {
        Some(d) => {
            // Now get the full view with the ABI
            let view = DeploymentRepository::get_view_by_id(state.db(), d.id.into())
                .await
                .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

            match view {
                Some(v) => Ok(Json(v)),
                None => Err((
                    StatusCode::NOT_FOUND,
                    format!(
                        "Deployment for contract '{}' on network '{}' not found",
                        contract, network
                    ),
                )),
            }
        }
        None => Err((
            StatusCode::NOT_FOUND,
            format!(
                "Deployment for contract '{}' on network '{}' not found",
                contract, network
            ),
        )),
    }
}
