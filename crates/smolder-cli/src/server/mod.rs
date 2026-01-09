mod error;
mod routes;
mod state;
mod static_files;

#[allow(unused_imports)]
pub use error::{ApiError, ApiResult};
pub use state::AppState;

use smolder_db::Database;
use std::net::SocketAddr;
use tower_http::cors::{Any, CorsLayer};

pub use routes::create_router;

/// Server configuration
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port: 3000,
        }
    }
}

/// Start the smolder server
pub async fn run_server(
    db: Database,
    config: ServerConfig,
) -> Result<(), Box<dyn std::error::Error>> {
    let state = AppState::new(db);

    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods(Any)
        .allow_headers(Any);

    let app = create_router(state).layer(cors);

    let addr: SocketAddr = format!("{}:{}", config.host, config.port).parse()?;
    let listener = tokio::net::TcpListener::bind(addr).await?;

    axum::serve(listener, app).await?;

    Ok(())
}
