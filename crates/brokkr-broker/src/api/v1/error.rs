/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Canonical error model for the v1 API.
//!
//! Every fallible v1 handler returns `Result<T, ApiError>`. `ApiError` carries
//! an HTTP status, a stable machine-readable `code`, a human-readable message,
//! and an optional structured `details` map. It serializes to the
//! [`ErrorResponse`] wire format, which is documented in the OpenAPI spec and
//! consumed by generated SDK clients.
//!
//! The `code` strings are part of the SDK contract — callers pattern-match on
//! them — so they should be chosen conservatively and treated as breaking when
//! renamed.

use axum::{
    http::StatusCode,
    response::{IntoResponse, Json, Response},
};
use serde::Serialize;
use serde_json::Value;
use std::collections::BTreeMap;
use utoipa::ToSchema;

/// Wire format for every 4xx/5xx response body in the v1 API.
#[derive(Debug, Clone, Serialize, ToSchema)]
pub struct ErrorResponse {
    /// Stable, machine-readable error code (e.g. `agent_not_found`).
    pub code: String,
    /// Human-readable message; not stable and may change between versions.
    pub message: String,
    /// Optional structured context. Keys and value shapes are documented per
    /// error code when present.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<BTreeMap<String, Value>>,
}

/// Errors returned by v1 handlers. Maps 1:1 to an HTTP status and an
/// [`ErrorResponse`] body via [`IntoResponse`].
#[derive(Debug, Clone)]
pub struct ApiError {
    pub status: StatusCode,
    pub code: String,
    pub message: String,
    pub details: Option<BTreeMap<String, Value>>,
}

impl ApiError {
    fn new(status: StatusCode, code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            status,
            code: code.into(),
            message: message.into(),
            details: None,
        }
    }

    /// Attach structured context to an error. Useful for surfacing the ID that
    /// wasn't found, the field that failed validation, etc.
    pub fn with_details(mut self, details: BTreeMap<String, Value>) -> Self {
        self.details = Some(details);
        self
    }

    // ----- 4xx constructors -----

    pub fn bad_request(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(StatusCode::BAD_REQUEST, code, message)
    }

    pub fn unauthorized(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(StatusCode::UNAUTHORIZED, code, message)
    }

    pub fn forbidden(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(StatusCode::FORBIDDEN, code, message)
    }

    pub fn not_found(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(StatusCode::NOT_FOUND, code, message)
    }

    pub fn conflict(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(StatusCode::CONFLICT, code, message)
    }

    pub fn unprocessable(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self::new(StatusCode::UNPROCESSABLE_ENTITY, code, message)
    }

    // ----- 5xx constructors -----

    pub fn internal(message: impl Into<String>) -> Self {
        Self::new(StatusCode::INTERNAL_SERVER_ERROR, "internal_error", message)
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let body = ErrorResponse {
            code: self.code,
            message: self.message,
            details: self.details,
        };
        (self.status, Json(body)).into_response()
    }
}

impl std::fmt::Display for ApiError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "[{}] {}: {}",
            self.status.as_u16(),
            self.code,
            self.message
        )
    }
}

impl std::error::Error for ApiError {}

/// Classify a `diesel::result::Error` into the right `ApiError` variant.
///
/// The default `From<diesel::Error>` impl below delegates to this. Handlers
/// can call it directly to override the human-readable message while keeping
/// the structured `code` + status classification: e.g.
/// `dal.foo().create(&x).map_err(|e| ApiError::from_diesel(e, "failed to create foo"))`.
///
/// The classification rules:
/// - `DatabaseError(UniqueViolation, _)` → **409** `unique_violation`. The
///   violated constraint name is folded into the message and exposed via
///   `details.constraint`, so callers can pattern-match on the specific
///   constraint when relevant (e.g. `unique_generator_name`).
/// - `DatabaseError(ForeignKeyViolation, _)` → **422** `foreign_key_violation`.
///   The referenced row doesn't exist; this is a client-side bug.
/// - `DatabaseError(CheckViolation, _)` → **422** `check_violation`.
/// - `DatabaseError(NotNullViolation, _)` → **422** `not_null_violation`.
/// - `NotFound` → **404** `not_found`. Most handlers handle missing rows
///   explicitly, but this is a safety net.
/// - Everything else → **500** `internal_error`. The original diesel error is
///   logged via `tracing::error!` for operator debugging.
impl ApiError {
    pub fn from_diesel(err: diesel::result::Error, internal_message: impl Into<String>) -> Self {
        use diesel::result::{DatabaseErrorKind, Error as DieselErr};
        let internal_message = internal_message.into();
        match err {
            DieselErr::DatabaseError(DatabaseErrorKind::UniqueViolation, info) => {
                let constraint = info.constraint_name().unwrap_or("unique constraint");
                let mut details = std::collections::BTreeMap::new();
                details.insert(
                    "constraint".into(),
                    serde_json::Value::String(constraint.to_string()),
                );
                ApiError::conflict(
                    "unique_violation",
                    format!("violates unique constraint `{constraint}`"),
                )
                .with_details(details)
            }
            DieselErr::DatabaseError(DatabaseErrorKind::ForeignKeyViolation, info) => {
                let constraint = info.constraint_name().unwrap_or("foreign-key constraint");
                let mut details = std::collections::BTreeMap::new();
                details.insert(
                    "constraint".into(),
                    serde_json::Value::String(constraint.to_string()),
                );
                ApiError::unprocessable(
                    "foreign_key_violation",
                    format!("violates foreign-key constraint `{constraint}`"),
                )
                .with_details(details)
            }
            DieselErr::DatabaseError(DatabaseErrorKind::CheckViolation, info) => {
                let constraint = info.constraint_name().unwrap_or("check constraint");
                ApiError::unprocessable(
                    "check_violation",
                    format!("violates check constraint `{constraint}`"),
                )
            }
            DieselErr::DatabaseError(DatabaseErrorKind::NotNullViolation, info) => {
                let column = info.column_name().unwrap_or("required column");
                ApiError::unprocessable(
                    "not_null_violation",
                    format!("required column `{column}` is null"),
                )
            }
            DieselErr::NotFound => ApiError::not_found("not_found", internal_message),
            other => {
                tracing::error!("diesel error: {:?}", other);
                ApiError::internal(internal_message)
            }
        }
    }
}

impl From<diesel::result::Error> for ApiError {
    fn from(err: diesel::result::Error) -> Self {
        ApiError::from_diesel(err, "database error")
    }
}
