/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

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
pub mod openapi;
pub mod stacks;
pub mod templates;

use crate::dal::DAL;
use axum::middleware::from_fn_with_state;
use axum::Router;
use tower_http::cors::{Any, CorsLayer};

/// Constructs and returns the main router for API v1.
///
/// This function combines all the route handlers from different modules
/// and applies the authentication middleware.
pub fn routes(dal: DAL) -> Router<DAL> {
    // Configure CORS
    let cors = CorsLayer::new()
        .allow_methods(Any)
        .allow_headers(Any)
        .allow_origin(Any);

    let api_routes = Router::new()
        .merge(agent_events::routes())
        .merge(agents::routes())
        .merge(auth::routes())
        .merge(deployment_objects::routes())
        .merge(generators::routes())
        .merge(stacks::routes())
        .merge(templates::routes())
        .layer(from_fn_with_state(
            dal.clone(),
            middleware::auth_middleware::<axum::body::Body>,
        ))
        .layer(cors);

    Router::new()
        .nest("/api/v1", api_routes)
        .merge(openapi::configure_openapi())
}
