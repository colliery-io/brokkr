/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! # API Module
//!
//! This module handles the API routes and configurations for the Brokkr Broker.
//! It includes versioned API endpoints and middleware for authentication and request handling.
//!
//! ## Submodules
//!
//! - `v1`: Contains the version 1 of the API endpoints.
//!
//! ## Main Functions
//!
//! - `configure_api_routes`: Sets up the main application router with all API routes.
//! - `healthz`: Health check endpoint handler.
//! - `readyz`: Ready check endpoint handler.
//! - `metrics`: Metrics endpoint handler.
//!
//! ## API Endpoints
//!
//! ### Authentication
//! - `POST /api/v1/auth/pak`: Verifies a Pre-Authentication Key (PAK).
//!   - Returns: AuthResponse with authentication details.
//!   - Required PAK: Any valid PAK (admin, agent, or generator).
//!
//! ### Agents
//! - `GET /api/v1/agents`: Lists all agents.
//!   - Returns: Array of Agent objects.
//!   - Required PAK: Admin PAK.
//! - `POST /api/v1/agents`: Creates a new agent.
//!   - Returns: Created Agent object and initial PAK.
//!   - Required PAK: Admin PAK.
//! - `GET /api/v1/agents/:id`: Retrieves a specific agent.
//!   - Returns: Agent object.
//!   - Required PAK: Admin PAK or matching Agent PAK.
//! - `PUT /api/v1/agents/:id`: Updates an existing agent.
//!   - Returns: Updated Agent object.
//!   - Required PAK: Admin PAK or matching Agent PAK.
//! - `DELETE /api/v1/agents/:id`: Soft deletes an agent.
//!   - Returns: No content on success.
//!   - Required PAK: Admin PAK.
//! - `GET /api/v1/agents/:id/events`: Lists events for a specific agent.
//!   - Returns: Array of AgentEvent objects.
//!   - Required PAK: Admin PAK or matching Agent PAK.
//! - `POST /api/v1/agents/:id/events`: Creates a new event for a specific agent.
//!   - Returns: Created AgentEvent object.
//!   - Required PAK: Admin PAK or matching Agent PAK.
//! - `GET /api/v1/agents/:id/labels`: Lists labels for a specific agent.
//!   - Returns: Array of AgentLabel objects.
//!   - Required PAK: Admin PAK or matching Agent PAK.
//! - `POST /api/v1/agents/:id/labels`: Adds a new label to a specific agent.
//!   - Returns: Created AgentLabel object.
//!   - Required PAK: Admin PAK only.
//! - `DELETE /api/v1/agents/:id/labels/:label`: Removes a label from a specific agent.
//!   - Returns: No content on success.
//!   - Required PAK: Admin PAK only.
//! - `GET /api/v1/agents/:id/annotations`: Lists annotations for a specific agent.
//!   - Returns: Array of AgentAnnotation objects.
//!   - Required PAK: Admin PAK or matching Agent PAK.
//! - `POST /api/v1/agents/:id/annotations`: Adds a new annotation to a specific agent.
//!   - Returns: Created AgentAnnotation object.
//!   - Required PAK: Admin PAK only.
//! - `DELETE /api/v1/agents/:id/annotations/:key`: Removes an annotation from a specific agent.
//!   - Returns: No content on success.
//!   - Required PAK: Admin PAK only.
//! - `GET /api/v1/agents/:id/targets`: Lists targets for a specific agent.
//!   - Returns: Array of AgentTarget objects.
//!   - Required PAK: Admin PAK or matching Agent PAK.
//! - `POST /api/v1/agents/:id/targets`: Adds a new target to a specific agent.
//!   - Returns: Created AgentTarget object.
//!   - Required PAK: Admin PAK or matching Agent PAK.
//! - `DELETE /api/v1/agents/:id/targets/:stack_id`: Removes a target from a specific agent.
//!   - Returns: No content on success.
//!   - Required PAK: Admin PAK or matching Agent PAK.
//! - `POST /api/v1/agents/:id/heartbeat`: Records a heartbeat for a specific agent.
//!   - Returns: No content on success.
//!   - Required PAK: Matching Agent PAK.
//! - `GET /api/v1/agents/:id/applicable-deployment-objects`: Retrieves applicable deployment objects for a specific agent.
//!   - Returns: Array of DeploymentObject objects.
//!   - Required PAK: Admin PAK or matching Agent PAK.
//!
//! ### Generators
//! - `GET /api/v1/generators`: Lists all generators.
//!   - Returns: Array of Generator objects.
//!   - Required PAK: Admin PAK.
//! - `POST /api/v1/generators`: Creates a new generator.
//!   - Returns: Created Generator object and its PAK.
//!   - Required PAK: Admin PAK.
//! - `GET /api/v1/generators/:id`: Retrieves a specific generator.
//!   - Returns: Generator object.
//!   - Required PAK: Admin PAK or matching Generator PAK.
//! - `PUT /api/v1/generators/:id`: Updates an existing generator.
//!   - Returns: Updated Generator object.
//!   - Required PAK: Admin PAK or matching Generator PAK.
//! - `DELETE /api/v1/generators/:id`: Soft deletes a generator.
//!   - Returns: No content on success.
//!   - Required PAK: Admin PAK or matching Generator PAK.
//!
//! ### Stacks
//! - `GET /api/v1/stacks`: Lists all stacks.
//!   - Returns: Array of Stack objects.
//!   - Required PAK: Admin PAK.
//! - `POST /api/v1/stacks`: Creates a new stack.
//!   - Returns: Created Stack object.
//!   - Required PAK: Admin PAK or Generator PAK (for self).
//! - `GET /api/v1/stacks/:id`: Retrieves a specific stack.
//!   - Returns: Stack object.
//!   - Required PAK: Admin PAK or associated Generator PAK.
//! - `PUT /api/v1/stacks/:id`: Updates an existing stack.
//!   - Returns: Updated Stack object.
//!   - Required PAK: Admin PAK or associated Generator PAK.
//! - `DELETE /api/v1/stacks/:id`: Soft deletes a stack.
//!   - Returns: No content on success.
//!   - Required PAK: Admin PAK or associated Generator PAK.
//! - `GET /api/v1/stacks/:id/deployment-objects`: Lists deployment objects for a specific stack.
//!   - Returns: Array of DeploymentObject objects.
//!   - Required PAK: Admin PAK or associated Generator PAK.
//! - `POST /api/v1/stacks/:id/deployment-objects`: Creates a new deployment object for a specific stack.
//!   - Returns: Created DeploymentObject object.
//!   - Required PAK: Admin PAK or associated Generator PAK.
//! - `GET /api/v1/stacks/:id/labels`: Lists labels for a specific stack.
//!   - Returns: Array of StackLabel objects.
//!   - Required PAK: Admin PAK, associated Generator PAK, or Agent PAK with target.
//! - `POST /api/v1/stacks/:id/labels`: Adds a new label to a specific stack.
//!   - Returns: Created StackLabel object.
//!   - Required PAK: Admin PAK or associated Generator PAK.
//! - `DELETE /api/v1/stacks/:id/labels/:label`: Removes a label from a specific stack.
//!   - Returns: No content on success.
//!   - Required PAK: Admin PAK or associated Generator PAK.
//! - `GET /api/v1/stacks/:id/annotations`: Lists annotations for a specific stack.
//!   - Returns: Array of StackAnnotation objects.
//!   - Required PAK: Admin PAK, associated Generator PAK, or Agent PAK with target.
//! - `POST /api/v1/stacks/:id/annotations`: Adds a new annotation to a specific stack.
//!   - Returns: Created StackAnnotation object.
//!   - Required PAK: Admin PAK or associated Generator PAK.
//! - `DELETE /api/v1/stacks/:id/annotations/:key`: Removes an annotation from a specific stack.
//!   - Returns: No content on success.
//!   - Required PAK: Admin PAK or associated Generator PAK.
//!
//! ### Deployment Objects
//! - `GET /api/v1/deployment-objects/:id`: Retrieves a specific deployment object.
//!   - Returns: DeploymentObject object.
//!   - Required PAK: Admin PAK, associated Generator PAK, or Agent PAK with target.
//!
//! ### Agent Events
//! - `GET /api/v1/agent-events`: Lists all agent events.
//!   - Returns: Array of AgentEvent objects.
//!   - Required PAK: Any valid PAK.
//! - `GET /api/v1/agent-events/:id`: Retrieves a specific agent event.
//!   - Returns: AgentEvent object.
//!   - Required PAK: Any valid PAK.

pub mod v1;
use crate::dal::DAL;
use axum::{response::IntoResponse, routing::get, Router};
use hyper::StatusCode;

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
pub fn configure_api_routes(dal: DAL) -> Router<DAL> {
    Router::new()
        .merge(v1::routes(dal.clone()))
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
/// It's used to provide metrics about the API's operation.
///
/// # Returns
///
/// Returns a 200 OK status code with metrics data in the body.
async fn metrics() -> impl IntoResponse {
    (StatusCode::OK, "Metrics")
}
