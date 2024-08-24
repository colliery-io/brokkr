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

pub fn configure_routes() -> Router<AppState> {
    Router::new()
        .route("/stacks", get(list_stacks))
        .route("/stacks", post(create_stack))
        .route("/stacks/:id", get(get_stack))
        .route("/stacks/:id", put(update_stack))
        .route("/stacks/:id", delete(delete_stack))
}

async fn list_stacks(
    State(state): State<AppState>,
) -> Result<Json<Vec<Stack>>, (StatusCode, String)> {
    state.dal.stacks().get_active()
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

async fn create_stack(
    State(state): State<AppState>,
    Json(new_stack): Json<NewStack>,
) -> Result<Json<Stack>, (StatusCode, String)> {
    state.dal.stacks().create(&new_stack)
        .map(Json)
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()))
}

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