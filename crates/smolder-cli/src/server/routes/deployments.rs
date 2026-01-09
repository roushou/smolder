use axum::{
    extract::{Path, Query, State},
    routing::get,
    Json, Router,
};
use serde::Deserialize;
use smolder_core::Error;
use smolder_db::{DeploymentFilter, DeploymentRepository, DeploymentView};

use crate::server::error::ApiError;
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
) -> Result<Json<Vec<DeploymentView>>, ApiError> {
    let filter = match query.network {
        Some(ref network) => DeploymentFilter::for_network(network),
        None => DeploymentFilter::current(),
    };

    let deployments = DeploymentRepository::list(state.db(), filter).await?;
    Ok(Json(deployments))
}

async fn get_by_contract_and_network(
    State(state): State<AppState>,
    Path((contract, network)): Path<(String, String)>,
) -> Result<Json<DeploymentView>, ApiError> {
    let not_found = || {
        ApiError::from(Error::DeploymentNotFound(format!(
            "contract '{}' on network '{}'",
            contract, network
        )))
    };

    let deployment = DeploymentRepository::get_current(state.db(), &contract, &network)
        .await?
        .ok_or_else(not_found)?;

    let view = DeploymentRepository::get_view_by_id(state.db(), deployment.id)
        .await?
        .ok_or_else(not_found)?;

    Ok(Json(view))
}
