
use axum::{
    routing::{get, delete},
    Router,
    extract::{Path, State},
    Json,
};
use serde::Deserialize;
use uuid::Uuid;
use crate::dal::DAL;
use brokkr_models::models::{
    stacks::{Stack, NewStack},
    deployment_objects::{DeploymentObject, NewDeploymentObject},
    stack_labels::{StackLabel, NewStackLabel},
    stack_annotations::{StackAnnotation, NewStackAnnotation},
};

#[allow(dead_code)]
#[derive(Deserialize)]
struct StackFilters {
    labels: Option<String>,
    annotations: Option<String>,
}

#[derive(Deserialize)]
struct UpdateStack {
    name: Option<String>,
    description: Option<String>,
}

async fn list_stacks(
    State(dal): State<DAL>,
) -> Result<Json<Vec<Stack>>, axum::http::StatusCode> {
    let stacks = dal.stacks().list()
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(stacks))
}

async fn create_stack(
    State(dal): State<DAL>,
    Json(new_stack): Json<NewStack>,
) -> Result<Json<Stack>, axum::http::StatusCode> {
    let stack = dal.stacks().create(&new_stack)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(stack))
}

async fn get_stack(
    State(dal): State<DAL>,
    Path(id): Path<Uuid>,
) -> Result<Json<Stack>, axum::http::StatusCode> {
    let stack = dal.stacks().get(id)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(axum::http::StatusCode::NOT_FOUND)?;
    Ok(Json(stack))
}

async fn update_stack(
    State(dal): State<DAL>,
    Path(id): Path<Uuid>,
    Json(update_stack): Json<UpdateStack>,
) -> Result<Json<Stack>, axum::http::StatusCode> {
    let mut stack = dal.stacks().get(id)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(axum::http::StatusCode::NOT_FOUND)?;
    
    if let Some(name) = update_stack.name {
        stack.name = name;
    }
    if let Some(description) = update_stack.description {
        stack.description = Some(description);
    }
    
    let updated_stack = dal.stacks().update(id, &stack)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(updated_stack))
}

async fn delete_stack(
    State(dal): State<DAL>,
    Path(id): Path<Uuid>,
) -> Result<Json<()>, axum::http::StatusCode> {
    dal.stacks().soft_delete(id)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(()))
}

async fn list_deployment_objects(
    State(dal): State<DAL>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<DeploymentObject>>, axum::http::StatusCode> {
    let deployment_objects = dal.deployment_objects().list_for_stack(id)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(deployment_objects))
}

async fn create_deployment_object(
    State(dal): State<DAL>,
    Json(new_deployment_object): Json<NewDeploymentObject>,
) -> Result<Json<DeploymentObject>, axum::http::StatusCode> {
    let deployment_object = dal.deployment_objects().create(&new_deployment_object)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(deployment_object))
}

async fn list_labels(
    State(dal): State<DAL>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<StackLabel>>, axum::http::StatusCode> {
    let labels = dal.stack_labels().list_for_stack(id)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(labels))
}

async fn add_label(
    State(dal): State<DAL>,
    Path(id): Path<Uuid>,
    Json(label): Json<String>,
) -> Result<Json<StackLabel>, axum::http::StatusCode> {
    let new_label = NewStackLabel::new(id, label)
        .map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
    
    let created_label = dal.stack_labels().create(&new_label)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(created_label))
}

async fn remove_label(
    State(dal): State<DAL>,
    Path((id, label)): Path<(Uuid, String)>,
) -> Result<Json<()>, axum::http::StatusCode> {
    let labels = dal.stack_labels().list_for_stack(id)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    
    if let Some(label_to_remove) = labels.iter().find(|l| l.label == label) {
        dal.stack_labels().delete(label_to_remove.id)
            .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(Json(()))
    } else {
        Err(axum::http::StatusCode::NOT_FOUND)
    }
}

async fn list_annotations(
    State(dal): State<DAL>,
    Path(id): Path<Uuid>,
) -> Result<Json<Vec<StackAnnotation>>, axum::http::StatusCode> {
    let annotations = dal.stack_annotations().list_for_stack(id)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(annotations))
}

async fn add_annotation(
    State(dal): State<DAL>,
    Path(id): Path<Uuid>,
    Json(annotation): Json<(String, String)>,
) -> Result<Json<StackAnnotation>, axum::http::StatusCode> {
    let (key, value) = annotation;
    let new_annotation = NewStackAnnotation::new(id, key, value)
        .map_err(|_| axum::http::StatusCode::BAD_REQUEST)?;
    
    let created_annotation = dal.stack_annotations().create(&new_annotation)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(created_annotation))
}

async fn remove_annotation(
    State(dal): State<DAL>,
    Path((id, key)): Path<(Uuid, String)>,
) -> Result<Json<()>, axum::http::StatusCode> {
    let annotations = dal.stack_annotations().list_for_stack(id)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    
    if let Some(annotation_to_remove) = annotations.iter().find(|a| a.key == key) {
        dal.stack_annotations().delete(annotation_to_remove.id)
            .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
        Ok(Json(()))
    } else {
        Err(axum::http::StatusCode::NOT_FOUND)
    }
}


/// Create the router for stack-related endpoints
pub fn configure_routes() -> Router<DAL> {
    Router::new()
        .route("/stacks", get(list_stacks).post(create_stack))
        .route("/stacks/:id", get(get_stack).put(update_stack).delete(delete_stack))
        .route("/stacks/:id/deployment-objects", get(list_deployment_objects).post(create_deployment_object))
        .route("/stacks/:id/labels", get(list_labels).post(add_label))
        .route("/stacks/:id/labels/:label", delete(remove_label))
        .route("/stacks/:id/annotations", get(list_annotations).post(add_annotation))
        .route("/stacks/:id/annotations/:key", delete(remove_annotation))
}

