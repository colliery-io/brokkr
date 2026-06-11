/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

use crate::api::v1::error::{ApiError, ErrorResponse};
use crate::api::v1::middleware::AuthPayload;
use crate::dal::DAL;
use crate::metrics;
use crate::utils::audit;
use crate::utils::matching::template_matches_stack;
use crate::utils::templating;
use crate::ws::{ConnectionRegistry, push_stack_changed_to_targets};
use axum::{
    Json, Router,
    body::Bytes,
    extract::{Extension, Path, Query, State},
    http::{HeaderMap, StatusCode, header::CONTENT_TYPE},
    routing::{delete, get, post},
};
use brokkr_models::models::audit_logs::{
    ACTION_STACK_CREATED, ACTION_STACK_DELETED, ACTION_STACK_UPDATED, ACTOR_TYPE_ADMIN,
    RESOURCE_TYPE_STACK,
};
use brokkr_models::models::deployment_objects::{DeploymentObject, NewDeploymentObject};
use brokkr_models::models::rendered_deployment_objects::NewRenderedDeploymentObject;
use brokkr_models::models::stack_annotations::{NewStackAnnotation, StackAnnotation};
use brokkr_models::models::stack_labels::{NewStackLabel, StackLabel};
use brokkr_models::models::stacks::{NewStack, Stack};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tracing::{error, info, instrument, warn};
use utoipa::ToSchema;
use uuid::Uuid;

pub fn routes() -> Router<DAL> {
    info!("Setting up stack routes");
    Router::new()
        .route("/stacks", get(list_stacks).post(create_stack))
        .route(
            "/stacks/:id",
            get(get_stack).put(update_stack).delete(delete_stack),
        )
        .route(
            "/stacks/:id/deployment-objects",
            get(list_deployment_objects).post(create_deployment_object),
        )
        .route(
            "/stacks/:id/deployment-objects/from-template",
            post(instantiate_template),
        )
        .route("/stacks/:id/labels", get(list_labels).post(add_label))
        .route("/stacks/:id/labels/:label", delete(remove_label))
        .route(
            "/stacks/:id/annotations",
            get(list_annotations).post(add_annotation),
        )
        .route("/stacks/:id/annotations/:key", delete(remove_annotation))
        // WS-10: short-lived telemetry history (BROKKR-I-0019). Bounded by
        // the 6h retention ceiling (project_log_retention_stance).
        .route("/stacks/:id/events", get(list_telemetry_events))
        .route("/stacks/:id/logs", get(list_telemetry_logs))
}

/// Fetch a stack or return 404; also enforces admin-or-generator-owner access.
async fn fetch_owned_stack(
    dal: &DAL,
    auth: &AuthPayload,
    stack_id: Uuid,
) -> Result<Stack, ApiError> {
    let mut stacks = dal.stacks().get(vec![stack_id]).map_err(|e| {
        error!("Failed to fetch stack {}: {:?}", stack_id, e);
        ApiError::internal("failed to fetch stack")
    })?;

    let stack = stacks
        .pop()
        .ok_or_else(|| ApiError::not_found("stack_not_found", "stack not found"))?;

    if !auth.admin && auth.generator != Some(stack.generator_id) {
        warn!("Unauthorized access to stack {}", stack_id);
        return Err(ApiError::forbidden(
            "stack_not_owned",
            "not authorized to access this stack",
        ));
    }
    Ok(stack)
}

#[utoipa::path(
    get,
    path = "/stacks",
    tag = "stacks",
    responses(
        (status = 200, description = "List of stacks (admin: all; generator: own)", body = Vec<Stack>),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("admin_pak" = []), ("generator_pak" = []))
)]
#[instrument(skip(dal, auth_payload), fields(admin = auth_payload.admin))]
async fn list_stacks(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
) -> Result<Json<Vec<Stack>>, ApiError> {
    info!("Handling request to list stacks");

    let stacks = if auth_payload.admin {
        dal.stacks().list().map_err(|e| {
            error!("Failed to fetch stacks: {:?}", e);
            ApiError::internal("failed to fetch stacks")
        })?
    } else if let Some(generator_id) = auth_payload.generator {
        dal.stacks().list_for_generator(generator_id).map_err(|e| {
            error!(
                "Failed to fetch stacks for generator {}: {:?}",
                generator_id, e
            );
            ApiError::internal("failed to fetch stacks")
        })?
    } else {
        return Err(ApiError::forbidden(
            "stacks_list_denied",
            "admin or generator access required",
        ));
    };

    info!("Successfully retrieved {} stacks", stacks.len());
    if auth_payload.admin {
        metrics::set_stacks_total(stacks.len() as i64);
    }
    Ok(Json(stacks))
}

