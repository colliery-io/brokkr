//! API v1 module for the Brokkr broker.
//!
//! This module defines the structure and routes for version 1 of the Brokkr API.
//! It includes submodules for various API functionalities and sets up the main router
//! with authentication middleware.

pub mod agent_events;
pub mod agents;
pub mod auth;
pub mod deployment_objects;
pub mod generators;
pub mod middleware;
pub mod stacks;

use crate::dal::DAL;
use axum::middleware::from_fn_with_state;
use axum::Router;

/// Constructs and returns the main router for API v1.
///
/// This function combines routes from various submodules and applies
/// authentication middleware to all routes.
///
/// # Arguments
///
/// * `dal` - The Data Access Layer instance for database operations.
///
/// # Returns
///
/// A `Router<DAL>` instance configured with all API v1 routes and middleware.
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
