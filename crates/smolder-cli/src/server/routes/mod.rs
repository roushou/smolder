mod artifacts;
mod contracts;
mod deploy;
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
                .merge(interact::router())
                .merge(artifacts::router())
                .merge(deploy::router()),
        )
        .with_state(state)
        .fallback(get(serve_static))
}

#[cfg(test)]
mod tests {
    use axum::{body::Body, http::Request, Router};
    use smolder_core::{Contract, DeploymentView, Network, NewContract, NewDeployment, NewNetwork};
    use tower::ServiceExt;

    use crate::db::Database;

    async fn setup_test_app() -> Router {
        let db = Database::connect_to(":memory:").await.unwrap();
        db.init_schema().await.unwrap();

        // Insert test data using database methods
        let network_id = db
            .upsert_network(&NewNetwork {
                name: "testnet".to_string(),
                chain_id: 12345,
                rpc_url: "https://rpc.test.xyz".to_string(),
                explorer_url: Some("https://explorer.test.xyz".to_string()),
            })
            .await
            .unwrap();

        let contract_id = db
            .upsert_contract(&NewContract {
                name: "TestToken".to_string(),
                source_path: "src/TestToken.sol".to_string(),
                abi: r#"[{"type":"function","name":"transfer"}]"#.to_string(),
                bytecode_hash: "0xabc123".to_string(),
            })
            .await
            .unwrap();

        db.create_deployment(&NewDeployment {
            contract_id,
            network_id,
            address: "0x1234567890abcdef1234567890abcdef12345678".to_string(),
            deployer: "0xdeployer".to_string(),
            tx_hash: "0xtxhash".to_string(),
            block_number: Some(100),
            constructor_args: None,
        })
        .await
        .unwrap();

        let state = crate::server::AppState::new(db);

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
