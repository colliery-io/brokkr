/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Generators API module for Brokkr.

use crate::api::v1::error::{ApiError, ErrorResponse};
use crate::api::v1::middleware::AuthPayload;
use crate::dal::DAL;
use crate::utils::{audit, pak};
use brokkr_models::models::audit_logs::{
    ACTION_GENERATOR_CREATED, ACTION_GENERATOR_DELETED, ACTION_GENERATOR_UPDATED,
    ACTION_PAK_CREATED, ACTION_PAK_ROTATED, ACTOR_TYPE_ADMIN, ACTOR_TYPE_GENERATOR,
    RESOURCE_TYPE_GENERATOR, RESOURCE_TYPE_PAK,
};
use axum::http::StatusCode;
use axum::{
    extract::{Extension, Path, State},
    routing::{delete, get, post, put},
    Json, Router,
};
use brokkr_models::models::generator::{Generator, NewGenerator};
use serde::Serialize;
use tracing::{error, info, warn};
use utoipa::ToSchema;
use uuid::Uuid;

/// Response for a successful generator creation or PAK rotation.
#[derive(Serialize, ToSchema)]
pub struct CreateGeneratorResponse {
    /// The created generator.
    pub generator: Generator,
    /// The Pre-Authentication Key for the generator.
    pub pak: String,
}

pub fn routes() -> Router<DAL> {
    info!("Setting up generator routes");
    Router::new()
        .route("/generators", get(list_generators))
        .route("/generators", post(create_generator))
        .route("/generators/:id", get(get_generator))
        .route("/generators/:id", put(update_generator))
        .route("/generators/:id", delete(delete_generator))
        .route("/generators/:id/rotate-pak", post(rotate_generator_pak))
}

#[utoipa::path(
    get,
    path = "/generators",
    responses(
        (status = 200, description = "List all generators", body = Vec<Generator>),
        (status = 403, description = "Forbidden - Admin access required", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(("admin_pak" = [])),
    tag = "generators"
)]
async fn list_generators(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
) -> Result<Json<Vec<Generator>>, ApiError> {
    info!("Handling request to list generators");
    if !auth_payload.admin {
        warn!("Unauthorized attempt to list generators");
        return Err(ApiError::forbidden("admin_required", "admin access required"));
    }

    let generators = dal.generators().list().map_err(|e| {
        error!("Failed to fetch generators: {:?}", e);
        ApiError::internal("failed to fetch generators")
    })?;
    info!("Successfully retrieved {} generators", generators.len());
    Ok(Json(generators))
}


/// Resolves the audit actor for generator endpoints: the admin, or the
/// generator acting on itself.
fn audit_actor(auth_payload: &AuthPayload) -> (&'static str, Option<Uuid>) {
    if auth_payload.admin {
        (ACTOR_TYPE_ADMIN, None)
    } else {
        (ACTOR_TYPE_GENERATOR, auth_payload.generator)
    }
}