#[utoipa::path(
    post,
    path = "/stacks",
    tag = "stacks",
    request_body = NewStack,
    responses(
        (status = 201, description = "Stack created", body = Stack),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("admin_pak" = []), ("generator_pak" = []))
)]
async fn create_stack(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Json(new_stack): Json<NewStack>,
) -> Result<(StatusCode, Json<Stack>), ApiError> {
    info!("Handling request to create a new stack");
    if !auth_payload.admin && auth_payload.generator.is_none() {
        return Err(ApiError::forbidden(
            "stack_create_denied",
            "admin or generator access required",
        ));
    }
    if let Some(generator_id) = auth_payload.generator
        && generator_id != new_stack.generator_id
    {
        return Err(ApiError::forbidden(
            "stack_generator_mismatch",
            "generator can only create stacks for itself",
        ));
    }

    let stack = dal.stacks().create(&new_stack).map_err(|e| {
        warn!("Failed to create stack: {:?}", e);
        ApiError::from_diesel(e, "failed to create stack")
    })?;
    info!("Successfully created stack with ID: {}", stack.id);

    audit::log_action(
        ACTOR_TYPE_ADMIN,
        None,
        ACTION_STACK_CREATED,
        RESOURCE_TYPE_STACK,
        Some(stack.id),
        Some(serde_json::json!({
            "name": stack.name,
            "generator_id": stack.generator_id,
        })),
        None,
        None,
    );

    Ok((StatusCode::CREATED, Json(stack)))
}

#[utoipa::path(
    get,
    path = "/stacks/{id}",
    tag = "stacks",
    params(("id" = Uuid, Path, description = "Stack ID")),
    responses(
        (status = 200, description = "Stack found", body = Stack),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Stack not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("admin_pak" = []), ("generator_pak" = []))
)]
async fn get_stack(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
) -> Result<Json<Stack>, ApiError> {
    info!("Handling request to get stack with ID: {}", id);
    let stack = fetch_owned_stack(&dal, &auth_payload, id).await?;
    Ok(Json(stack))
}

#[utoipa::path(
    put,
    path = "/stacks/{id}",
    tag = "stacks",
    params(("id" = Uuid, Path, description = "Stack ID")),
    request_body = Stack,
    responses(
        (status = 200, description = "Stack updated", body = Stack),
        (status = 400, description = "Bad request", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Stack not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("admin_pak" = []), ("generator_pak" = []))
)]
async fn update_stack(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
    Json(updated_stack): Json<Stack>,
) -> Result<Json<Stack>, ApiError> {
    info!("Handling request to update stack with ID: {}", id);
    fetch_owned_stack(&dal, &auth_payload, id).await?;
    if id != updated_stack.id {
        return Err(ApiError::bad_request(
            "stack_id_mismatch",
            "stack ID mismatch",
        ));
    }

    let stack = dal.stacks().update(id, &updated_stack).map_err(|e| {
        error!("Failed to update stack with ID {}: {:?}", id, e);
        ApiError::internal("failed to update stack")
    })?;
    info!("Successfully updated stack with ID: {}", id);

    audit::log_action(
        ACTOR_TYPE_ADMIN,
        None,
        ACTION_STACK_UPDATED,
        RESOURCE_TYPE_STACK,
        Some(id),
        Some(serde_json::json!({ "name": stack.name })),
        None,
        None,
    );

    Ok(Json(stack))
}

#[utoipa::path(
    delete,
    path = "/stacks/{id}",
    tag = "stacks",
    params(("id" = Uuid, Path, description = "Stack ID")),
    responses(
        (status = 204, description = "Stack deleted"),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Stack not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("admin_pak" = []), ("generator_pak" = []))
)]
async fn delete_stack(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, ApiError> {
    info!("Handling request to delete stack with ID: {}", id);
    fetch_owned_stack(&dal, &auth_payload, id).await?;
    dal.stacks().soft_delete(id).map_err(|e| {
        error!("Failed to delete stack with ID {}: {:?}", id, e);
        ApiError::internal("failed to delete stack")
    })?;
    info!("Successfully deleted stack with ID: {}", id);
    audit::log_action(
        ACTOR_TYPE_ADMIN,
        None,
        ACTION_STACK_DELETED,
        RESOURCE_TYPE_STACK,
        Some(id),
        None,
        None,
        None,
    );
    Ok(StatusCode::NO_CONTENT)
}

#[utoipa::path(
    get,
    path = "/stacks/{id}/deployment-objects",
    tag = "stacks",
    params(("id" = Uuid, Path, description = "Stack ID")),
    responses(
        (status = 200, description = "List of deployment objects for the stack", body = Vec<DeploymentObject>),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Stack not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("admin_pak" = []), ("generator_pak" = []))
)]
pub async fn list_deployment_objects(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(stack_id): Path<Uuid>,
) -> Result<Json<Vec<DeploymentObject>>, ApiError> {
    fetch_owned_stack(&dal, &auth_payload, stack_id).await?;
    let objects = dal
        .deployment_objects()
        .list_for_stack(stack_id)
        .map_err(|_| ApiError::internal("failed to fetch deployment objects"))?;
    metrics::set_deployment_objects_total(objects.len() as i64);
    Ok(Json(objects))
}

