//! Authentication module for the Brokkr API v1.
//!
//! This module provides routes and handlers for authentication-related endpoints.

use crate::api::v1::middleware::AuthPayload;
use crate::api::v1::middleware::AuthResponse;
use crate::dal::DAL;
use axum::extract::Extension;
use axum::http::StatusCode;
use axum::{routing::post, Json, Router};
use brokkr_utils::logging::prelude::*;

/// Creates and returns the authentication routes for the API.
pub fn routes() -> Router<DAL> {
    Router::new().route("/auth/pak", post(verify_pak))
}

/// Verifies a PAK (Personal Access Key) and returns an AuthResponse.
///
/// This function handles the authentication process for both admin and agent PAKs.
async fn verify_pak(Extension(auth_payload): Extension<AuthPayload>) -> Json<AuthResponse> {
    Json(AuthResponse {
        admin: auth_payload.admin,
        agent: auth_payload.agent.map(|id| id.to_string()),
        generator: auth_payload.generator.map(|id| id.to_string()),
    })
}
