use axum::{routing::get, Json, Router};
use serde::Serialize;

use crate::server::AppState;

pub fn router() -> Router<AppState> {
    Router::new().route("/health", get(check))
}

#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}

async fn check() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}
