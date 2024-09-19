use axum::{
    Router,
    routing::get,
    extract::Path,
    Json,
};
use serde_json::Value;

pub fn routes() -> Router {
    Router::new()
        .route("/agent-events", get(list_agent_events))
        .route("/agent-events/:id", get(get_agent_event))
}

async fn list_agent_events() -> Json<Value> {
    Json(serde_json::json!({"message": "List all agent events"}))
}

async fn get_agent_event(Path(id): Path<String>) -> Json<Value> {
    Json(serde_json::json!({"message": format!("Get agent event details for ID: {}", id)}))
}
