//! API error types for HTTP responses
//!
//! This module provides structured error types for API responses.
//! These types are prepared for future migration of handlers from
//! (StatusCode, String) error tuples to proper ApiError responses.

#![allow(dead_code)]

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::Serialize;
use smolder_core::Error as CoreError;

/// Structured API error response
#[derive(Debug, Serialize)]
pub struct ApiError {
    /// Machine-readable error code
    pub code: &'static str,
    /// Human-readable error message
    pub message: String,
}

impl ApiError {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
        }
    }

    /// Create a not found error
    pub fn not_found(message: impl Into<String>) -> Self {
        Self::new("NOT_FOUND", message)
    }

    /// Create a bad request error
    pub fn bad_request(message: impl Into<String>) -> Self {
        Self::new("BAD_REQUEST", message)
    }

    /// Create an internal error
    pub fn internal(message: impl Into<String>) -> Self {
        Self::new("INTERNAL_ERROR", message)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let status = match self.code {
            "NETWORK_NOT_FOUND"
            | "CONTRACT_NOT_FOUND"
            | "DEPLOYMENT_NOT_FOUND"
            | "WALLET_NOT_FOUND"
            | "FUNCTION_NOT_FOUND"
            | "ARTIFACT_NOT_FOUND"
            | "FILE_NOT_FOUND"
            | "NOT_FOUND" => StatusCode::NOT_FOUND,

            "INVALID_PARAMETER" | "VALIDATION_ERROR" | "BAD_REQUEST" | "ABI_PARSE_ERROR"
            | "ABI_ENCODE_ERROR" | "ABI_DECODE_ERROR" | "HEX_DECODE_ERROR" => {
                StatusCode::BAD_REQUEST
            }

            "RPC_ERROR" | "TRANSACTION_FAILED" | "TRANSACTION_REVERTED" => StatusCode::BAD_GATEWAY,

            _ => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, Json(self)).into_response()
    }
}

impl From<CoreError> for ApiError {
    fn from(err: CoreError) -> Self {
        let code = err.code();

        // For internal errors, don't expose details
        let message = if err.is_database() {
            "An internal database error occurred".to_string()
        } else {
            err.to_string()
        };

        Self { code, message }
    }
}

impl From<sqlx::Error> for ApiError {
    fn from(err: sqlx::Error) -> Self {
        Self {
            code: "DATABASE_ERROR",
            message: format!("Database error: {}", err),
        }
    }
}

impl From<serde_json::Error> for ApiError {
    fn from(err: serde_json::Error) -> Self {
        Self {
            code: "SERIALIZATION_ERROR",
            message: format!("Serialization error: {}", err),
        }
    }
}

/// Result type alias for API handlers
pub type ApiResult<T> = Result<T, ApiError>;