/// Wire DTO for creating a deployment object via the public API.
///
/// Distinct from [`brokkr_models::models::deployment_objects::NewDeploymentObject`],
/// which carries server-derived fields (e.g. `yaml_checksum`).
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateDeploymentObjectRequest {
    /// YAML content of the deployment.
    pub yaml_content: String,
    /// Optional. Defaults to false.
    #[serde(default)]
    pub is_deletion_marker: bool,
}

/// Query parameters for the deployment-object create endpoint. Used by the
/// raw-YAML body path (`Content-Type: application/yaml`), where the flag
/// cannot ride in the body (BROKKR-T-0194).
#[derive(Debug, Default, Deserialize, utoipa::IntoParams)]
pub struct CreateDeploymentObjectQuery {
    /// Marks the submission as a deletion marker. Only consulted on the
    /// raw-YAML path; on the JSON path the body field wins.
    #[serde(default)]
    pub deletion_marker: Option<bool>,
}

/// Whether a `Content-Type` denotes a raw YAML body rather than the JSON
/// envelope. Treats `application/yaml`, `text/yaml`, and the legacy
/// `application/x-yaml` as YAML; everything else (including no header) is
/// JSON, preserving backward compatibility.
fn content_type_is_yaml(headers: &HeaderMap) -> bool {
    headers
        .get(CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .map(|ct| {
            let mime = ct.split(';').next().unwrap_or("").trim();
            matches!(
                mime,
                "application/yaml" | "text/yaml" | "application/x-yaml"
            )
        })
        .unwrap_or(false)
}

/// Resolves the request body into `(yaml_content, is_deletion_marker)` based
/// on the content type. Pure and unit-testable.
fn resolve_create_body(
    headers: &HeaderMap,
    query: &CreateDeploymentObjectQuery,
    body: &[u8],
) -> Result<(String, bool), ApiError> {
    if content_type_is_yaml(headers) {
        let yaml_content = String::from_utf8(body.to_vec()).map_err(|_| {
            ApiError::bad_request("invalid_deployment_object", "request body is not valid UTF-8")
        })?;
        Ok((yaml_content, query.deletion_marker.unwrap_or(false)))
    } else {
        let req: CreateDeploymentObjectRequest = serde_json::from_slice(body).map_err(|e| {
            ApiError::bad_request("invalid_deployment_object", format!("invalid JSON body: {e}"))
        })?;
        Ok((req.yaml_content, req.is_deletion_marker))
    }
}

/// Validates the manifest body at ingest so malformed YAML fails here with a
/// clear 400 instead of late at agent apply. A deletion marker may be empty;
/// otherwise the body must parse as one or more non-null YAML documents.
fn validate_manifest_yaml(yaml_content: &str, is_deletion_marker: bool) -> Result<(), ApiError> {
    if is_deletion_marker && yaml_content.trim().is_empty() {
        return Ok(());
    }
    if yaml_content.trim().is_empty() {
        return Err(ApiError::bad_request(
            "invalid_deployment_object",
            "YAML content cannot be empty",
        ));
    }
    let mut saw_document = false;
    for doc in serde_yaml::Deserializer::from_str(yaml_content) {
        let value = serde_yaml::Value::deserialize(doc).map_err(|e| {
            ApiError::bad_request("invalid_deployment_object", format!("invalid YAML: {e}"))
        })?;
        if !value.is_null() {
            saw_document = true;
        }
    }
    if !saw_document {
        return Err(ApiError::bad_request(
            "invalid_deployment_object",
            "YAML content has no documents",
        ));
    }
    Ok(())
}

#[utoipa::path(
    post,
    path = "/stacks/{id}/deployment-objects",
    tag = "stacks",
    params(("id" = Uuid, Path, description = "Stack ID"), CreateDeploymentObjectQuery),
    request_body = CreateDeploymentObjectRequest,
    responses(
        (status = 201, description = "Deployment object created", body = DeploymentObject),
        (status = 400, description = "Invalid request", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Stack not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("admin_pak" = []), ("generator_pak" = []))
)]
pub async fn create_deployment_object(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Extension(ws_registry): Extension<Arc<ConnectionRegistry>>,
    Path(stack_id): Path<Uuid>,
    Query(query): Query<CreateDeploymentObjectQuery>,
    headers: HeaderMap,
    // Body-consuming extractor must come last.
    body: Bytes,
) -> Result<(StatusCode, Json<DeploymentObject>), ApiError> {
    let stack = fetch_owned_stack(&dal, &auth_payload, stack_id).await?;
    let (yaml_content, is_deletion_marker) = resolve_create_body(&headers, &query, &body)?;
    validate_manifest_yaml(&yaml_content, is_deletion_marker)?;
    let new_object = NewDeploymentObject::new(stack_id, yaml_content, is_deletion_marker)
        .map_err(|e| ApiError::bad_request("invalid_deployment_object", e))?;
    let object = dal
        .deployment_objects()
        .create(&new_object)
        .map_err(|_| ApiError::internal("failed to create deployment object"))?;

    // Post-commit: notify every connected agent that targets this stack so
    // they can reconcile the new deployment object immediately rather than
    // waiting for the next REST polling tick.
    push_stack_changed_to_targets(&ws_registry, &dal, &stack);

    Ok((StatusCode::CREATED, Json(object)))
}

