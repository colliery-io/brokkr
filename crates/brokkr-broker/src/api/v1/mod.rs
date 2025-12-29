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
pub mod diagnostics;
pub mod generators;
pub mod health;
pub mod middleware;
pub mod openapi;
pub mod stacks;
pub mod templates;
pub mod webhooks;
pub mod work_orders;

use crate::dal::DAL;
use axum::middleware::from_fn_with_state;
use axum::Router;
use brokkr_utils::config::Cors;
use brokkr_utils::logging::prelude::*;
use hyper::{header::HeaderName, Method};
use std::time::Duration;
use tower_http::cors::CorsLayer;

/// Constructs and returns the main router for API v1.
///
/// This function combines all the route handlers from different modules
/// and applies the authentication middleware.
pub fn routes(dal: DAL, cors_config: &Cors) -> Router<DAL> {
    // Configure CORS from settings
    let cors = build_cors_layer(cors_config);

    let api_routes = Router::new()
        .merge(agent_events::routes())
        .merge(agents::routes())
        .merge(auth::routes())
        .merge(deployment_objects::routes())
        .merge(diagnostics::routes())
        .merge(generators::routes())
        .merge(health::routes())
        .merge(stacks::routes())
        .merge(templates::routes())
        .merge(webhooks::routes())
        .merge(work_orders::routes())
        .merge(work_orders::agent_routes())
        .layer(from_fn_with_state(
            dal.clone(),
            middleware::auth_middleware::<axum::body::Body>,
        ))
        .layer(cors);

    Router::new()
        .nest("/api/v1", api_routes)
        .merge(openapi::configure_openapi())
}

/// Builds a CORS layer from configuration.
///
/// If "*" is in the allowed_origins list, allows all origins.
/// Otherwise, restricts to the configured origins.
fn build_cors_layer(config: &Cors) -> CorsLayer {
    let mut cors = CorsLayer::new();

    // Handle allowed origins
    if config.allowed_origins.iter().any(|o| o == "*") {
        info!("CORS: Allowing all origins (not recommended for production)");
        cors = cors.allow_origin(tower_http::cors::Any);
    } else {
        let origins: Vec<_> = config
            .allowed_origins
            .iter()
            .filter_map(|o| o.parse().ok())
            .collect();
        info!("CORS: Allowing origins: {:?}", config.allowed_origins);
        cors = cors.allow_origin(origins);
    }

    // Handle allowed methods
    let methods: Vec<Method> = config
        .allowed_methods
        .iter()
        .filter_map(|m| m.parse().ok())
        .collect();
    cors = cors.allow_methods(methods);

    // Handle allowed headers
    let headers: Vec<HeaderName> = config
        .allowed_headers
        .iter()
        .filter_map(|h| h.parse().ok())
        .collect();
    cors = cors.allow_headers(headers);

    // Set max age
    cors = cors.max_age(Duration::from_secs(config.max_age_seconds));

    cors
}
