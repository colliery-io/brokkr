/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

use crate::dal::DAL;
use crate::metrics;
use crate::utils::audit;
use crate::utils::matching::template_matches_stack;
use crate::utils::templating;
use brokkr_models::models::audit_logs::{
    ACTION_STACK_CREATED, ACTION_STACK_DELETED, ACTION_STACK_UPDATED,
    ACTOR_TYPE_ADMIN, RESOURCE_TYPE_STACK,
};

use crate::api::v1::middleware::AuthPayload;
use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    routing::{delete, get, post},
    Json, Router,
};
use brokkr_models::models::deployment_objects::{DeploymentObject, NewDeploymentObject};
use brokkr_models::models::rendered_deployment_objects::NewRenderedDeploymentObject;
use brokkr_models::models::stack_annotations::{NewStackAnnotation, StackAnnotation};
use brokkr_models::models::stack_labels::{NewStackLabel, StackLabel};
use brokkr_models::models::stacks::{NewStack, Stack};
use serde::{Deserialize, Serialize};
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
}

/// Lists all stacks.
///
/// # Authorization
/// Requires admin privileges.
#[utoipa::path(
    get,
    path = "/api/v1/stacks",
    tag = "stacks",
    responses(
        (status = 200, description = "List of stacks", body = Vec<Stack>),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - requires admin PAK"),
    ),
    security(
        ("pak" = [])
    )
)]
#[instrument(skip(dal, auth_payload), fields(admin = auth_payload.admin))]
async fn list_stacks(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
) -> Result<Json<Vec<Stack>>, (StatusCode, Json<serde_json::Value>)> {
    info!("Handling request to list stacks");
    if !auth_payload.admin {
        warn!("Unauthorized attempt to list stacks");
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Admin access required"})),
        ));
    }

    match dal.stacks().list() {
        Ok(stacks) => {
            info!("Successfully retrieved {} stacks", stacks.len());
            // Update stacks metric
            metrics::set_stacks_total(stacks.len() as i64);
            Ok(Json(stacks))
        }
        Err(e) => {
            error!("Failed to fetch stacks: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch stacks"})),
            ))
        }
    }
}

/// Creates a new stack.
///
/// # Authorization
/// Requires admin privileges.
#[utoipa::path(
    post,
    path = "/api/v1/stacks",
    tag = "stacks",
    request_body = NewStack,
    responses(
        (status = 201, description = "Stack created", body = Stack),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - requires admin PAK"),
    ),
    security(
        ("pak" = [])
    )
)]
async fn create_stack(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Json(new_stack): Json<NewStack>,
) -> Result<Json<Stack>, (StatusCode, Json<serde_json::Value>)> {
    info!("Handling request to create a new stack");
    if !auth_payload.admin && auth_payload.generator.is_none() {
        warn!("Unauthorized attempt to create a stack");
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Admin or generator access required"})),
        ));
    }

    if let Some(generator_id) = auth_payload.generator {
        if generator_id != new_stack.generator_id {
            warn!("Generator attempted to create stack for another generator");
            return Err((
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({"error": "Generator can only create stacks for itself"})),
            ));
        }
    }

    match dal.stacks().create(&new_stack) {
        Ok(stack) => {
            info!("Successfully created stack with ID: {}", stack.id);

            // Log audit entry for stack creation
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

            Ok(Json(stack))
        }
        Err(e) => {
            error!("Failed to create stack: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to create stack"})),
            ))
        }
    }
}

/// Gets a stack by ID.
///
/// # Authorization
/// Requires admin privileges.
#[utoipa::path(
    get,
    path = "/api/v1/stacks/{id}",
    tag = "stacks",
    params(
        ("id" = Uuid, Path, description = "Stack ID")
    ),
    responses(
        (status = 200, description = "Stack found", body = Stack),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - requires admin PAK"),
        (status = 404, description = "Stack not found"),
    ),
    security(
        ("pak" = [])
    )
)]
async fn get_stack(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
) -> Result<Json<Stack>, (StatusCode, Json<serde_json::Value>)> {
    info!("Handling request to get stack with ID: {}", id);
    let stack = dal.stacks().get(vec![id]).map_err(|e| {
        error!("Failed to fetch stack with ID {}: {:?}", id, e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to fetch stack"})),
        )
    })?;

    if stack.is_empty() {
        warn!("Stack not found with ID: {}", id);
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Stack not found"})),
        ));
    }

    let stack = &stack[0];

    if !auth_payload.admin && auth_payload.generator != Some(stack.generator_id) {
        warn!("Unauthorized attempt to access stack with ID: {}", id);
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Access denied"})),
        ));
    }

    info!("Successfully retrieved stack with ID: {}", id);
    Ok(Json(stack.clone()))
}

