use axum::{
    extract::Path,
    routing::{delete, get, put, post},
    Json, Router,
};
use serde_json::Value;
use crate::dal::DAL;

pub fn routes() -> Router<DAL> {
    Router::new()
        .route("/generators", get(list_generators))
        .route("/generators", post(create_generator))
        .route("/generators/:id", get(get_generator))
        .route("/generators/:id", put(update_generator))
        .route("/generators/:id", delete(delete_generator))
}

async fn list_generators() -> Json<Value> {
    Json(serde_json::json!({"message": "List all generators"}))
}

async fn create_generator() -> Json<Value> {
    Json(serde_json::json!({"message": "Create a new generator"}))
}

async fn get_generator(Path(id): Path<String>) -> Json<Value> {
    Json(serde_json::json!({"message": format!("Get generator details for ID: {}", id)}))
}

async fn update_generator(Path(id): Path<String>) -> Json<Value> {
    Json(serde_json::json!({"message": format!("Update generator with ID: {}", id)}))
}

async fn delete_generator(Path(id): Path<String>) -> Json<Value> {
    Json(serde_json::json!({"message": format!("Delete generator with ID: {}", id)}))
}