async fn is_authorized_for_stack(
    dal: &DAL,
    auth_payload: &AuthPayload,
    stack_id: Uuid,
) -> Result<bool, ApiError> {
    if auth_payload.admin {
        return Ok(true);
    }
    let stacks = dal
        .stacks()
        .get(vec![stack_id])
        .map_err(|_| ApiError::internal("failed to fetch stack"))?;
    let stack = stacks
        .first()
        .ok_or_else(|| ApiError::not_found("stack_not_found", "stack not found"))?;

    if auth_payload.generator == Some(stack.generator_id) {
        return Ok(true);
    }
    if let Some(agent_id) = auth_payload.agent {
        let agent_targets = dal
            .agent_targets()
            .list_for_agent(agent_id)
            .map_err(|_| ApiError::internal("failed to fetch agent targets"))?;
        if agent_targets.iter().any(|t| t.stack_id == stack_id) {
            return Ok(true);
        }
    }
    Ok(false)
}

#[utoipa::path(
    get,
    path = "/stacks/{id}/labels",
    tag = "stacks",
    operation_id = "stacks_list_labels",
    params(("id" = Uuid, Path, description = "Stack ID")),
    responses(
        (status = 200, description = "List of stack labels", body = Vec<StackLabel>),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Stack not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("admin_pak" = []), ("generator_pak" = []), ("agent_pak" = []))
)]
pub async fn list_labels(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(stack_id): Path<Uuid>,
) -> Result<Json<Vec<StackLabel>>, ApiError> {
    if !is_authorized_for_stack(&dal, &auth_payload, stack_id).await? {
        return Err(ApiError::forbidden(
            "stack_not_accessible",
            "not authorized to access this stack",
        ));
    }
    let labels = dal
        .stack_labels()
        .list_for_stack(stack_id)
        .map_err(|_| ApiError::internal("failed to fetch stack labels"))?;
    Ok(Json(labels))
}

#[utoipa::path(
    post,
    path = "/stacks/{id}/labels",
    tag = "stacks",
    operation_id = "stacks_add_label",
    params(("id" = Uuid, Path, description = "Stack ID")),
    request_body(content = String, content_type = "application/json", description = "JSON-encoded label string, e.g. \"mylabel\""),
    responses(
        (status = 201, description = "Label added", body = StackLabel),
        (status = 400, description = "Invalid label", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Stack not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("admin_pak" = []), ("generator_pak" = []))
)]
pub async fn add_label(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(stack_id): Path<Uuid>,
    Json(label): Json<String>,
) -> Result<(StatusCode, Json<StackLabel>), ApiError> {
    fetch_owned_stack(&dal, &auth_payload, stack_id).await?;
    let new_label = NewStackLabel::new(stack_id, label)
        .map_err(|e| ApiError::bad_request("invalid_label", e))?;
    // Route the DB error through the standard conversion so a re-added label
    // (UNIQUE (stack_id, label)) surfaces as 409 unique_violation rather than a
    // blanket 500 — idempotent callers like `apply` rely on the 409 to treat a
    // label that already exists as a no-op.
    let label = dal
        .stack_labels()
        .create(&new_label)
        .map_err(|e| ApiError::from_diesel(e, "failed to add stack label"))?;
    Ok((StatusCode::CREATED, Json(label)))
}

#[utoipa::path(
    delete,
    path = "/stacks/{id}/labels/{label}",
    tag = "stacks",
    operation_id = "stacks_remove_label",
    params(
        ("id" = Uuid, Path, description = "Stack ID"),
        ("label" = String, Path, description = "Label to remove"),
    ),
    responses(
        (status = 204, description = "Label removed"),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Stack or label not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("admin_pak" = []), ("generator_pak" = []))
)]
pub async fn remove_label(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path((stack_id, label)): Path<(Uuid, String)>,
) -> Result<StatusCode, ApiError> {
    fetch_owned_stack(&dal, &auth_payload, stack_id).await?;
    let deleted = dal
        .stack_labels()
        .delete_by_stack_and_label(stack_id, &label)
        .map_err(|_| ApiError::internal("failed to remove stack label"))?;
    if deleted > 0 {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::not_found(
            "stack_label_not_found",
            "label not found",
        ))
    }
}

