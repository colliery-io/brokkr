use crate::dal::DAL;
use axum::{
    extract::{Path, State},
    routing::get,
    Json, Router,
};
use brokkr_models::models::deployment_objects::DeploymentObject;
use uuid::Uuid;

// /// List all deployment objects
// async fn list_all_deployment_objects(
//     State(dal): State<DAL>,
// ) -> Result<Json<Vec<DeploymentObject>>, axum::http::StatusCode> {
//     let objects = dal.deployment_objects().list()
//         .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
//     Ok(Json(objects))
// }

/// Get a specific deployment object by ID
async fn get_deployment_object(
    State(dal): State<DAL>,
    Path(id): Path<Uuid>,
) -> Result<Json<DeploymentObject>, axum::http::StatusCode> {
    let deployment_object = dal
        .deployment_objects()
        .get(id)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(axum::http::StatusCode::NOT_FOUND)?;
    Ok(Json(deployment_object))
}

/// Soft delete a deployment object
async fn delete_deployment_object(
    State(dal): State<DAL>,
    Path(id): Path<Uuid>,
) -> Result<Json<()>, axum::http::StatusCode> {
    dal.deployment_objects()
        .soft_delete(id)
        .map_err(|_| axum::http::StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(()))
}

/// Configure routes for deployment objects
pub fn configure_routes() -> Router<DAL> {
    Router::new().route(
        "/deployment-objects/:id",
        get(get_deployment_object).delete(delete_deployment_object),
    )
}