/// Updates a stack.
///
/// # Authorization
/// Requires admin privileges.
#[utoipa::path(
    put,
    path = "/api/v1/stacks/{id}",
    tag = "stacks",
    params(
        ("id" = Uuid, Path, description = "Stack ID")
    ),
    request_body = Stack,
    responses(
        (status = 200, description = "Stack updated", body = Stack),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - requires admin PAK"),
        (status = 404, description = "Stack not found"),
    ),
    security(
        ("pak" = [])
    )
)]
async fn update_stack(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
    Json(updated_stack): Json<Stack>,
) -> Result<Json<Stack>, (StatusCode, Json<serde_json::Value>)> {
    info!("Handling request to update stack with ID: {}", id);
    let existing_stack = dal.stacks().get(vec![id]).map_err(|e| {
        error!("Failed to fetch stack with ID {}: {:?}", id, e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to fetch stack"})),
        )
    })?;

    if existing_stack.is_empty() {
        warn!("Stack not found with ID: {}", id);
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Stack not found"})),
        ));
    }

    let existing_stack = &existing_stack[0];

    if !auth_payload.admin && auth_payload.generator != Some(existing_stack.generator_id) {
        warn!("Unauthorized attempt to update stack with ID: {}", id);
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Access denied"})),
        ));
    }

    if id != updated_stack.id {
        warn!("Stack ID mismatch during update for ID: {}", id);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Stack ID mismatch"})),
        ));
    }

    match dal.stacks().update(id, &updated_stack) {
        Ok(stack) => {
            info!("Successfully updated stack with ID: {}", id);

            // Log audit entry for stack update
            audit::log_action(
                ACTOR_TYPE_ADMIN,
                None,
                ACTION_STACK_UPDATED,
                RESOURCE_TYPE_STACK,
                Some(id),
                Some(serde_json::json!({
                    "name": stack.name,
                })),
                None,
                None,
            );

            Ok(Json(stack))
        }
        Err(e) => {
            error!("Failed to update stack with ID {}: {:?}", id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to update stack"})),
            ))
        }
    }
}

/// Deletes a stack.
///
/// # Authorization
/// Requires admin privileges.
#[utoipa::path(
    delete,
    path = "/api/v1/stacks/{id}",
    tag = "stacks",
    params(
        ("id" = Uuid, Path, description = "Stack ID")
    ),
    responses(
        (status = 204, description = "Stack deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden - requires admin PAK"),
        (status = 404, description = "Stack not found"),
    ),
    security(
        ("pak" = [])
    )
)]
async fn delete_stack(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    info!("Handling request to delete stack with ID: {}", id);
    let existing_stack = dal.stacks().get(vec![id]).map_err(|e| {
        error!("Failed to fetch stack with ID {}: {:?}", id, e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to fetch stack"})),
        )
    })?;

    if existing_stack.is_empty() {
        warn!("Stack not found with ID: {}", id);
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Stack not found"})),
        ));
    }

    let existing_stack = &existing_stack[0];

    if !auth_payload.admin && auth_payload.generator != Some(existing_stack.generator_id) {
        warn!("Unauthorized attempt to delete stack with ID: {}", id);
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Access denied"})),
        ));
    }

    match dal.stacks().soft_delete(id) {
        Ok(_) => {
            info!("Successfully deleted stack with ID: {}", id);

            // Log audit entry for stack deletion
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
        Err(e) => {
            error!("Failed to delete stack with ID {}: {:?}", id, e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to delete stack"})),
            ))
        }
    }
}

async fn list_deployment_objects(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(stack_id): Path<Uuid>,
) -> Result<Json<Vec<DeploymentObject>>, (StatusCode, Json<serde_json::Value>)> {
    // Check if the user is an admin or the associated generator
    if !auth_payload.admin {
        let stack = dal.stacks().get(vec![stack_id]).map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch stack"})),
            )
        })?;

        if stack.is_empty() {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Stack not found"})),
            ));
        }

        let stack = &stack[0];
        if auth_payload.generator != Some(stack.generator_id) {
            return Err((
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({"error": "Access denied"})),
            ));
        }
    }

    // Fetch deployment objects
    match dal.deployment_objects().list_for_stack(stack_id) {
        Ok(objects) => {
            // Update deployment objects metric
            metrics::set_deployment_objects_total(objects.len() as i64);
            Ok(Json(objects))
        }
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to fetch deployment objects"})),
        )),
    }
}

