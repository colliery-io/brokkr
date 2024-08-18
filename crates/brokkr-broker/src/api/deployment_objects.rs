use axum::{
    extract::{Path, Extension},
    routing::{get, post, put, delete},
    Json, Router,
};
use crate::dal::DAL;
use crate::api::auth::Claims;
use uuid::Uuid;
use brokkr_models::models::deployment_objects::{DeploymentObject, NewDeploymentObject};
use std::sync::Arc;

pub fn router() -> Router {
    Router::new()
        .route("/deployment_objects", get(list_deployment_objects).post(create_deployment_object))
        .route("/deployment_objects/:id", get(get_deployment_object).put(update_deployment_object).delete(delete_deployment_object))
        .route("/deployment_objects/stack/:stack_id", get(get_deployment_objects_by_stack))
}

async fn list_deployment_objects(
    Extension(dal): Extension<Arc<DAL>>,
    _claims: Claims,
) -> Json<Vec<DeploymentObject>> {
    let objects = dal.deployment_objects().get_active().unwrap();
    Json(objects)
}

async fn create_deployment_object(
    Extension(dal): Extension<Arc<DAL>>,
    _claims: Claims,
    Json(new_object): Json<NewDeploymentObject>,
) -> Json<DeploymentObject> {
    let object = dal.deployment_objects().create(&new_object).unwrap();
    Json(object)
}

async fn get_deployment_object(
    Extension(dal): Extension<Arc<DAL>>,
    _claims: Claims,
    Path(id): Path<Uuid>,
) -> Json<DeploymentObject> {
    let object = dal.deployment_objects().get_by_id(id).unwrap();
    Json(object)
}

async fn update_deployment_object(
    Extension(dal): Extension<Arc<DAL>>,
    _claims: Claims,
    Path(id): Path<Uuid>,
    Json(object): Json<DeploymentObject>,
) -> Json<DeploymentObject> {
    let updated_object = dal.deployment_objects().update(id, &object).unwrap();
    Json(updated_object)
}

async fn delete_deployment_object(
    Extension(dal): Extension<Arc<DAL>>,
    _claims: Claims,
    Path(id): Path<Uuid>,
) -> Json<DeploymentObject> {
    let deleted_object = dal.deployment_objects().soft_delete(id).unwrap();
    Json(deleted_object)
}

async fn get_deployment_objects_by_stack(
    Extension(dal): Extension<Arc<DAL>>,
    _claims: Claims,
    Path(stack_id): Path<Uuid>,
) -> Json<Vec<DeploymentObject>> {
    let objects = dal.deployment_objects().get_by_stack_id(stack_id).unwrap();
    Json(objects)
}