#[utoipa::path(
    post,
    path = "/generators",
    request_body = NewGenerator,
    responses(
        (status = 201, description = "Generator created successfully", body = CreateGeneratorResponse),
        (status = 400, description = "Invalid generator data", body = ErrorResponse),
        (status = 403, description = "Forbidden - Admin access required", body = ErrorResponse),
        (status = 409, description = "Generator name already taken", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    security(("admin_pak" = [])),
    tag = "generators"
)]
async fn create_generator(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Json(new_generator): Json<NewGenerator>,
) -> Result<(StatusCode, Json<CreateGeneratorResponse>), ApiError> {
    info!("Handling request to create a new generator");
    if !auth_payload.admin {
        warn!("Unauthorized attempt to create a generator");
        return Err(ApiError::forbidden("admin_required", "admin access required"));
    }

    let (pak_value, pak_hash) = pak::create_pak().map_err(|e| {
        error!("Failed to create PAK: {:?}", e);
        ApiError::internal("failed to create PAK")
    })?;

    let generator = dal.generators().create(&new_generator).map_err(|e| {
        warn!("Failed to create generator: {:?}", e);
        ApiError::from_diesel(e, "failed to create generator")
    })?;

    let updated_generator = dal
        .generators()
        .update_pak_hash(generator.id, pak_hash)
        .map_err(|e| {
            error!("Failed to update generator PAK hash: {:?}", e);
            ApiError::internal("failed to update generator PAK hash")
        })?;

    info!(
        "Successfully created generator with ID: {}",
        updated_generator.id
    );
    audit::log_action(
        ACTOR_TYPE_ADMIN,
        None,
        ACTION_GENERATOR_CREATED,
        RESOURCE_TYPE_GENERATOR,
        Some(updated_generator.id),
        Some(serde_json::json!({ "name": updated_generator.name })),
        None,
        None,
    );
    audit::log_action(
        ACTOR_TYPE_ADMIN,
        None,
        ACTION_PAK_CREATED,
        RESOURCE_TYPE_PAK,
        Some(updated_generator.id),
        Some(serde_json::json!({ "entity": "generator", "name": updated_generator.name })),
        None,
        None,
    );
    Ok((
        StatusCode::CREATED,
        Json(CreateGeneratorResponse {
            generator: updated_generator,
            pak: pak_value,
        }),
    ))
}

#[utoipa::path(
    get,
    path = "/generators/{id}",
    responses(
        (status = 200, description = "Get generator by id", body = Generator),
        (status = 403, description = "Forbidden - Unauthorized access", body = ErrorResponse),
        (status = 404, description = "Generator not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(("id" = Uuid, Path, description = "Generator id")),
    security(("admin_pak" = []), ("generator_pak" = [])),
    tag = "generators"
)]
async fn get_generator(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
) -> Result<Json<Generator>, ApiError> {
    info!("Handling request to get generator with ID: {}", id);
    if !auth_payload.admin && auth_payload.generator != Some(id) {
        warn!("Unauthorized attempt to access generator with ID: {}", id);
        return Err(ApiError::forbidden(
            "generator_not_owned",
            "not authorized to access this generator",
        ));
    }

    let generator = dal
        .generators()
        .get(id)
        .map_err(|e| {
            error!("Failed to fetch generator with ID {}: {:?}", id, e);
            ApiError::internal("failed to fetch generator")
        })?
        .ok_or_else(|| {
            warn!("Generator not found with ID: {}", id);
            ApiError::not_found("generator_not_found", "generator not found")
        })?;

    info!("Successfully retrieved generator with ID: {}", id);
    Ok(Json(generator))
}

#[utoipa::path(
    put,
    path = "/generators/{id}",
    request_body = Generator,
    responses(
        (status = 200, description = "Generator updated successfully", body = Generator),
        (status = 403, description = "Forbidden - Unauthorized access", body = ErrorResponse),
        (status = 404, description = "Generator not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(("id" = Uuid, Path, description = "Generator id")),
    security(("admin_pak" = []), ("generator_pak" = [])),
    tag = "generators"
)]
async fn update_generator(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
    Json(updated_generator): Json<Generator>,
) -> Result<Json<Generator>, ApiError> {
    info!("Handling request to update generator with ID: {}", id);
    if !auth_payload.admin && auth_payload.generator != Some(id) {
        warn!("Unauthorized attempt to update generator with ID: {}", id);
        return Err(ApiError::forbidden(
            "generator_not_owned",
            "not authorized to update this generator",
        ));
    }

    let generator = dal.generators().update(id, &updated_generator).map_err(|e| {
        error!("Failed to update generator with ID {}: {:?}", id, e);
        ApiError::internal("failed to update generator")
    })?;
    info!("Successfully updated generator with ID: {}", id);
    let (actor_type, actor_id) = audit_actor(&auth_payload);
    audit::log_action(
        actor_type,
        actor_id,
        ACTION_GENERATOR_UPDATED,
        RESOURCE_TYPE_GENERATOR,
        Some(id),
        Some(serde_json::json!({ "name": generator.name })),
        None,
        None,
    );
    Ok(Json(generator))
}

