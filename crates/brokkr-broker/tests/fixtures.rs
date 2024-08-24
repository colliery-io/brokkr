//! This module provides a test fixture for the Brokkr project.
//!
//! It includes functionality to set up a test database, run migrations,
//! and insert test data for various entities like stacks, agents, deployment objects,
//! and agent events.

use brokkr_broker::dal::DAL;
use brokkr_broker::db::create_shared_connection_pool;
use brokkr_broker::api;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use diesel::connection::Connection;
use dotenv::dotenv;
use std::env;
use uuid::Uuid;
use axum::Router;
use tower::ServiceExt;
use hyper::{Request, Method, Body,StatusCode};
use serde_json::json;


use brokkr_models::models::{NewStack, Stack, DeploymentObject, NewDeploymentObject, NewAgent, Agent, AgentEvent, NewAgentEvent};

/// Embedded migrations for the test database.
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("../brokkr-models/migrations");

/// Represents a test fixture for the Brokkr project.
#[derive(Clone)]
pub struct TestFixture {
    /// The Data Access Layer (DAL) instance for database operations.
    pub dal: DAL,
}

impl TestFixture {
    /// Creates a new TestFixture instance.
    ///
    /// This method sets up a test database connection, runs migrations,
    /// and prepares the environment for testing.
    ///
    /// # Returns
    ///
    /// Returns a new TestFixture instance.
    ///
    /// # Panics
    ///
    /// This method will panic if:
    /// * The DATABASE_URL environment variable is not set
    /// * It fails to create a database connection
    /// * It fails to run migrations
    pub fn new() -> Self {
        dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        
        let connection_pool = create_shared_connection_pool(&database_url, "brokkr", 5);
        let dal = DAL::new(connection_pool.pool.clone());

        // Run migrations
        let mut conn = connection_pool.pool.get().expect("Failed to get DB connection");
        conn.begin_test_transaction().expect("Failed to start test transaction");
        
        // This runs the migrations within the transaction
        conn.run_pending_migrations(MIGRATIONS).expect("Failed to run migrations");

        TestFixture { dal }
    }

    /// Inserts a test stack into the database.
    ///
    /// # Returns
    ///
    /// Returns the UUID of the created stack.
    pub fn insert_test_stack(&self) -> Uuid {
        let new_stack = NewStack::new(
            format!("Test Stack {}", Uuid::new_v4()),  // Ensure unique name
            Some("Test Description".to_string()),
            Some(vec!["test".to_string()]),
            Some(vec![("key".to_string(), "value".to_string())]),
            Some(vec!["agent1".to_string()]),
        ).expect("Failed to create NewStack");

        let created_stack = self.dal.stacks().create(&new_stack)
            .expect("Failed to create stack");
        
        created_stack.id
    }

    /// Inserts a test agent event into the database.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - The UUID of the agent associated with this event.
    /// * `deployment_object_id` - The UUID of the deployment object associated with this event.
    ///
    /// # Returns
    ///
    /// Returns the created AgentEvent.
    pub fn insert_test_agent_event(&self, agent_id: Uuid, deployment_object_id: Uuid) -> AgentEvent {
        let new_agent_event = NewAgentEvent::new(
            agent_id,
            deployment_object_id,
            format!("Test Event {}", Uuid::new_v4()),
            "success".to_string(),
            Some("Test event message".to_string()),
        ).unwrap();

        self.dal.agent_events().create(&new_agent_event)
            .expect("Failed to create test agent event")
    }

    /// Inserts a test deployment object into the database.
    ///
    /// # Arguments
    ///
    /// * `stack_id` - The UUID of the stack associated with this deployment object.
    ///
    /// # Returns
    ///
    /// Returns the created DeploymentObject.
    pub fn insert_test_deployment_object(&self, stack_id: Uuid) -> DeploymentObject {
        let new_deployment_object = NewDeploymentObject::new(
            stack_id,
            format!("key: value{}", Uuid::new_v4()),  // Ensure unique content
            false,
        ).expect("Failed to create NewDeploymentObject");

        self.dal.deployment_objects().create(&new_deployment_object)
            .expect("Failed to create deployment object")
    }

    /// Inserts a test agent into the database.
    ///
    /// # Returns
    ///
    /// Returns the created Agent.
    pub fn insert_test_agent(&self) -> Agent {
        let new_agent = NewAgent::new(
            format!("Test Agent {}", Uuid::new_v4()),
            "Test Cluster".to_string(),
            Some(vec!["test".to_string(), "fixture".to_string()]),
            Some(vec![("key".to_string(), "value".to_string())]),
        ).expect("Failed to create NewAgent");

        self.dal.agents().create(&new_agent)
            .expect("Failed to create test agent")
    }

    /// Creates and returns an Axum Router with configured API routes.
    ///
    /// # Returns
    ///
    /// Returns a configured Axum Router.
    pub fn create_test_router(&self) -> Router {
        api::configure_api_routes(self.dal.clone())
    }

}

impl Drop for TestFixture {
    fn drop(&mut self) {
        // The test transaction will be automatically rolled back when the connection is dropped
    }
}


pub async fn create_test_agent(app: &axum::Router) -> Agent {
    let new_agent = NewAgent::new(
        "Test Agent".to_string(),
        "Test Cluster".to_string(),
        Some(vec!["test".to_string()]),
        Some(vec![("key".to_string(), "value".to_string())]),
    ).unwrap();

    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/agents")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&new_agent).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_response.status(), StatusCode::CREATED);

    let body = hyper::body::to_bytes(create_response.into_body()).await.unwrap();
    serde_json::from_slice(&body).unwrap()
}



pub async fn create_test_stack(app: &axum::Router) -> Stack {
    let new_stack = NewStack::new(
        "Test Stack".to_string(),
        Some("Test Description".to_string()),
        Some(vec!["test".to_string()]),
        Some(vec![("key".to_string(), "value".to_string())]),
        Some(vec!["agent1".to_string()]),
    ).unwrap();

    let create_response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/stacks")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&new_stack).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(create_response.status(), StatusCode::OK);

    let body = hyper::body::to_bytes(create_response.into_body()).await.unwrap();
    serde_json::from_slice(&body).unwrap()
}


/// Creates a test deployment object and returns it.
///
/// This function sends a POST request to create a new deployment object
/// and returns the created object.
///
/// # Arguments
///
/// * `app` - The test application router
/// * `stack_id` - The UUID of the stack to associate with the deployment object
///
/// # Returns
///
/// The created DeploymentObject
pub async fn create_test_deployment_object(app: &Router, stack_id: Uuid) -> DeploymentObject {
    let new_stack_object =  create_test_stack(app).await;

    
    let new_deployment_object = NewDeploymentObject {
        stack_id: new_stack_object.id,
        yaml_content: "test: content".to_string(),
        yaml_checksum: "test_checksum".to_string(),
        is_deletion_marker: false,
    };

    let response = app
        .clone()
        .oneshot(
            Request::builder()
                .method(Method::POST)
                .uri("/deployment-objects")
                .header("Content-Type", "application/json")
                .body(Body::from(serde_json::to_string(&new_deployment_object).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();

    assert_eq!(response.status(), StatusCode::CREATED);

    let body = hyper::body::to_bytes(response.into_body()).await.unwrap();
    serde_json::from_slice(&body).unwrap()
}
