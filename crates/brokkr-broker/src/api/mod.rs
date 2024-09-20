// //! # API Routes Aggregator Module
// //!
// //! This module aggregates all API routes and provides a function to configure the main router.
// //! It serves as the central point for organizing and initializing all API endpoints of the application.

pub mod v1;
use crate::dal::DAL;
use axum::{response::IntoResponse, routing::get, Router};
use hyper::StatusCode;

// /// Configures and returns the main application router with all API routes
// ///
// /// This function is responsible for setting up the entire API structure of the application.
// /// It merges routes from all submodules and adds a health check endpoint.
// ///
// /// # Arguments
// ///
// /// * `dal` - An instance of the Data Access Layer
// ///
// /// # Returns
// ///
// /// Returns a configured `Router` instance that includes all API routes and middleware.

pub fn configure_api_routes(dal: DAL) -> Router<DAL> {
    Router::new()
        .nest("/api/v1", v1::routes(dal))
        .route("/healthz", get(healthz))
        .route("/readyz", get(readyz))
        .route("/metrics", get(metrics))
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

/// Ready check endpoint handler
///
/// This handler responds to GET requests at the "/readyz" endpoint.
/// It's used to verify that the API is ready for use.
///
/// # Returns
///
/// Returns a 200 OK status code with "Ready" in the body.
async fn readyz() -> impl IntoResponse {
    (StatusCode::OK, "Ready")
}

/// Metrics endpoint handler
///
/// This handler responds to GET requests at the "/metrics" endpoint.
/// It's used to retrieve metrics data.
///
/// # Returns
///
/// Returns a 200 OK status code with "Metrics data" in the body.
async fn metrics() -> impl IntoResponse {
    // Implement metrics collection and formatting here
    (StatusCode::OK, "Metrics data")
}
