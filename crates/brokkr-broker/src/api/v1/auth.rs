//! Authentication module for the Brokkr API v1.
//!
//! This module provides routes and handlers for authentication-related endpoints.

use crate::api::v1::middleware::AuthPayload;
use crate::api::v1::middleware::AuthResponse;
use crate::dal::DAL;
use axum::extract::Extension;
use axum::{routing::post, Json, Router};

/// Creates and returns the authentication routes for the API.
///
/// # Returns
///
/// A `Router` instance configured with the authentication routes.
pub fn routes() -> Router<DAL> {
    Router::new().route("/auth/pak", post(verify_pak))
}

/// Handles the PAK (Pre-Authentication Key) verification endpoint.
///
/// This function verifies the authentication payload and returns an `AuthResponse`
/// containing the authentication details.
///
/// # Arguments
///
/// * `auth_payload` - An `AuthPayload` extracted from the request's extension.
///
/// # Returns
///
/// A JSON response containing the `AuthResponse` with authentication details.
async fn verify_pak(Extension(auth_payload): Extension<AuthPayload>) -> Json<AuthResponse> {
    Json(AuthResponse {
        admin: auth_payload.admin,
        agent: auth_payload.agent.map(|id| id.to_string()),
        generator: auth_payload.generator.map(|id| id.to_string()),
    })
}
