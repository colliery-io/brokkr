use crate::dal::DAL;
use axum::{extract::Path, routing::get, Json, Router};
use serde_json::Value;

pub fn routes() -> Router<DAL> {
    Router::new()
        .route("/deployment-objects", get(list_deployment_objects))
        .route("/deployment-objects/:id", get(get_deployment_object))
}

async fn list_deployment_objects() -> Json<Value> {
    Json(serde_json::json!({"message": "List all deployment objects"}))
}

async fn get_deployment_object(Path(id): Path<String>) -> Json<Value> {
    Json(serde_json::json!({"message": format!("Get deployment object details for ID: {}", id)}))
}
