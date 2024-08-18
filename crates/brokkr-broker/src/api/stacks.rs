use axum::{
    extract::{Path, Extension},
    routing::{get, post, put, delete},
    Json, Router,
};
use crate::dal::DAL;
use crate::api::auth::Claims;
use uuid::Uuid;
use brokkr_models::models::stacks::{Stack, NewStack};
use std::sync::Arc;

pub fn router() -> Router {
    Router::new()
        .route("/stacks", get(list_stacks).post(create_stack))
        .route("/stacks/:id", get(get_stack).put(update_stack).delete(delete_stack))
        .route("/stacks/active", get(list_active_stacks))
}

async fn list_stacks(
    Extension(dal): Extension<Arc<DAL>>,
    _claims: Claims,
) -> Json<Vec<Stack>> {
    let stacks = dal.stacks().get_all().unwrap();
    Json(stacks)
}

async fn create_stack(
    Extension(dal): Extension<Arc<DAL>>,
    _claims: Claims,
    Json(new_stack): Json<NewStack>,
) -> Json<Stack> {
    let stack = dal.stacks().create(&new_stack).unwrap();
    Json(stack)
}

async fn get_stack(
    Extension(dal): Extension<Arc<DAL>>,
    _claims: Claims,
    Path(id): Path<Uuid>,
) -> Json<Stack> {
    let stack = dal.stacks().get_by_id(id).unwrap();
    Json(stack)
}

async fn update_stack(
    Extension(dal): Extension<Arc<DAL>>,
    _claims: Claims,
    Path(id): Path<Uuid>,
    Json(stack): Json<Stack>,
) -> Json<Stack> {
    let updated_stack = dal.stacks().update(id, &stack).unwrap();
    Json(updated_stack)
}

async fn delete_stack(
    Extension(dal): Extension<Arc<DAL>>,
    _claims: Claims,
    Path(id): Path<Uuid>,
) -> Json<Stack> {
    let deleted_stack = dal.stacks().soft_delete(id).unwrap();
    Json(deleted_stack)
}

async fn list_active_stacks(
    Extension(dal): Extension<Arc<DAL>>,
    _claims: Claims,
) -> Json<Vec<Stack>> {
    let active_stacks = dal.stacks().get_active().unwrap();
    Json(active_stacks)
}