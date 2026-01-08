mod contracts;
mod deployments;
mod health;
mod interact;
mod networks;
mod wallets;

use axum::{routing::get, Router};

use crate::server::{static_files::serve_static, AppState};

pub fn create_router(state: AppState) -> Router {
    Router::new()
        .nest(
            "/api",
            health::router()
                .merge(networks::router())
                .merge(contracts::router())
                .merge(deployments::router())
                .merge(wallets::router())
                .merge(interact::router()),
        )
        .with_state(state)
        .fallback(get(serve_static))
}

#[cfg(test)]
mod tests {
    use axum::{body::Body, http::Request, Router};
    use smolder_core::{schema, Contract, DeploymentView, Network};
    use sqlx::sqlite::{SqliteConnectOptions, SqlitePoolOptions};
    use std::str::FromStr;
    use std::sync::Arc;
    use tower::ServiceExt;

    async fn setup_test_app() -> Router {
        let options = SqliteConnectOptions::from_str(":memory:")
            .unwrap()
            .create_if_missing(true)
            .foreign_keys(true);

        let pool = SqlitePoolOptions::new()
            .max_connections(1)
            .connect_with(options)
            .await
            .unwrap();

        schema::init_schema(&pool).await.unwrap();

        sqlx::query(
            "INSERT INTO networks (name, chain_id, rpc_url, explorer_url) VALUES (?, ?, ?, ?)",
        )
        .bind("testnet")
        .bind(12345_i64)
        .bind("https://rpc.test.xyz")
        .bind("https://explorer.test.xyz")
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO contracts (name, source_path, abi, bytecode_hash) VALUES (?, ?, ?, ?)",
        )
        .bind("TestToken")
        .bind("src/TestToken.sol")
        .bind(r#"[{"type":"function","name":"transfer"}]"#)
        .bind("0xabc123")
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query(
            "INSERT INTO deployments (contract_id, network_id, address, deployer, tx_hash, block_number, version, is_current) VALUES (?, ?, ?, ?, ?, ?, ?, ?)"
        )
        .bind(1_i64)
        .bind(1_i64)
        .bind("0x1234567890abcdef1234567890abcdef12345678")
        .bind("0xdeployer")
        .bind("0xtxhash")
        .bind(100_i64)
        .bind(1_i64)
        .bind(true)
        .execute(&pool)
        .await
        .unwrap();

        let state = crate::server::AppState {
            pool: Arc::new(pool),
        };

        super::create_router(state)
    }

    #[tokio::test]
    async fn test_health_check() {
        let app = setup_test_app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/health")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), axum::http::StatusCode::OK);
    }

    #[tokio::test]
    async fn test_list_networks() {
        let app = setup_test_app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/networks")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), axum::http::StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let networks: Vec<Network> = serde_json::from_slice(&body).unwrap();

        assert_eq!(networks.len(), 1);
        assert_eq!(networks[0].name, "testnet");
        assert_eq!(networks[0].chain_id, 12345);
    }

    #[tokio::test]
    async fn test_get_network() {
        let app = setup_test_app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/networks/testnet")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), axum::http::StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let network: Network = serde_json::from_slice(&body).unwrap();

        assert_eq!(network.name, "testnet");
    }

    #[tokio::test]
    async fn test_get_network_not_found() {
        let app = setup_test_app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/networks/nonexistent")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_list_contracts() {
        let app = setup_test_app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/contracts")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), axum::http::StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let contracts: Vec<Contract> = serde_json::from_slice(&body).unwrap();

        assert_eq!(contracts.len(), 1);
        assert_eq!(contracts[0].name, "TestToken");
    }

    #[tokio::test]
    async fn test_list_deployments() {
        let app = setup_test_app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/deployments")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), axum::http::StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let deployments: Vec<DeploymentView> = serde_json::from_slice(&body).unwrap();

        assert_eq!(deployments.len(), 1);
        assert_eq!(deployments[0].contract_name, "TestToken");
        assert_eq!(deployments[0].network_name, "testnet");
    }

    #[tokio::test]
    async fn test_list_deployments_filtered_by_network() {
        let app = setup_test_app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/deployments?network=testnet")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), axum::http::StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let deployments: Vec<DeploymentView> = serde_json::from_slice(&body).unwrap();

        assert_eq!(deployments.len(), 1);
    }

    #[tokio::test]
    async fn test_get_deployment() {
        let app = setup_test_app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/deployments/TestToken/testnet")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), axum::http::StatusCode::OK);

        let body = axum::body::to_bytes(response.into_body(), usize::MAX)
            .await
            .unwrap();
        let deployment: DeploymentView = serde_json::from_slice(&body).unwrap();

        assert_eq!(deployment.contract_name, "TestToken");
        assert_eq!(
            deployment.address,
            "0x1234567890abcdef1234567890abcdef12345678"
        );
    }

    #[tokio::test]
    async fn test_get_deployment_not_found() {
        let app = setup_test_app().await;

        let response = app
            .oneshot(
                Request::builder()
                    .uri("/api/deployments/NonExistent/testnet")
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();

        assert_eq!(response.status(), axum::http::StatusCode::NOT_FOUND);
    }
}
