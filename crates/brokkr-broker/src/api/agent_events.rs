use axum::{
    extract::{Path, Extension},
    routing::{get, post, delete},
    Json, Router,
};
use crate::dal::DAL;
use crate::api::auth::Claims;
use uuid::Uuid;
use brokkr_models::models::agent_events::{AgentEvent, NewAgentEvent};
use std::sync::Arc;

pub fn router() -> Router {
    Router::new()
        .route("/agent_events", get(list_agent_events).post(create_agent_event))
        .route("/agent_events/:id", get(get_agent_event).delete(delete_agent_event))
        .route("/agent_events", get(list_agent_events).post(create_agent_event))
        .route("/agent_events/:id", get(get_agent_event).delete(delete_agent_event))
}

async fn list_agent_events(
    Extension(dal): Extension<Arc<DAL>>,
    _claims: Claims,
) -> Json<Vec<AgentEvent>> {
    let events = dal.agent_events().list().unwrap();
    Json(events)
}

async fn create_agent_event(
    Extension(dal): Extension<Arc<DAL>>,
    _claims: Claims,
    Json(new_event): Json<NewAgentEvent>,
) -> Json<AgentEvent> {
    let event = dal.agent_events().create(&new_event).unwrap();
    Json(event)
}

async fn get_agent_event(
    Extension(dal): Extension<Arc<DAL>>,
    _claims: Claims,
    Path(id): Path<Uuid>,
) -> Json<Option<AgentEvent>> {
    let event = dal.agent_events().get(id).unwrap();
    Json(event)
}

async fn delete_agent_event(
    Extension(dal): Extension<Arc<DAL>>,
    _claims: Claims,
    Path(id): Path<Uuid>,
) -> Json<()> {
    dal.agent_events().soft_delete(id).unwrap();
    Json(())
}