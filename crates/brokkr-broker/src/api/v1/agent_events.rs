use crate::dal::DAL;
use axum::{
    extract::{Extension, Path, State},
    routing::{get, post},
    Json, Router,
};
use brokkr_models::models::agent_events::{AgentEvent, NewAgentEvent};
use uuid::Uuid;

pub fn routes() -> Router<DAL> {
    Router::new()
        .route("/agent-events", get(list_agent_events))
        .route("/agent-events/:id", get(get_agent_event))
}

async fn list_agent_events(
    State(dal): State<DAL>,
    Extension(_auth_payload): Extension<crate::api::v1::middleware::AuthPayload>,
) -> Result<Json<Vec<AgentEvent>>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    match dal.agent_events().list() {
        Ok(events) => Ok(Json(events)),
        Err(e) => {
            eprintln!("Error fetching agent events: {:?}", e);
            Err((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch agent events"})),
            ))
        }
    }
}

async fn get_agent_event(
    State(dal): State<DAL>,
    Extension(_auth_payload): Extension<crate::api::v1::middleware::AuthPayload>,
    Path(id): Path<Uuid>,
) -> Result<Json<AgentEvent>, (axum::http::StatusCode, Json<serde_json::Value>)> {
    match dal.agent_events().get(id) {
        Ok(Some(event)) => Ok(Json(event)),
        Ok(None) => Err((
            axum::http::StatusCode::NOT_FOUND,
            Json(serde_json::json!({"error": "Agent event not found"})),
        )),
        Err(e) => {
            eprintln!("Error fetching agent event: {:?}", e);
            Err((
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                Json(serde_json::json!({"error": "Failed to fetch agent event"})),
            ))
        }
    }
}
