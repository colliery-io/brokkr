mod agents;
mod agent_events;
mod stacks;
mod deployment_objects;
mod auth;

use axum::{
    routing::Router,
    Extension,
};
use crate::dal::DAL;
use std::sync::Arc;

pub fn create_router(dal: DAL) -> Router {
    let dal = Arc::new(dal);
    Router::new()
        .merge(agents::router())
        // .merge(agent_events::router())
        // .merge(stacks::router())
        // .merge(deployment_objects::router())
        .layer(Extension(dal))
}