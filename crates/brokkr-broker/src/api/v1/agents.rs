use axum::{
    extract::{Path, State, Extension},
    routing::{delete, get, post},
    Json, Router,
};
use serde_json::Value;
use axum::http::StatusCode;
use crate::dal::DAL;
use crate::api::v1::middleware::AuthPayload;
use brokkr_models::models::agents::Agent;

pub fn routes() -> Router<DAL> {
    Router::new()
        .route("/agents", get(list_agents).post(create_agent))
        .route(
            "/agents/:id",
            get(get_agent).put(update_agent).delete(delete_agent),
        )
        .route("/agents/:id/events", get(list_events).post(create_event))
        .route("/agents/:id/labels", get(list_labels).post(add_label))
        .route("/agents/:id/labels/:label", delete(remove_label))
        .route(
            "/agents/:id/annotations",
            get(list_annotations).post(add_annotation),
        )
        .route("/agents/:id/annotations/:key", delete(remove_annotation))
        .route("/agents/:id/targets", get(list_targets).post(add_target))
        .route("/agents/:id/targets/:stack_id", delete(remove_target))
        .route("/agents/:id/heartbeat", post(record_heartbeat))
        .route(
            "/agents/:id/applicable-deployment-objects",
            get(get_applicable_deployment_objects),
        )
}

async fn list_agents(
    State(dal): State<DAL>,
    Extension(auth_payload): Extension<AuthPayload>,
) -> Result<Json<Vec<Agent>>, (StatusCode, Json<serde_json::Value>)> {
    if !auth_payload.admin {
        return Err((
            StatusCode::FORBIDDEN,
            Json(serde_json::json!({"error": "Unauthorized"})),
        ));
    }

    match dal.agents().list() {
        Ok(agents) => Ok(Json(agents)),
        Err(e) => {
            eprintln!("Error fetching agents: {:?}", e);
            Err((
                StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch agents"})),
            ))
        }
    }
}

async fn create_agent() -> Json<Value> {
    Json(serde_json::json!({"message": "Create a new agent"}))
}

async fn get_agent(Path(id): Path<String>) -> Json<Value> {
    Json(serde_json::json!({"message": format!("Get agent details for ID: {}", id)}))
}

async fn update_agent(Path(id): Path<String>) -> Json<Value> {
    Json(serde_json::json!({"message": format!("Update agent with ID: {}", id)}))
}

async fn delete_agent(Path(id): Path<String>) -> Json<Value> {
    Json(serde_json::json!({"message": format!("Delete agent with ID: {}", id)}))
}

async fn list_events(Path(id): Path<String>) -> Json<Value> {
    Json(serde_json::json!({"message": format!("List events for agent with ID: {}", id)}))
}

async fn create_event(Path(id): Path<String>) -> Json<Value> {
    Json(serde_json::json!({"message": format!("Create event for agent with ID: {}", id)}))
}

async fn list_labels(Path(id): Path<String>) -> Json<Value> {
    Json(serde_json::json!({"message": format!("List labels for agent with ID: {}", id)}))
}

async fn add_label(Path(id): Path<String>) -> Json<Value> {
    Json(serde_json::json!({"message": format!("Add label to agent with ID: {}", id)}))
}

async fn remove_label(Path((id, label)): Path<(String, String)>) -> Json<Value> {
    Json(
        serde_json::json!({"message": format!("Remove label '{}' from agent with ID: {}", label, id)}),
    )
}

async fn list_annotations(Path(id): Path<String>) -> Json<Value> {
    Json(serde_json::json!({"message": format!("List annotations for agent with ID: {}", id)}))
}

async fn add_annotation(Path(id): Path<String>) -> Json<Value> {
    Json(serde_json::json!({"message": format!("Add annotation to agent with ID: {}", id)}))
}

async fn remove_annotation(Path((id, key)): Path<(String, String)>) -> Json<Value> {
    Json(
        serde_json::json!({"message": format!("Remove annotation '{}' from agent with ID: {}", key, id)}),
    )
}

async fn list_targets(Path(id): Path<String>) -> Json<Value> {
    Json(serde_json::json!({"message": format!("List targets for agent with ID: {}", id)}))
}

async fn add_target(Path(id): Path<String>) -> Json<Value> {
    Json(serde_json::json!({"message": format!("Add target to agent with ID: {}", id)}))
}

async fn remove_target(Path((id, stack_id)): Path<(String, String)>) -> Json<Value> {
    Json(
        serde_json::json!({"message": format!("Remove target '{}' from agent with ID: {}", stack_id, id)}),
    )
}

async fn record_heartbeat(Path(id): Path<String>) -> Json<Value> {
    Json(serde_json::json!({"message": format!("Record heartbeat for agent with ID: {}", id)}))
}

async fn get_applicable_deployment_objects(Path(id): Path<String>) -> Json<Value> {
    Json(
        serde_json::json!({"message": format!("Get applicable deployment objects for agent with ID: {}", id)}),
    )
}