#[utoipa::path(
    get,
    path = "/stacks/{id}/annotations",
    tag = "stacks",
    operation_id = "stacks_list_annotations",
    params(("id" = Uuid, Path, description = "Stack ID")),
    responses(
        (status = 200, description = "List of stack annotations", body = Vec<StackAnnotation>),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Stack not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("admin_pak" = []), ("generator_pak" = []), ("agent_pak" = []))
)]
pub async fn list_annotations(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(stack_id): Path<Uuid>,
) -> Result<Json<Vec<StackAnnotation>>, ApiError> {
    if !is_authorized_for_stack(&dal, &auth_payload, stack_id).await? {
        return Err(ApiError::forbidden(
            "stack_not_accessible",
            "not authorized to access this stack",
        ));
    }
    let annotations = dal
        .stack_annotations()
        .list_for_stack(stack_id)
        .map_err(|_| ApiError::internal("failed to fetch stack annotations"))?;
    Ok(Json(annotations))
}

#[utoipa::path(
    post,
    path = "/stacks/{id}/annotations",
    tag = "stacks",
    operation_id = "stacks_add_annotation",
    params(("id" = Uuid, Path, description = "Stack ID")),
    request_body = NewStackAnnotation,
    responses(
        (status = 201, description = "Annotation added", body = StackAnnotation),
        (status = 400, description = "Invalid annotation", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Stack not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("admin_pak" = []), ("generator_pak" = []))
)]
pub async fn add_annotation(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(stack_id): Path<Uuid>,
    Json(new_annotation): Json<NewStackAnnotation>,
) -> Result<(StatusCode, Json<StackAnnotation>), ApiError> {
    fetch_owned_stack(&dal, &auth_payload, stack_id).await?;
    if new_annotation.stack_id != stack_id {
        return Err(ApiError::bad_request(
            "stack_id_mismatch",
            "stack ID mismatch",
        ));
    }
    let annotation = dal
        .stack_annotations()
        .create(&new_annotation)
        .map_err(|_| ApiError::internal("failed to add stack annotation"))?;
    Ok((StatusCode::CREATED, Json(annotation)))
}

#[utoipa::path(
    delete,
    path = "/stacks/{id}/annotations/{key}",
    tag = "stacks",
    operation_id = "stacks_remove_annotation",
    params(
        ("id" = Uuid, Path, description = "Stack ID"),
        ("key" = String, Path, description = "Annotation key to remove"),
    ),
    responses(
        (status = 204, description = "Annotation removed"),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Stack or annotation not found", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("admin_pak" = []), ("generator_pak" = []))
)]
pub async fn remove_annotation(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path((stack_id, key)): Path<(Uuid, String)>,
) -> Result<StatusCode, ApiError> {
    fetch_owned_stack(&dal, &auth_payload, stack_id).await?;
    let deleted = dal
        .stack_annotations()
        .delete_by_stack_and_key(stack_id, &key)
        .map_err(|_| ApiError::internal("failed to remove stack annotation"))?;
    if deleted > 0 {
        Ok(StatusCode::NO_CONTENT)
    } else {
        Err(ApiError::not_found(
            "stack_annotation_not_found",
            "annotation not found",
        ))
    }
}

#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct TemplateInstantiationRequest {
    pub template_id: Uuid,
    pub parameters: serde_json::Value,
}