async fn create_deployment_object(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(stack_id): Path<Uuid>,
    Json(payload): Json<serde_json::Value>,
) -> Result<Json<DeploymentObject>, (StatusCode, Json<serde_json::Value>)> {
    // Check if the user is an admin or the associated generator
    if !auth_payload.admin {
        let stack = dal.stacks().get(vec![stack_id]).map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch stack"})),
            )
        })?;

        if stack.is_empty() {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Stack not found"})),
            ));
        }

        let stack = &stack[0];
        if auth_payload.generator != Some(stack.generator_id) {
            return Err((
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({"error": "Access denied"})),
            ));
        }
    }

    // Extract required fields from payload
    let yaml_content = payload["yaml_content"]
        .as_str()
        .ok_or((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Missing or invalid yaml_content"})),
        ))?
        .to_string();

    let is_deletion_marker = payload["is_deletion_marker"].as_bool().unwrap_or(false);

    // Create new deployment object with proper hash calculation
    let new_object =
        NewDeploymentObject::new(stack_id, yaml_content, is_deletion_marker).map_err(|e| {
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": e})),
            )
        })?;

    // Create the deployment object
    match dal.deployment_objects().create(&new_object) {
        Ok(object) => Ok(Json(object)),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to create deployment object"})),
        )),
    }
}

async fn list_labels(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(stack_id): Path<Uuid>,
) -> Result<Json<Vec<StackLabel>>, (StatusCode, Json<serde_json::Value>)> {
    // Check authorization
    if !is_authorized_for_stack(&dal, &auth_payload, stack_id).await? {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Access denied"})),
        ));
    }

    // Fetch labels
    match dal.stack_labels().list_for_stack(stack_id) {
        Ok(labels) => Ok(Json(labels)),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to fetch stack labels"})),
        )),
    }
}

async fn add_label(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(stack_id): Path<Uuid>,
    Json(label): Json<String>,
) -> Result<Json<StackLabel>, (StatusCode, Json<serde_json::Value>)> {
    // Check if the user is an admin or the associated generator
    if !auth_payload.admin {
        let stack = dal.stacks().get(vec![stack_id]).map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch stack"})),
            )
        })?;

        if stack.is_empty() {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Stack not found"})),
            ));
        }

        let stack = &stack[0];
        if auth_payload.generator != Some(stack.generator_id) {
            return Err((
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({"error": "Access denied"})),
            ));
        }
    }

    // Create NewStackLabel
    let new_label = match NewStackLabel::new(stack_id, label) {
        Ok(label) => label,
        Err(e) => {
            return Err((
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": e})),
            ));
        }
    };

    // Add the label
    match dal.stack_labels().create(&new_label) {
        Ok(new_label) => Ok(Json(new_label)),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to add stack label"})),
        )),
    }
}

async fn remove_label(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path((stack_id, label)): Path<(Uuid, String)>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    // Check authorization
    if !auth_payload.admin {
        let stack = dal.stacks().get(vec![stack_id]).map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch stack"})),
            )
        })?;

        if stack.is_empty() {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Stack not found"})),
            ));
        }

        let stack = &stack[0];
        if auth_payload.generator != Some(stack.generator_id) {
            return Err((
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({"error": "Access denied"})),
            ));
        }
    }

    // Delete the label directly using indexed query
    match dal.stack_labels().delete_by_stack_and_label(stack_id, &label) {
        Ok(deleted_count) => {
            if deleted_count > 0 {
                Ok(StatusCode::NO_CONTENT)
            } else {
                Err((
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({"error": "Label not found"})),
                ))
            }
        }
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to remove stack label"})),
        )),
    }
}

async fn is_authorized_for_stack(
    dal: &DAL,
    auth_payload: &AuthPayload,
    stack_id: Uuid,
) -> Result<bool, (StatusCode, Json<serde_json::Value>)> {
    if auth_payload.admin {
        return Ok(true);
    }

    let stack = dal.stacks().get(vec![stack_id]).map_err(|_| {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to fetch stack"})),
        )
    })?;

    if stack.is_empty() {
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Stack not found"})),
        ));
    }

    let stack = &stack[0];

    if auth_payload.generator == Some(stack.generator_id) {
        return Ok(true);
    }

    if let Some(agent_id) = auth_payload.agent {
        let agent_targets = dal.agent_targets().list_for_agent(agent_id).map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch agent targets"})),
            )
        })?;

        if agent_targets
            .iter()
            .any(|target| target.stack_id == stack_id)
        {
            return Ok(true);
        }
    }

    Ok(false)
}

