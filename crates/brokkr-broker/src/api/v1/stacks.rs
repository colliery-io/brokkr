use axum::{
    Router,
    routing::{get, delete},
    extract::Path,
    Json,
};
use serde_json::Value;

pub fn routes() -> Router {
    Router::new()
        .route("/stacks", get(list_stacks).post(create_stack))
        .route("/stacks/:id", get(get_stack).put(update_stack).delete(delete_stack))
        .route("/stacks/:id/deployment-objects", get(list_deployment_objects).post(create_deployment_object))
        .route("/stacks/:id/labels", get(list_labels).post(add_label))
        .route("/stacks/:id/labels/:label", delete(remove_label))
        .route("/stacks/:id/annotations", get(list_annotations).post(add_annotation))
        .route("/stacks/:id/annotations/:key", delete(remove_annotation))
}

async fn list_stacks() -> Json<Value> {
    Json(serde_json::json!({"message": "List all stacks"}))
}

async fn create_stack() -> Json<Value> {
    Json(serde_json::json!({"message": "Create a new stack"}))
}

async fn get_stack(Path(id): Path<String>) -> Json<Value> {
    Json(serde_json::json!({"message": format!("Get stack details for ID: {}", id)}))
}

async fn update_stack(Path(id): Path<String>) -> Json<Value> {
    Json(serde_json::json!({"message": format!("Update stack with ID: {}", id)}))
}

async fn delete_stack(Path(id): Path<String>) -> Json<Value> {
    Json(serde_json::json!({"message": format!("Delete stack with ID: {}", id)}))
}

async fn list_deployment_objects(Path(id): Path<String>) -> Json<Value> {
    Json(serde_json::json!({"message": format!("List deployment objects for stack with ID: {}", id)}))
}

async fn create_deployment_object(Path(id): Path<String>) -> Json<Value> {
    Json(serde_json::json!({"message": format!("Create deployment object for stack with ID: {}", id)}))
}

async fn list_labels(Path(id): Path<String>) -> Json<Value> {
    Json(serde_json::json!({"message": format!("List labels for stack with ID: {}", id)}))
}

async fn add_label(Path(id): Path<String>) -> Json<Value> {
    Json(serde_json::json!({"message": format!("Add label to stack with ID: {}", id)}))
}

async fn remove_label(Path((id, label)): Path<(String, String)>) -> Json<Value> {
    Json(serde_json::json!({"message": format!("Remove label '{}' from stack with ID: {}", label, id)}))
}

async fn list_annotations(Path(id): Path<String>) -> Json<Value> {
    Json(serde_json::json!({"message": format!("List annotations for stack with ID: {}", id)}))
}

async fn add_annotation(Path(id): Path<String>) -> Json<Value> {
    Json(serde_json::json!({"message": format!("Add annotation to stack with ID: {}", id)}))
}

async fn remove_annotation(Path((id, key)): Path<(String, String)>) -> Json<Value> {
    Json(serde_json::json!({"message": format!("Remove annotation '{}' from stack with ID: {}", key, id)}))
}
