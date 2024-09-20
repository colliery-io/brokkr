mod agent_events;
mod agents;
mod auth;
mod deployment_objects;
mod generators;
mod stacks; // Add this line

use crate::dal::DAL;
use axum::middleware::from_fn_with_state;
use axum::Router;
mod middleware;

pub fn routes(dal: DAL) -> Router<DAL> {
    Router::new()
        .merge(agents::routes())
        .merge(stacks::routes())
        .merge(deployment_objects::routes())
        .merge(agent_events::routes())
        .merge(generators::routes())
        .merge(auth::routes())
        .layer(from_fn_with_state(
            dal.clone(),
            middleware::auth_middleware::<axum::body::Body>,
        ))
}
