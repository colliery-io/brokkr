// //! # API Routes Aggregator Module
// //!
// //! This module aggregates all API routes and provides a function to configure the main router.
// //! It serves as the central point for organizing and initializing all API endpoints of the application.

pub mod v1;
use crate::dal::DAL;

use axum::{
    Router,
    routing::get,
    response::IntoResponse
    };

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

pub fn configure_api_routes(dal: DAL) -> Router {

    Router::new()
        .merge(v1::configure_routes())
        .route("/healthz", get(healthz))
        .with_state(dal)
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