#[utoipa::path(
    delete,
    path = "/generators/{id}",
    responses(
        (status = 204, description = "Generator deleted successfully"),
        (status = 403, description = "Forbidden - Unauthorized access", body = ErrorResponse),
        (status = 404, description = "Generator not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(("id" = Uuid, Path, description = "Generator id")),
    security(("admin_pak" = []), ("generator_pak" = [])),
    tag = "generators"
)]
async fn delete_generator(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    info!("Handling request to delete generator with ID: {}", id);
    if !auth_payload.admin && auth_payload.generator != Some(id) {
        warn!("Unauthorized attempt to delete generator with ID: {}", id);
        return Err(ApiError::forbidden(
            "generator_not_owned",
            "not authorized to delete this generator",
        ));
    }

    let old_pak_hash = dal
        .generators()
        .get(id)
        .ok()
        .flatten()
        .and_then(|g| g.pak_hash);

    dal.generators().soft_delete(id).map_err(|e| {
        error!("Failed to delete generator with ID {}: {:?}", id, e);
        ApiError::internal("failed to delete generator")
    })?;

    info!("Successfully deleted generator with ID: {}", id);
    if let Some(ref hash) = old_pak_hash {
        dal.invalidate_auth_cache(hash);
    }
    let (actor_type, actor_id) = audit_actor(&auth_payload);
    audit::log_action(
        actor_type,
        actor_id,
        ACTION_GENERATOR_DELETED,
        RESOURCE_TYPE_GENERATOR,
        Some(id),
        None,
        None,
        None,
    );
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    post,
    path = "/generators/{id}/rotate-pak",
    responses(
        (status = 201, description = "Generator PAK rotated successfully", body = CreateGeneratorResponse),
        (status = 403, description = "Forbidden - Unauthorized access", body = ErrorResponse),
        (status = 404, description = "Generator not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse)
    ),
    params(("id" = Uuid, Path, description = "Generator id")),
    security(("admin_pak" = []), ("generator_pak" = [])),
    tag = "generators"
)]
async fn rotate_generator_pak(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
) -> Result<(StatusCode, Json<CreateGeneratorResponse>), ApiError> {
    info!("Handling request to rotate PAK for generator with ID: {}", id);

    if !auth_payload.admin && auth_payload.generator != Some(id) {
        warn!("Unauthorized attempt to rotate PAK for generator with ID: {}", id);
        return Err(ApiError::forbidden(
            "generator_not_owned",
            "not authorized to rotate this generator's PAK",
        ));
    }

    let old_pak_hash = dal
        .generators()
        .get(id)
        .map_err(|e| {
            error!("Failed to fetch generator with ID {}: {:?}", id, e);
            ApiError::internal("failed to fetch generator")
        })?
        .ok_or_else(|| ApiError::not_found("generator_not_found", "generator not found"))?
        .pak_hash;

    let (pak_value, pak_hash) = pak::create_pak().map_err(|e| {
        error!("Failed to create new PAK: {:?}", e);
        ApiError::internal("failed to create new PAK")
    })?;

    let updated_generator = dal.generators().update_pak_hash(id, pak_hash).map_err(|e| {
        error!("Failed to update generator PAK hash: {:?}", e);
        ApiError::internal("failed to update generator PAK hash")
    })?;

    info!("Successfully rotated PAK for generator with ID: {}", id);
    if let Some(ref hash) = old_pak_hash {
        dal.invalidate_auth_cache(hash);
    }

    let (actor_type, actor_id) = if auth_payload.admin {
        (ACTOR_TYPE_ADMIN, None)
    } else {
        (ACTOR_TYPE_GENERATOR, auth_payload.generator)
    };
    audit::log_action(
        actor_type,
        actor_id,
        ACTION_PAK_ROTATED,
        RESOURCE_TYPE_GENERATOR,
        Some(id),
        Some(serde_json::json!({
            "generator_name": updated_generator.name,
        })),
        None,
        None,
    );

    Ok((
        StatusCode::CREATED,
        Json(CreateGeneratorResponse {
            generator: updated_generator,
            pak: pak_value,
        }),
    ))
}