#[utoipa::path(
    post,
    path = "/stacks/{stack_id}/deployment-objects/from-template",
    tag = "stacks",
    params(("stack_id" = Uuid, Path, description = "Stack ID")),
    request_body = TemplateInstantiationRequest,
    responses(
        (status = 201, description = "Deployment object created", body = DeploymentObject),
        (status = 400, description = "Invalid parameters or template rendering failed", body = ErrorResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Template or stack not found", body = ErrorResponse),
        (status = 422, description = "Template labels don't match stack", body = ErrorResponse),
        (status = 500, description = "Internal server error", body = ErrorResponse),
    ),
    security(("admin_pak" = []), ("generator_pak" = []))
)]
async fn instantiate_template(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(stack_id): Path<Uuid>,
    Json(request): Json<TemplateInstantiationRequest>,
) -> Result<(StatusCode, Json<DeploymentObject>), ApiError> {
    info!(
        "Handling template instantiation: template={}, stack={}",
        request.template_id, stack_id
    );

    let stack = fetch_owned_stack(&dal, &auth_payload, stack_id).await?;

    let template = dal
        .templates()
        .get(request.template_id)
        .map_err(|e| {
            error!("Failed to fetch template {}: {:?}", request.template_id, e);
            ApiError::internal("failed to fetch template")
        })?
        .ok_or_else(|| ApiError::not_found("template_not_found", "template not found"))?;

    let template_labels: Vec<String> = dal
        .template_labels()
        .list_for_template(template.id)
        .map_err(|_| ApiError::internal("failed to fetch template labels"))?
        .into_iter()
        .map(|l| l.label)
        .collect();
    let template_annotations: Vec<(String, String)> = dal
        .template_annotations()
        .list_for_template(template.id)
        .map_err(|_| ApiError::internal("failed to fetch template annotations"))?
        .into_iter()
        .map(|a| (a.key, a.value))
        .collect();
    let stack_labels: Vec<String> = dal
        .stack_labels()
        .list_for_stack(stack_id)
        .map_err(|_| ApiError::internal("failed to fetch stack labels"))?
        .into_iter()
        .map(|l| l.label)
        .collect();
    let stack_annotations: Vec<(String, String)> = dal
        .stack_annotations()
        .list_for_stack(stack_id)
        .map_err(|_| ApiError::internal("failed to fetch stack annotations"))?
        .into_iter()
        .map(|a| (a.key, a.value))
        .collect();

    let match_result = template_matches_stack(
        &template_labels,
        &template_annotations,
        &stack_labels,
        &stack_annotations,
    );
    if !match_result.matches {
        let mut details = std::collections::BTreeMap::new();
        details.insert(
            "missing_labels".into(),
            serde_json::json!(match_result.missing_labels),
        );
        details.insert(
            "missing_annotations".into(),
            serde_json::json!(match_result.missing_annotations),
        );
        return Err(ApiError {
            status: StatusCode::UNPROCESSABLE_ENTITY,
            code: "template_stack_mismatch".into(),
            message: "template labels do not match stack".into(),
            details: Some(details),
        });
    }

    if let Err(errors) =
        templating::validate_parameters(&template.parameters_schema, &request.parameters)
    {
        let error_messages: Vec<String> = errors.iter().map(|e| e.to_string()).collect();
        let mut details = std::collections::BTreeMap::new();
        details.insert(
            "validation_errors".into(),
            serde_json::json!(error_messages),
        );
        return Err(
            ApiError::bad_request("invalid_parameters", "invalid parameters").with_details(details),
        );
    }

    let rendered_yaml =
        templating::render_template(&template.template_content, &request.parameters).map_err(
            |e| {
                error!("Failed to render template: {:?}", e);
                ApiError::bad_request("template_render_failed", e.to_string())
            },
        )?;

    let new_deployment_object = NewDeploymentObject::new(stack_id, rendered_yaml.clone(), false)
        .map_err(|e| ApiError::bad_request("invalid_deployment_object", e))?;
    let deployment_object = dal
        .deployment_objects()
        .create(&new_deployment_object)
        .map_err(|e| {
            error!("Failed to insert deployment object: {:?}", e);
            ApiError::internal("failed to create deployment object")
        })?;

    let parameters_json = serde_json::to_string(&request.parameters)
        .map_err(|_| ApiError::internal("failed to serialize parameters"))?;
    let provenance = NewRenderedDeploymentObject::new(
        deployment_object.id,
        template.id,
        template.version,
        parameters_json,
    )
    .map_err(|e| ApiError::internal(format!("failed to create provenance record: {}", e)))?;
    dal.rendered_deployment_objects()
        .create(&provenance)
        .map_err(|e| {
            error!("Failed to insert provenance record: {:?}", e);
            ApiError::internal("failed to create provenance record")
        })?;

    info!(
        "Successfully instantiated template {} into deployment object {} for stack {} (gen: {:?})",
        template.id, deployment_object.id, stack_id, stack.generator_id
    );
    Ok((StatusCode::CREATED, Json(deployment_object)))
}

// =============================================================================
// WS-10: telemetry history endpoints
//
// Short-lived operational buffer with a hard 6h retention ceiling. The
// retention metadata is included in every response so callers — including
// the UI and the generated SDKs — render the right "this is not your
// long-term log store" UX (NFR-007). See project_log_retention_stance.
// =============================================================================

use crate::ws::HARD_RETENTION_CEILING;
use brokkr_models::models::agent_k8s_events::AgentK8sEvent;
use brokkr_models::models::agent_pod_logs::AgentPodLog;

/// Default page size for the telemetry history endpoints.
const TELEMETRY_DEFAULT_LIMIT: i64 = 500;
/// Maximum page size — protect the broker from "give me everything" callers.
const TELEMETRY_MAX_LIMIT: i64 = 5000;