async fn list_annotations(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(stack_id): Path<Uuid>,
) -> Result<Json<Vec<StackAnnotation>>, (StatusCode, Json<serde_json::Value>)> {
    // Check authorization
    if !is_authorized_for_stack(&dal, &auth_payload, stack_id).await? {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Access denied"})),
        ));
    }

    // Fetch annotations
    match dal.stack_annotations().list_for_stack(stack_id) {
        Ok(annotations) => Ok(Json(annotations)),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to fetch stack annotations"})),
        )),
    }
}

async fn add_annotation(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(stack_id): Path<Uuid>,
    Json(new_annotation): Json<NewStackAnnotation>,
) -> Result<Json<StackAnnotation>, (StatusCode, Json<serde_json::Value>)> {
    // Check if the user is an admin or the associated generator
    if !auth_payload.admin {
        let stack = dal.stacks().get(vec![stack_id]).map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch stack"})),
            )
        })?;

        if stack.is_empty() {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Stack not found"})),
            ));
        }

        let stack = &stack[0];
        if auth_payload.generator != Some(stack.generator_id) {
            return Err((
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({"error": "Access denied"})),
            ));
        }
    }

    // Ensure the stack_id in the path matches the one in the new annotation
    if new_annotation.stack_id != stack_id {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Stack ID mismatch"})),
        ));
    }

    // Add the annotation
    match dal.stack_annotations().create(&new_annotation) {
        Ok(new_annotation) => Ok(Json(new_annotation)),
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to add stack annotation"})),
        )),
    }
}

async fn remove_annotation(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path((stack_id, key)): Path<(Uuid, String)>,
) -> Result<StatusCode, (StatusCode, Json<serde_json::Value>)> {
    // Check authorization
    if !auth_payload.admin {
        let stack = dal.stacks().get(vec![stack_id]).map_err(|_| {
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch stack"})),
            )
        })?;

        if stack.is_empty() {
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Stack not found"})),
            ));
        }

        let stack = &stack[0];
        if auth_payload.generator != Some(stack.generator_id) {
            return Err((
                StatusCode::FORBIDDEN,
                Json(serde_json::json!({"error": "Access denied"})),
            ));
        }
    }

    // Delete the annotation directly using indexed query
    match dal.stack_annotations().delete_by_stack_and_key(stack_id, &key) {
        Ok(deleted_count) => {
            if deleted_count > 0 {
                Ok(StatusCode::NO_CONTENT)
            } else {
                Err((
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({"error": "Annotation not found"})),
                ))
            }
        }
        Err(_) => Err((
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to remove stack annotation"})),
        )),
    }
}

/// Request body for template instantiation.
#[derive(Debug, Deserialize, Serialize, ToSchema)]
pub struct TemplateInstantiationRequest {
    /// ID of the template to instantiate.
    pub template_id: Uuid,
    /// Parameters to render the template with.
    pub parameters: serde_json::Value,
}

