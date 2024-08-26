
//! # API Routes Aggregator Module
//!
//! This module aggregates all API routes and provides a function to configure the main router.
//! It serves as the central point for organizing and initializing all API endpoints of the application.

use axum::Router;
use axum::{http::StatusCode,
    routing::get,
    response::IntoResponse
};

use crate::dal::DAL;

// Import submodules
pub mod agents;
pub mod stacks;
pub mod deployment_objects;
pub mod agent_events;

/// Shared state for the application
///
/// This struct holds the Data Access Layer (DAL) which is shared across
/// all route handlers to interact with the database.
#[derive(Clone)]
pub struct AppState {
    /// The Data Access Layer instance
    dal: DAL,
}

/// Configures and returns the main application router with all API routes
///
/// This function is responsible for setting up the entire API structure of the application.
/// It merges routes from all submodules and adds a health check endpoint.
///
/// # Arguments
///
/// * `dal` - An instance of the Data Access Layer
///
/// # Returns
///
/// Returns a configured `Router` instance that includes all API routes and middleware.

pub fn configure_api_routes(dal: DAL) -> Router {
    let app_state = AppState { dal };

    Router::new()
        .merge(agents::configure_routes())
        .merge(stacks::configure_routes())
        .merge(deployment_objects::configure_routes())
        .merge(agent_events::configure_routes())
        .route("/healthz", get(healthz))
        .with_state(app_state)
}

/// Health check endpoint handler
///
/// This handler responds to GET requests at the "/healthz" endpoint.
/// It's used to verify that the API is up and running.
///
/// # Returns
///
/// Returns a 200 OK status code with "OK" in the body.
async fn healthz() -> impl IntoResponse {
    (StatusCode::OK, "OK")
}
