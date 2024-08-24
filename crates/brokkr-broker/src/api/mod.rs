//! This module aggregates all API routes and provides a function to configure the main router.

use axum::Router;
use crate::dal::DAL;

// Import submodules
pub mod agents;
pub mod stacks;
pub mod deployment_objects;
// pub mod agent_events;

/// Shared state for the application
#[derive(Clone)]
pub struct AppState {
    dal: DAL,
}

/// Configures and returns the main application router with all API routes
pub fn configure_api_routes(dal: DAL) -> Router {
    let app_state = AppState { dal };

    Router::new()
        .merge(agents::configure_routes())
        .merge(stacks::configure_routes())
        .merge(deployment_objects::configure_routes())
        // .merge(agent_events::configure_routes())
        .with_state(app_state)
}