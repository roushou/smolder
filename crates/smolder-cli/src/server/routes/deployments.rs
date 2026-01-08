use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
    Json, Router,
};
use serde::Deserialize;
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
    let deployments = match query.network {
        Some(network) => {
            sqlx::query_as::<_, DeploymentView>(
                r#"
                SELECT
                    d.id,
                    c.name as contract_name,
                    n.name as network_name,
                    n.chain_id,
                    d.address,
                    d.deployer,
                    d.tx_hash,
                    d.block_number,
                    d.version,
                    d.deployed_at,
                    d.is_current,
                    c.abi
                FROM deployments d
                JOIN contracts c ON d.contract_id = c.id
                JOIN networks n ON d.network_id = n.id
                WHERE n.name = ? AND d.is_current = TRUE
                ORDER BY n.name, c.name
                "#,
            )
            .bind(&network)
            .fetch_all(state.pool.as_ref())
            .await
        }
        None => {
            sqlx::query_as::<_, DeploymentView>(
                r#"
                SELECT
                    d.id,
                    c.name as contract_name,
                    n.name as network_name,
                    n.chain_id,
                    d.address,
                    d.deployer,
                    d.tx_hash,
                    d.block_number,
                    d.version,
                    d.deployed_at,
                    d.is_current,
                    c.abi
                FROM deployments d
                JOIN contracts c ON d.contract_id = c.id
                JOIN networks n ON d.network_id = n.id
                WHERE d.is_current = TRUE
                ORDER BY n.name, c.name
                "#,
            )
            .fetch_all(state.pool.as_ref())
            .await
        }
    }
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    Ok(Json(deployments))
}

async fn get_by_contract_and_network(
    State(state): State<AppState>,
    Path((contract, network)): Path<(String, String)>,
) -> Result<Json<DeploymentView>, (StatusCode, String)> {
    let deployment = sqlx::query_as::<_, DeploymentView>(
        r#"
        SELECT
            d.id,
            c.name as contract_name,
            n.name as network_name,
            n.chain_id,
            d.address,
            d.deployer,
            d.tx_hash,
            d.block_number,
            d.version,
            d.deployed_at,
            d.is_current,
            c.abi
        FROM deployments d
        JOIN contracts c ON d.contract_id = c.id
        JOIN networks n ON d.network_id = n.id
        WHERE c.name = ? AND n.name = ? AND d.is_current = TRUE
        "#,
    )
    .bind(&contract)
    .bind(&network)
    .fetch_optional(state.pool.as_ref())
    .await
    .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))?;

    match deployment {
        Some(d) => Ok(Json(d)),
        None => Err((
            StatusCode::NOT_FOUND,
            format!(
                "Deployment for contract '{}' on network '{}' not found",
                contract, network
            ),
        )),
    }
}
