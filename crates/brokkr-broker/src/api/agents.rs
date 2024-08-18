use axum::{
    extract::{Path, Extension},
    routing::{get, post, put, delete},
    Json, Router, http::StatusCode,
};
use crate::dal::DAL;
use crate::api::auth::Claims;
use uuid::Uuid;
use brokkr_models::models::agents::{Agent, NewAgent};
use std::sync::Arc;

pub fn router() -> Router {
    Router::new()
        .route("/agents", get(list_agents).post(create_agent))
        .route("/agents/:id", get(get_agent).put(update_agent).delete(delete_agent))
        .route("/agents/:id/heartbeat", post(update_agent_heartbeat))
        .route("/agents/:id/status", put(update_agent_status))
}


async fn list_agents(
    Extension(dal): Extension<Arc<DAL>>,
    _claims: Claims,
) -> Json<Vec<Agent>> {
    let agents = dal.agents().list().unwrap();
    Json(agents)
}

async fn create_agent(
    Extension(dal): Extension<Arc<DAL>>,
    _claims: Claims,
    Json(new_agent): Json<NewAgent>,
) -> Json<Agent> {
    let agent = dal.agents().create(&new_agent).unwrap();
    Json(agent)
}

async fn get_agent(
    Extension(dal): Extension<Arc<DAL>>,
    _claims: Claims,
    Path(id): Path<Uuid>,
) -> (StatusCode, Json<Option<Agent>>) {
    match dal.agents().get(id) {
        Ok(agent) => (StatusCode::OK, Json(agent)),
        Err(_) => (StatusCode::INTERNAL_SERVER_ERROR, Json(None)),
    }
}

async fn update_agent(
    Extension(dal): Extension<Arc<DAL>>,
    _claims: Claims,
    Path(id): Path<Uuid>,
    Json(agent): Json<Agent>,
) -> (StatusCode, Json<Option<Agent>>) {
    match dal.agents().update(id, &agent) {
        Ok(updated_agent) => (StatusCode::OK, Json(Some(updated_agent))),
        Err(_) => (StatusCode::UNPROCESSABLE_ENTITY, Json(None)),
    }
}

async fn delete_agent(
    Extension(dal): Extension<Arc<DAL>>,
    _claims: Claims,
    Path(id): Path<Uuid>,
) -> StatusCode {
    match dal.agents().soft_delete(id) {
        Ok(_) => StatusCode::OK,
        Err(_) => StatusCode::OK, // Still return OK even if the agent doesn't exist
    }
}

async fn update_agent_heartbeat(
    Extension(dal): Extension<Arc<DAL>>,
    _claims: Claims,
    Path(id): Path<Uuid>,
) -> Json<Agent> {
    let updated_agent = dal.agents().update_heartbeat(id).unwrap();
    Json(updated_agent)
}


async fn update_agent_status(
    Extension(dal): Extension<Arc<DAL>>,
    _claims: Claims,
    Path(id): Path<Uuid>,
    Json(status): Json<String>,
) -> Json<Agent> {
    let updated_agent = dal.agents().update_status(id, &status).unwrap();
    Json(updated_agent)
}