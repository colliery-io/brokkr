use crate::dal::DAL;

use axum::{    
    extract::{Extension, Path, State},
    http::StatusCode,
    routing::{get, post, delete},
    Json,
    Router,
};
use brokkr_models::models::stacks::{Stack, NewStack};
use brokkr_models::models::deployment_objects::{NewDeploymentObject, DeploymentObject};
use brokkr_models::models::stack_labels::{StackLabel, NewStackLabel};
use brokkr_models::models::stack_annotations::{StackAnnotation, NewStackAnnotation};
use crate::api::v1::middleware::AuthPayload;
use uuid::Uuid;
use brokkr_utils::logging::prelude::*;

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
        .route("/stacks/:id/labels", get(list_labels).post(add_label))
        .route("/stacks/:id/labels/:label", delete(remove_label))
        .route("/stacks/:id/annotations", get(list_annotations).post(add_annotation))
        .route("/stacks/:id/annotations/:key", delete(remove_annotation))
}

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
        Ok(objects) => Ok(Json(objects)),
        Err(_) => {
            
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch deployment objects"})),
            ))
        }
    }
}

async fn create_deployment_object(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
    Path(stack_id): Path<Uuid>,
    Json(new_object): Json<NewDeploymentObject>,
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

    // Ensure the stack_id in the path matches the one in the new object
    if new_object.stack_id != stack_id {
        return Err((
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({"error": "Stack ID mismatch"})),
        ));
    }

    // Create the deployment object
    match dal.deployment_objects().create(&new_object) {
        Ok(object) => Ok(Json(object)),
        Err(_) => {
            
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to create deployment object"})),
            ))
        }
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
        Err(_) => {
            
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch stack labels"})),
            ))
        }
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
            ))
        }
    };

    // Add the label
    match dal.stack_labels().create(&new_label) {
        Ok(new_label) => Ok(Json(new_label)),
        Err(_) => {
            
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to add stack label"})),
            ))
        }
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

    // Find the label to delete
    match dal.stack_labels().list_for_stack(stack_id) {
        Ok(labels) => {
            if let Some(stack_label) = labels.into_iter().find(|l| l.label == label) {
                match dal.stack_labels().delete(stack_label.id) {
                    Ok(_) => Ok(StatusCode::NO_CONTENT),
                    Err(_) => {
                        
                        Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(serde_json::json!({"error": "Failed to remove stack label"})),
                        ))
                    }
                }
            } else {
                Err((
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({"error": "Label not found"})),
                ))
            }
        }
        Err(_) => {
            
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch stack labels"})),
            ))
        }
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

        if agent_targets.iter().any(|target| target.stack_id == stack_id) {
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
        Err(_) => {
            
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch stack annotations"})),
            ))
        }
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
        Err(_) => {
            
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to add stack annotation"})),
            ))
        }
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

    // Find the annotation to delete
    match dal.stack_annotations().list_for_stack(stack_id) {
        Ok(annotations) => {
            if let Some(stack_annotation) = annotations.into_iter().find(|a| a.key == key) {
                match dal.stack_annotations().delete(stack_annotation.id) {
                    Ok(_) => Ok(StatusCode::NO_CONTENT),
                    Err(_) => {
                        
                        Err((
                            StatusCode::INTERNAL_SERVER_ERROR,
                            Json(serde_json::json!({"error": "Failed to remove stack annotation"})),
                        ))
                    }
                }
            } else {
                Err((
                    StatusCode::NOT_FOUND,
                    Json(serde_json::json!({"error": "Annotation not found"})),
                ))
            }
        }
        Err(_) => {
            
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch stack annotations"})),
            ))
        }
    }
}