#[derive(Debug, serde::Deserialize, utoipa::IntoParams)]
pub struct TelemetryHistoryQuery {
    /// Earliest `created_at` to include (ISO-8601). Defaults to
    /// `now - retention_ceiling_seconds`. Values older than the ceiling
    /// are silently clamped: only the retained window can be returned.
    #[serde(default)]
    pub since: Option<chrono::DateTime<chrono::Utc>>,
    /// Maximum rows to return. Defaults to 500; capped at 5000.
    #[serde(default)]
    pub limit: Option<i64>,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct RetentionInfo {
    /// Hard upper bound on retention. Never exceeds 21600 (6h).
    pub retention_ceiling_seconds: u64,
    /// Effective retention window for the stack. <= ceiling.
    pub effective_retention_seconds: u64,
    /// Server-side timestamp of the oldest row currently retained for
    /// this stack, or null when no rows exist in the window.
    pub oldest_available_ts: Option<chrono::DateTime<chrono::Utc>>,
    /// Recommended sink for long-term centralisation. Brokkr is NOT a
    /// log warehouse — see project_log_retention_stance.
    pub long_term_sink_hint: &'static str,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct K8sEventHistoryResponse {
    pub retention: RetentionInfo,
    pub events: Vec<AgentK8sEvent>,
}

#[derive(Debug, serde::Serialize, utoipa::ToSchema)]
pub struct PodLogHistoryResponse {
    pub retention: RetentionInfo,
    pub lines: Vec<AgentPodLog>,
}

fn retention_info(oldest: Option<chrono::DateTime<chrono::Utc>>) -> RetentionInfo {
    RetentionInfo {
        retention_ceiling_seconds: HARD_RETENTION_CEILING.as_secs(),
        effective_retention_seconds: HARD_RETENTION_CEILING.as_secs(),
        oldest_available_ts: oldest,
        long_term_sink_hint: "Brokkr retains telemetry for at most 6 hours. \
                              For long-term log centralisation, ship to Datadog or equivalent.",
    }
}

fn clamp_since(since: Option<chrono::DateTime<chrono::Utc>>) -> chrono::DateTime<chrono::Utc> {
    let ceiling_ago =
        chrono::Utc::now() - chrono::Duration::from_std(HARD_RETENTION_CEILING).unwrap_or_default();
    match since {
        Some(s) if s > ceiling_ago => s,
        _ => ceiling_ago,
    }
}

fn clamp_limit(limit: Option<i64>) -> i64 {
    let l = limit.unwrap_or(TELEMETRY_DEFAULT_LIMIT);
    l.clamp(1, TELEMETRY_MAX_LIMIT)
}

#[utoipa::path(
    get,
    path = "/stacks/{id}/events",
    tag = "stack-telemetry",
    params(
        ("id" = Uuid, Path, description = "Stack ID"),
        TelemetryHistoryQuery,
    ),
    responses(
        (status = 200, description = "Retained kube events for this stack", body = K8sEventHistoryResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Stack not found", body = ErrorResponse),
    ),
    security(("admin_pak" = []), ("generator_pak" = []))
)]
pub async fn list_telemetry_events(
    State(dal): State<DAL>,
    Extension(auth): Extension<AuthPayload>,
    Path(stack_id): Path<Uuid>,
    axum::extract::Query(q): axum::extract::Query<TelemetryHistoryQuery>,
) -> Result<Json<K8sEventHistoryResponse>, ApiError> {
    fetch_owned_stack(&dal, &auth, stack_id).await?;
    let since = clamp_since(q.since);
    let limit = clamp_limit(q.limit);
    let events = dal
        .agent_k8s_events()
        .list_for_stack(stack_id, since, limit)
        .map_err(|e| {
            error!("failed to list k8s events for stack {}: {:?}", stack_id, e);
            ApiError::internal("failed to list telemetry events")
        })?;
    let oldest = events.last().map(|e| e.created_at);
    Ok(Json(K8sEventHistoryResponse {
        retention: retention_info(oldest),
        events,
    }))
}

#[utoipa::path(
    get,
    path = "/stacks/{id}/logs",
    tag = "stack-telemetry",
    params(
        ("id" = Uuid, Path, description = "Stack ID"),
        TelemetryHistoryQuery,
    ),
    responses(
        (status = 200, description = "Retained pod log lines for this stack", body = PodLogHistoryResponse),
        (status = 403, description = "Forbidden", body = ErrorResponse),
        (status = 404, description = "Stack not found", body = ErrorResponse),
    ),
    security(("admin_pak" = []), ("generator_pak" = []))
)]
pub async fn list_telemetry_logs(
    State(dal): State<DAL>,
    Extension(auth): Extension<AuthPayload>,
    Path(stack_id): Path<Uuid>,
    axum::extract::Query(q): axum::extract::Query<TelemetryHistoryQuery>,
) -> Result<Json<PodLogHistoryResponse>, ApiError> {
    fetch_owned_stack(&dal, &auth, stack_id).await?;
    let since = clamp_since(q.since);
    let limit = clamp_limit(q.limit);
    let lines = dal
        .agent_pod_logs()
        .list_for_stack(stack_id, since, limit)
        .map_err(|e| {
            error!("failed to list pod logs for stack {}: {:?}", stack_id, e);
            ApiError::internal("failed to list telemetry logs")
        })?;
    let oldest = lines.last().map(|l| l.created_at);
    Ok(Json(PodLogHistoryResponse {
        retention: retention_info(oldest),
        lines,
    }))
}

