//! Generators API module for Brokkr.
//!
//! This module provides routes and handlers for managing generators,
//! including CRUD operations with appropriate access control.

use crate::dal::DAL;
use crate::api::v1::middleware::AuthPayload;
use axum::{
    extract::{Extension, Path, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use brokkr_models::models::generator::{Generator, NewGenerator};
use uuid::Uuid;
use crate::utils::pak;
use axum::http::StatusCode;
use brokkr_utils::logging::prelude::*;

/// Creates and returns the router for generator endpoints.
///
/// # Returns
///
/// A `Router` instance configured with the generator routes.
pub fn routes() -> Router<DAL> {
    info!("Setting up generator routes");
    Router::new()
        .route("/generators", get(list_generators))
        .route("/generators", post(create_generator))
        .route("/generators/:id", get(get_generator))
        .route("/generators/:id", put(update_generator))
        .route("/generators/:id", delete(delete_generator))
}

/// Lists all generators. Requires admin access.
///
/// # Arguments
///
/// * `dal` - The data access layer for database operations.
/// * `auth_payload` - The authentication payload containing user role information.
///
/// # Returns
///
/// A `Result` containing either a list of `Generator`s as JSON or an error response.
async fn list_generators(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
) -> Result<Json<Vec<Generator>>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    info!("Handling request to list generators");
    if !auth_payload.admin {
        warn!("Unauthorized attempt to list generators");
        return Err((
            axum::http::StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Admin access required"})),
        ));
    }

    match dal.generators().list() {
        Ok(generators) => {
            info!("Successfully retrieved {} generators", generators.len());
            Ok(Json(generators))
        }
        Err(e) => {
            error!("Failed to fetch generators: {:?}", e);
            Err((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch generators"})),
            ))
        }
    }
}

/// Creates a new generator. Requires admin access.
///
/// # Arguments
///
/// * `dal` - The data access layer for database operations.
/// * `auth_payload` - The authentication payload containing user role information.
/// * `new_generator` - The data for the new generator to be created.
///
/// # Returns
///
/// A `Result` containing either the created `Generator` and its PAK as JSON or an error response.
async fn create_generator(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Json(new_generator): Json<NewGenerator>,
) -> Result<Json<serde_json::Value>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    info!("Handling request to create a new generator");
    if !auth_payload.admin {
        warn!("Unauthorized attempt to create a generator");
        return Err((
            axum::http::StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Admin access required"})),
        ));
    }

    let (pak, pak_hash) = pak::create_pak().map_err(|e| {
        error!("Failed to create PAK: {:?}", e);
        (
            axum::http::StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to create PAK"})),
        )
    })?;

    match dal.generators().create(&new_generator) {
        Ok(generator) => {
            match dal.generators().update_pak_hash(generator.id, pak_hash) {
                Ok(updated_generator) => {
                    info!("Successfully created generator with ID: {}", updated_generator.id);
                    Ok(Json(serde_json::json!({
                        "generator": updated_generator,
                        "pak": pak
                    })))
                }
                Err(e) => {
                    error!("Failed to update generator PAK hash: {:?}", e);
                    Err((
                        axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                        Json(serde_json::json!({"error": "Failed to update generator PAK hash"})),
                    ))
                }
            }
        }
        Err(e) => {
            error!("Failed to create generator: {:?}", e);
            Err((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to create generator"})),
            ))
        }
    }
}

/// Retrieves a specific generator by ID.
///
/// # Arguments
///
/// * `dal` - The data access layer for database operations.
/// * `auth_payload` - The authentication payload containing user role information.
/// * `id` - The UUID of the generator to retrieve.
///
/// # Returns
///
/// A `Result` containing either the `Generator` as JSON or an error response.
async fn get_generator(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
) -> Result<Json<Generator>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    info!("Handling request to get generator with ID: {}", id);
    if !auth_payload.admin && auth_payload.generator != Some(id) {
        warn!("Unauthorized attempt to access generator with ID: {}", id);
        return Err((
            axum::http::StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Unauthorized access"})),
        ));
    }

    match dal.generators().get(id) {
        Ok(Some(generator)) => {
            info!("Successfully retrieved generator with ID: {}", id);
            Ok(Json(generator))
        }
        Ok(None) => {
            warn!("Generator not found with ID: {}", id);
            Err((
                axum::http::StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Generator not found"})),
            ))
        }
        Err(e) => {
            error!("Failed to fetch generator with ID {}: {:?}", id, e);
            Err((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch generator"})),
            ))
        }
    }
}

/// Updates an existing generator.
///
/// # Arguments
///
/// * `dal` - The data access layer for database operations.
/// * `auth_payload` - The authentication payload containing user role information.
/// * `id` - The UUID of the generator to update.
/// * `updated_generator` - The updated generator data.
///
/// # Returns
///
/// A `Result` containing either the updated `Generator` as JSON or an error response.
async fn update_generator(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
    Json(updated_generator): Json<Generator>,
) -> Result<Json<Generator>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    info!("Handling request to update generator with ID: {}", id);
    if !auth_payload.admin && auth_payload.generator != Some(id) {
        warn!("Unauthorized attempt to update generator with ID: {}", id);
        return Err((
            axum::http::StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Unauthorized access"})),
        ));
    }

    match dal.generators().update(id, &updated_generator) {
        Ok(generator) => {
            info!("Successfully updated generator with ID: {}", id);
            Ok(Json(generator))
        }
        Err(e) => {
            error!("Failed to update generator with ID {}: {:?}", id, e);
            Err((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to update generator"})),
            ))
        }
    }
}

/// Soft deletes a generator.
///
/// # Arguments
///
/// * `dal` - The data access layer for database operations.
/// * `auth_payload` - The authentication payload containing user role information.
/// * `id` - The UUID of the generator to delete.
///
/// # Returns
///
/// A `Result` containing either a success status code or an error response.
async fn delete_generator(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (axum::http::StatusCode, Json<serde_json::Value>)> {
    info!("Handling request to delete generator with ID: {}", id);
    if !auth_payload.admin && auth_payload.generator != Some(id) {
        warn!("Unauthorized attempt to delete generator with ID: {}", id);
        return Err((
            axum::http::StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Unauthorized access"})),
        ));
    }

    match dal.generators().soft_delete(id) {
        Ok(_) => {
            info!("Successfully deleted generator with ID: {}", id);
            Ok(StatusCode::NO_CONTENT)
        }
        Err(e) => {
            error!("Failed to delete generator with ID {}: {:?}", id, e);
            Err((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to delete generator"})),
            ))
        }
    }
}