//! Authentication middleware for the Brokkr API v1.
//!
//! This module provides middleware for authenticating requests using Pre-Authentication Keys (PAKs)
//! and handling different types of authenticated entities (admin, agent, generator).

use crate::dal::DAL;
use crate::utils::pak;
use axum::{
    body::Body,
    extract::State,
    http::{Request, StatusCode},
    middleware::Next,
    response::Response,
};
use brokkr_models::schema::admin_role;
use diesel::prelude::*;
use serde::Serialize;
use uuid::Uuid;

/// Represents the authenticated entity's payload.
#[derive(Clone, Debug)]
pub struct AuthPayload {
    /// Indicates if the authenticated entity is an admin.
    pub admin: bool,
    /// The UUID of the authenticated agent, if applicable.
    pub agent: Option<Uuid>,
    /// The UUID of the authenticated generator, if applicable.
    pub generator: Option<Uuid>,
}

/// Represents the response structure for authentication information.
#[derive(Serialize)]
pub struct AuthResponse {
    /// Indicates if the authenticated entity is an admin.
    pub admin: bool,
    /// The string representation of the agent's UUID, if applicable.
    pub agent: Option<String>,
    /// The string representation of the generator's UUID, if applicable.
    pub generator: Option<String>,
}

/// Middleware function for authenticating requests.
///
/// This function extracts the PAK from the Authorization header, verifies it,
/// and adds the resulting `AuthPayload` to the request's extensions.
///
/// # Arguments
///
/// * `dal` - The data access layer for database operations.
/// * `request` - The incoming HTTP request.
/// * `next` - The next middleware in the chain.
///
/// # Returns
///
/// A `Result` containing either the response from the next middleware or an error status code.
pub async fn auth_middleware<B>(
    State(dal): State<DAL>,
    mut request: Request<Body>,
    next: Next,
) -> Result<Response, StatusCode> {
    let pak = request
        .headers()
        .get("Authorization")
        .and_then(|header| header.to_str().ok())
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let auth_payload = verify_pak(&dal, pak).await?;

    request.extensions_mut().insert(auth_payload);

    Ok(next.run(request).await)
}

/// Verifies the provided PAK and returns the corresponding `AuthPayload`.
///
/// This function checks the PAK against admin roles, agents, and generators
/// to determine the type and permissions of the authenticated entity.
///
/// # Arguments
///
/// * `dal` - The data access layer for database operations.
/// * `pak` - The Pre-Authentication Key to verify.
///
/// # Returns
///
/// A `Result` containing either the `AuthPayload` or an error status code.
async fn verify_pak(dal: &DAL, pak: &str) -> Result<AuthPayload, StatusCode> {
    let conn = &mut dal
        .pool
        .get()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Check admin role
    let admin_key = admin_role::table
        .select(admin_role::pak_hash)
        .first::<String>(conn)
        .optional()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if let Some(admin_hash) = admin_key {
        if pak::verify_pak(pak.to_string(), admin_hash) {
            return Ok(AuthPayload {
                admin: true,
                agent: None,
                generator: None,
            });
        }
    }

    // Check agents
    let agents = dal
        .agents()
        .list()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    for agent in agents {
        if pak::verify_pak(pak.to_string(), agent.pak_hash) {
            return Ok(AuthPayload {
                admin: false,
                agent: Some(agent.id),
                generator: None,
            });
        }
    }

    // Check generators
    let generators = dal
        .generators()
        .list()
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    for generator in generators {
        if pak::verify_pak(pak.to_string(), generator.pak_hash.unwrap_or_default()) {
            return Ok(AuthPayload {
                admin: false,
                agent: None,
                generator: Some(generator.id),
            });
        }
    }

    Err(StatusCode::UNAUTHORIZED)
}