#[cfg(test)]
mod create_body_tests {
    use super::*;
    use axum::http::HeaderValue;

    fn headers_with(ct: Option<&str>) -> HeaderMap {
        let mut h = HeaderMap::new();
        if let Some(ct) = ct {
            h.insert(CONTENT_TYPE, HeaderValue::from_str(ct).unwrap());
        }
        h
    }

    #[test]
    fn content_type_detection() {
        assert!(content_type_is_yaml(&headers_with(Some("application/yaml"))));
        assert!(content_type_is_yaml(&headers_with(Some("text/yaml"))));
        assert!(content_type_is_yaml(&headers_with(Some(
            "application/yaml; charset=utf-8"
        ))));
        assert!(content_type_is_yaml(&headers_with(Some("application/x-yaml"))));
        assert!(!content_type_is_yaml(&headers_with(Some("application/json"))));
        assert!(!content_type_is_yaml(&headers_with(None)));
    }

    #[test]
    fn yaml_body_uses_raw_string_and_query_flag() {
        let q = CreateDeploymentObjectQuery {
            deletion_marker: Some(true),
        };
        let body = b"apiVersion: v1\nkind: Namespace\nmetadata:\n  name: foo\n";
        let (yaml, marker) =
            resolve_create_body(&headers_with(Some("application/yaml")), &q, body).unwrap();
        assert_eq!(yaml, String::from_utf8(body.to_vec()).unwrap());
        assert!(marker, "deletion_marker query flag should be honored on the YAML path");
    }

    #[test]
    fn yaml_body_defaults_marker_false() {
        let q = CreateDeploymentObjectQuery::default();
        let (_, marker) =
            resolve_create_body(&headers_with(Some("application/yaml")), &q, b"kind: ConfigMap")
                .unwrap();
        assert!(!marker);
    }

    #[test]
    fn json_body_still_parses() {
        let q = CreateDeploymentObjectQuery::default();
        let body = br#"{"yaml_content":"kind: ConfigMap","is_deletion_marker":true}"#;
        let (yaml, marker) =
            resolve_create_body(&headers_with(Some("application/json")), &q, body).unwrap();
        assert_eq!(yaml, "kind: ConfigMap");
        assert!(marker, "JSON body field wins on the JSON path");
    }

    #[test]
    fn json_path_query_flag_ignored() {
        // On the JSON path the body field is authoritative, not the query.
        let q = CreateDeploymentObjectQuery {
            deletion_marker: Some(true),
        };
        let body = br#"{"yaml_content":"kind: ConfigMap"}"#;
        let (_, marker) = resolve_create_body(&headers_with(None), &q, body).unwrap();
        assert!(!marker);
    }

    #[test]
    fn malformed_json_is_rejected() {
        let q = CreateDeploymentObjectQuery::default();
        let err =
            resolve_create_body(&headers_with(Some("application/json")), &q, b"{not json")
                .unwrap_err();
        assert_eq!(err.code, "invalid_deployment_object");
    }

    #[test]
    fn validate_accepts_multidoc_yaml() {
        let yaml = "apiVersion: v1\nkind: Namespace\nmetadata:\n  name: a\n---\napiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: b\n";
        assert!(validate_manifest_yaml(yaml, false).is_ok());
    }

    #[test]
    fn validate_rejects_malformed_yaml() {
        // Unbalanced flow mapping — not parseable.
        let err = validate_manifest_yaml("kind: : : [unbalanced", false).unwrap_err();
        assert_eq!(err.code, "invalid_deployment_object");
    }

    #[test]
    fn validate_rejects_empty_non_marker() {
        let err = validate_manifest_yaml("   \n", false).unwrap_err();
        assert_eq!(err.code, "invalid_deployment_object");
    }

    #[test]
    fn validate_allows_empty_marker() {
        assert!(validate_manifest_yaml("", true).is_ok());
        assert!(validate_manifest_yaml("   \n", true).is_ok());
    }

    #[test]
    fn validate_rejects_only_empty_documents() {
        // A body that parses but contains no real document.
        let err = validate_manifest_yaml("---\n---\n", false).unwrap_err();
        assert_eq!(err.code, "invalid_deployment_object");
    }
}
