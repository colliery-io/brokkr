//! # Stacks API Module
//!
//! This module provides the API endpoints for managing Stack entities using Axum.
//!
//! It includes routes for creating, retrieving, updating, and soft-deleting stacks,
//! as well as listing all active stacks. The module uses the Axum web framework
//! and interacts with a data access layer (DAL) to perform operations on Stack entities.

use axum::{
    extract::{Path, State},
    http::StatusCode,
    Json,
    routing::{get, post, put, delete},
    Router,
};
use brokkr_models::models::stacks::{Stack, NewStack};
use uuid::Uuid;
use crate::api::AppState;

/// Configures the stacks API routes.
///
/// This function sets up the following routes:
/// - GET /stacks: List all active stacks
/// - POST /stacks: Create a new stack
/// - GET /stacks/:id: Get a specific stack
/// - PUT /stacks/:id: Update a stack
/// - DELETE /stacks/:id: Soft delete a stack
///
/// # Returns
/// A configured `Router<AppState>` with all stack routes.
pub fn configure_routes() -> Router<AppState> {
    Router::new()
        .route("/stacks", get(list_stacks))
        .route("/stacks", post(create_stack))
        .route("/stacks/:id", get(get_stack))
        .route("/stacks/:id", put(update_stack))
        .route("/stacks/:id", delete(delete_stack))
}

/// Handler for listing all active stacks.
///
/// # Arguments
/// * `state` - The application state containing the DAL
///
/// # Returns
/// * On success: JSON array of active `Stack` objects
/// * On failure: `StatusCode::INTERNAL_SERVER_ERROR` with error message
async fn list_stacks(
    State(state): State<AppState>,
) -> Result<Json<Vec<Stack>>, (StatusCode, String)> {
    state.dal.stacks().get_active()
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

/// Handler for creating a new stack.
///
/// # Arguments
/// * `state` - The application state containing the DAL
/// * `new_stack` - JSON payload containing the new stack data
///
/// # Returns
/// * On success: JSON representation of the created `Stack`
/// * On failure: `StatusCode::INTERNAL_SERVER_ERROR` with error message
async fn create_stack(
    State(state): State<AppState>,
    Json(new_stack): Json<NewStack>,
) -> Result<Json<Stack>, (StatusCode, String)> {
    state.dal.stacks().create(&new_stack)
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

/// Handler for retrieving a stack by UUID.
///
/// # Arguments
/// * `state` - The application state containing the DAL
/// * `id` - The UUID of the stack to retrieve
///
/// # Returns
/// * On success: JSON representation of the `Stack`
/// * On not found: `StatusCode::NOT_FOUND` with error message
/// * On other errors: `StatusCode::INTERNAL_SERVER_ERROR` with error message
async fn get_stack(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<Stack>, (StatusCode, String)> {
    state.dal.stacks().get_by_id(id)
        .map(Json)
        .map_err(|e| match e {
            diesel::result::Error::NotFound => (StatusCode::NOT_FOUND, "Stack not found".to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        })
}

/// Handler for updating a stack.
///
/// # Arguments
/// * `state` - The application state containing the DAL
/// * `id` - The UUID of the stack to update
/// * `updated_stack` - JSON payload containing the updated stack data
///
/// # Returns
/// * On success: JSON representation of the updated `Stack`
/// * On not found: `StatusCode::NOT_FOUND` with error message
/// * On other errors: `StatusCode::INTERNAL_SERVER_ERROR` with error message
async fn update_stack(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
    Json(updated_stack): Json<Stack>,
) -> Result<Json<Stack>, (StatusCode, String)> {
    state.dal.stacks().update(id, &updated_stack)
        .map(Json)
        .map_err(|e| match e {
            diesel::result::Error::NotFound => (StatusCode::NOT_FOUND, "Stack not found".to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        })
}

/// Handler for soft-deleting a stack.
///
/// # Arguments
/// * `state` - The application state containing the DAL
/// * `id` - The UUID of the stack to soft delete
///
/// # Returns
/// * On success: `StatusCode::NO_CONTENT`
/// * On not found: `StatusCode::NOT_FOUND` with error message
/// * On other errors: `StatusCode::INTERNAL_SERVER_ERROR` with error message
async fn delete_stack(
    State(state): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, (StatusCode, String)> {
    state.dal.stacks().soft_delete(id)
        .map(|_| StatusCode::NO_CONTENT)
        .map_err(|e| match e {
            diesel::result::Error::NotFound => (StatusCode::NOT_FOUND, "Stack not found".to_string()),
            _ => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        })
}