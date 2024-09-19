mod agent_events;
mod agents;
mod auth;
mod deployment_objects;
mod stacks; // Add this line

use axum::Router;

pub fn routes() -> Router {
    Router::new()
        .merge(stacks::routes())
        .merge(agents::routes())
        .merge(deployment_objects::routes())
        .merge(agent_events::routes())
        .merge(auth::routes()) // Add this line
}