/// Instantiates a template into a deployment object.
///
/// This endpoint renders a template with the provided parameters and creates
/// a deployment object in the specified stack.
///
/// # Authorization
/// Admin or generator with stack access.
#[utoipa::path(
    post,
    path = "/api/v1/stacks/{stack_id}/deployment-objects/from-template",
    tag = "stacks",
    params(
        ("stack_id" = Uuid, Path, description = "Stack ID")
    ),
    request_body = TemplateInstantiationRequest,
    responses(
        (status = 201, description = "Deployment object created", body = DeploymentObject),
        (status = 400, description = "Invalid parameters or template rendering failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Template or stack not found"),
        (status = 422, description = "Template labels don't match stack"),
    ),
    security(
        ("pak" = [])
    )
)]
async fn instantiate_template(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(stack_id): Path<Uuid>,
    Json(request): Json<TemplateInstantiationRequest>,
) -> Result<(StatusCode, Json<DeploymentObject>), (StatusCode, Json<serde_json::Value>)> {
    info!(
        "Handling template instantiation: template={}, stack={}",
        request.template_id, stack_id
    );

    // 1. Get stack (404 if not found)
    let stack = dal.stacks().get(vec![stack_id]).map_err(|e| {
        error!("Failed to fetch stack {}: {:?}", stack_id, e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to fetch stack"})),
        )
    })?;

    if stack.is_empty() {
        warn!("Stack not found: {}", stack_id);
        return Err((
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Stack not found"})),
        ));
    }
    let stack = &stack[0];

    // 2. Verify authorization (admin or generator with stack access)
    if !auth_payload.admin && auth_payload.generator != Some(stack.generator_id) {
        warn!(
            "Unauthorized template instantiation attempt for stack {}",
            stack_id
        );
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Access denied"})),
        ));
    }

    // 3. Get template (404 if not found/deleted)
    let template = dal.templates().get(request.template_id).map_err(|e| {
        error!("Failed to fetch template {}: {:?}", request.template_id, e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to fetch template"})),
        )
    })?;

    let template = match template {
        Some(t) => t,
        None => {
            warn!("Template not found: {}", request.template_id);
            return Err((
                StatusCode::NOT_FOUND,
                Json(serde_json::json!({"error": "Template not found"})),
            ));
        }
    };

    // 4. Get template labels/annotations
    let template_labels: Vec<String> = dal
        .template_labels()
        .list_for_template(template.id)
        .map_err(|e| {
            error!("Failed to fetch template labels: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch template labels"})),
            )
        })?
        .into_iter()
        .map(|l| l.label)
        .collect();

    let template_annotations: Vec<(String, String)> = dal
        .template_annotations()
        .list_for_template(template.id)
        .map_err(|e| {
            error!("Failed to fetch template annotations: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch template annotations"})),
            )
        })?
        .into_iter()
        .map(|a| (a.key, a.value))
        .collect();

    // 5. Get stack labels/annotations
    let stack_labels: Vec<String> = dal
        .stack_labels()
        .list_for_stack(stack_id)
        .map_err(|e| {
            error!("Failed to fetch stack labels: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch stack labels"})),
            )
        })?
        .into_iter()
        .map(|l| l.label)
        .collect();

    let stack_annotations: Vec<(String, String)> = dal
        .stack_annotations()
        .list_for_stack(stack_id)
        .map_err(|e| {
            error!("Failed to fetch stack annotations: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch stack annotations"})),
            )
        })?
        .into_iter()
        .map(|a| (a.key, a.value))
        .collect();

    // 6. Validate label matching (422 with details on mismatch)
    let match_result = template_matches_stack(
        &template_labels,
        &template_annotations,
        &stack_labels,
        &stack_annotations,
    );

    if !match_result.matches {
        warn!(
            "Template {} labels don't match stack {}: missing_labels={:?}, missing_annotations={:?}",
            template.id, stack_id, match_result.missing_labels, match_result.missing_annotations
        );
        return Err((
            StatusCode::UNPROCESSABLE_ENTITY,
            Json(serde_json::json!({
                "error": "Template labels do not match stack",
                "missing_labels": match_result.missing_labels,
                "missing_annotations": match_result.missing_annotations,
            })),
        ));
    }

    // 7. Validate parameters against JSON Schema (400 on invalid)
    if let Err(errors) = templating::validate_parameters(&template.parameters_schema, &request.parameters) {
        let error_messages: Vec<String> = errors.iter().map(|e| e.to_string()).collect();
        warn!("Parameter validation failed: {:?}", error_messages);
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({
                "error": "Invalid parameters",
                "validation_errors": error_messages,
            })),
        ));
    }

    // 8. Render template with Tera (400 on render error)
    let rendered_yaml = templating::render_template(&template.template_content, &request.parameters)
        .map_err(|e| {
            error!("Failed to render template: {:?}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": e.to_string()})),
            )
        })?;

    // 9. Create DeploymentObject
    let new_deployment_object =
        NewDeploymentObject::new(stack_id, rendered_yaml.clone(), false).map_err(|e| {
            error!("Failed to create deployment object: {:?}", e);
            (
                StatusCode::BAD_REQUEST,
                Json(serde_json::json!({"error": e})),
            )
        })?;

    let deployment_object = dal
        .deployment_objects()
        .create(&new_deployment_object)
        .map_err(|e| {
            error!("Failed to insert deployment object: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to create deployment object"})),
            )
        })?;

    // 10. Create RenderedDeploymentObject provenance record
    let parameters_json = serde_json::to_string(&request.parameters).map_err(|e| {
        error!("Failed to serialize parameters: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": "Failed to serialize parameters"})),
        )
    })?;

    let provenance = NewRenderedDeploymentObject::new(
        deployment_object.id,
        template.id,
        template.version,
        parameters_json,
    )
    .map_err(|e| {
        error!("Failed to create provenance record: {:?}", e);
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({"error": e})),
        )
    })?;

    dal.rendered_deployment_objects()
        .create(&provenance)
        .map_err(|e| {
            error!("Failed to insert provenance record: {:?}", e);
            (
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to create provenance record"})),
            )
        })?;

    info!(
        "Successfully instantiated template {} into deployment object {} for stack {}",
        template.id, deployment_object.id, stack_id
    );

    Ok((StatusCode::CREATED, Json(deployment_object)))
}
