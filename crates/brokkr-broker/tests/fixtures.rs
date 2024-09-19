//! This module provides a test fixture for the Brokkr project.
//!
//! It includes functionality to set up a test database, run migrations,
//! and insert test data for various entities like stacks, agents, deployment objects,
//! and agent events.
use brokkr_broker::api;
use brokkr_broker::dal::DAL;
use brokkr_broker::db::create_shared_connection_pool;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use dotenv::dotenv;

use axum::Router;
use brokkr_models::models::{
    agent_annotations::{AgentAnnotation, NewAgentAnnotation},
    agent_events::{AgentEvent, NewAgentEvent},
    agent_labels::{AgentLabel, NewAgentLabel},
    agent_targets::{AgentTarget, NewAgentTarget},
    agents::{Agent, NewAgent},
    deployment_objects::{DeploymentObject, NewDeploymentObject},
    generator::{Generator, NewGenerator},
    stack_annotations::{NewStackAnnotation, StackAnnotation},
    stack_labels::{NewStackLabel, StackLabel},
    stacks::{NewStack, Stack},
};
use brokkr_utils::Settings;
use std::env;

use uuid::Uuid;
/// Embedded migrations for the test database.
pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("../brokkr-models/migrations");

/// Represents a test fixture for the Brokkr project.
#[derive(Clone)]
pub struct TestFixture {
    /// The Data Access Layer (DAL) instance for database operations.
    pub dal: DAL,
    pub settings: Settings,
}

impl Default for TestFixture {
    fn default() -> Self {
        Self::new()
    }
}

impl TestFixture {
    /// Creates and returns an Axum Router with configured API routes.
    ///
    /// # Returns
    ///
    /// Returns a configured Axum Router.
    #[allow(dead_code)]
    pub fn create_test_router(&self) -> Router {
        api::configure_api_routes()
    }

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

        // Run migrations
        let mut conn = connection_pool
            .pool
            .get()
            .expect("Failed to get DB connection");

        // This runs the migrations within the transaction
        conn.run_pending_migrations(MIGRATIONS)
            .expect("Failed to run migrations");

        let dal = DAL::new(connection_pool.pool.clone());
        let settings = Settings::new(None).expect("Failed to load settings");
        TestFixture { dal, settings }
    }

    /// Creates a new stack for testing purposes.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the stack.
    /// * `description` - An optional description for the stack.
    /// * `labels` - An optional vector of labels for the stack.
    /// * `annotations` - An optional vector of key-value pairs for annotations.
    /// * `agent_target` - An optional vector of agent-cluster pairs for targeting.
    ///
    /// # Returns
    ///
    /// Returns the created Stack on success, or panics on failure.
    pub fn create_test_stack(
        &self,
        name: String,
        description: Option<String>,
        generator_id: Uuid,
    ) -> Stack {
        let new_stack =
            NewStack::new(name, description, generator_id).expect("Failed to create NewStack");
        self.dal
            .stacks()
            .create(&new_stack)
            .expect("Failed to create stack")
    }

    /// Creates a new agent for testing purposes.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the agent.
    /// * `cluster_name` - The name of the cluster the agent belongs to.
    ///
    /// # Returns
    ///
    /// Returns the created Agent on success, or panics on failure.
    pub fn create_test_agent(&self, name: String, cluster_name: String) -> Agent {
        let new_agent = NewAgent::new(name, cluster_name).expect("Failed to create NewAgent");
        self.dal
            .agents()
            .create(&new_agent)
            .expect("Failed to create agent")
    }

    /// Creates a new deployment object for testing purposes.
    ///
    /// # Arguments
    ///
    /// * `stack_id` - The UUID of the stack this deployment object belongs to.
    /// * `yaml_content` - The YAML content of the deployment object.
    /// * `is_deletion_marker` - Whether this deployment object marks a deletion.
    ///
    /// # Returns
    ///
    /// Returns the created DeploymentObject on success, or panics on failure.
    pub fn create_test_deployment_object(
        &self,
        stack_id: Uuid,
        yaml_content: String,
        is_deletion_marker: bool,
    ) -> DeploymentObject {
        let new_deployment_object =
            NewDeploymentObject::new(stack_id, yaml_content, is_deletion_marker)
                .expect("Failed to create NewDeploymentObject");
        self.dal
            .deployment_objects()
            .create(&new_deployment_object)
            .expect("Failed to create deployment object")
    }

    /// Creates a new stack label for testing purposes.
    ///
    /// # Arguments
    ///
    /// * `stack_id` - The UUID of the stack to associate the label with.
    /// * `label` - The label text.
    ///
    /// # Returns
    ///
    /// Returns the created StackLabel on success, or panics on failure.
    pub fn create_test_stack_label(&self, stack_id: Uuid, label: String) -> StackLabel {
        let new_label =
            NewStackLabel::new(stack_id, label).expect("Failed to create NewStackLabel");
        self.dal
            .stack_labels()
            .create(&new_label)
            .expect("Failed to create stack label")
    }

    /// Creates a new stack annotation for testing purposes.
    ///
    /// # Arguments
    ///
    /// * `stack_id` - The UUID of the stack to associate the annotation with.
    /// * `key` - The key for the annotation.
    /// * `value` - The value for the annotation.
    ///
    /// # Returns
    ///
    /// Returns the created StackAnnotation on success, or panics on failure.
    pub fn create_test_stack_annotation(
        &self,
        stack_id: Uuid,
        key: &str,
        value: &str,
    ) -> StackAnnotation {
        let new_annotation = NewStackAnnotation {
            stack_id,
            key: key.to_string(),
            value: value.to_string(),
        };
        self.dal
            .stack_annotations()
            .create(&new_annotation)
            .expect("Failed to create stack annotation")
    }

    /// Creates a new agent annotation for testing purposes.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - The UUID of the agent to associate the annotation with.
    /// * `key` - The key for the annotation.
    /// * `value` - The value for the annotation.
    ///
    /// # Returns
    ///
    /// Returns the created AgentAnnotation on success, or panics on failure.
    pub fn create_test_agent_annotation(
        &self,
        agent_id: Uuid,
        key: String,
        value: String,
    ) -> AgentAnnotation {
        let new_annotation = NewAgentAnnotation::new(agent_id, key, value)
            .expect("Failed to create NewAgentAnnotation");
        self.dal
            .agent_annotations()
            .create(&new_annotation)
            .expect("Failed to create agent annotation")
    }

    /// Creates a new agent target for testing purposes.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - The UUID of the agent.
    /// * `stack_id` - The UUID of the stack.
    ///
    /// # Returns
    ///
    /// Returns the created AgentTarget on success, or panics on failure.
    pub fn create_test_agent_target(&self, agent_id: Uuid, stack_id: Uuid) -> AgentTarget {
        let new_target =
            NewAgentTarget::new(agent_id, stack_id).expect("Failed to create NewAgentTarget");
        self.dal
            .agent_targets()
            .create(&new_target)
            .expect("Failed to create agent target")
    }

    /// Creates a new agent event for testing purposes.
    ///
    /// # Arguments
    ///
    /// * `agent` - A reference to the Agent the event belongs to.
    /// * `deployment_object` - A reference to the DeploymentObject associated with the event.
    /// * `event_type` - The type of the event.
    /// * `status` - The status of the event.
    /// * `message` - An optional message for the event.
    ///
    /// # Returns
    ///
    /// Returns the created AgentEvent on success, or panics on failure.
    pub fn create_test_agent_event(
        &self,
        agent: &Agent,
        deployment_object: &DeploymentObject,
        event_type: &str,
        status: &str,
        message: Option<&str>,
    ) -> AgentEvent {
        let new_event = NewAgentEvent {
            agent_id: agent.id,
            deployment_object_id: deployment_object.id,
            event_type: event_type.to_string(),
            status: status.to_string(),
            message: message.map(|m| m.to_string()),
        };
        self.dal
            .agent_events()
            .create(&new_event)
            .expect("Failed to create agent event")
    }

    /// Creates a new agent label for testing purposes.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - The UUID of the agent to associate the label with.
    /// * `label` - The label text.
    ///
    /// # Returns
    ///
    /// Returns the created AgentLabel on success, or panics on failure.
    pub fn create_test_agent_label(&self, agent_id: Uuid, label: String) -> AgentLabel {
        let new_label =
            NewAgentLabel::new(agent_id, label).expect("Failed to create NewAgentLabel");
        self.dal
            .agent_labels()
            .create(&new_label)
            .expect("Failed to create agent label")
    }
    /// Creates a new generator for testing purposes.
    ///
    /// # Arguments
    ///
    /// * `name` - The name of the generator.
    /// * `description` - An optional description for the generator.
    /// * `api_key_hash` - The hashed API key for the generator.
    ///
    /// # Returns
    ///
    /// Returns the created Generator on success, or panics on failure.
    pub fn create_test_generator(
        &self,
        name: String,
        description: Option<String>,
        api_key_hash: String,
    ) -> Generator {
        let new_generator =
            NewGenerator::new(name, description).expect("Failed to create NewGenerator");
        let created_generator = self
            .dal
            .generators()
            .create(&new_generator)
            .expect("Failed to create generator");

        self.dal
            .generators()
            .update_pak_hash(created_generator.id, api_key_hash)
            .expect("Failed to update pak_hash")
    }

    fn reset_database(&self) {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        // Revert all migrations
        conn.revert_all_migrations(MIGRATIONS)
            .expect("Failed to revert migrations");

        // Run all migrations forward
        conn.run_pending_migrations(MIGRATIONS)
            .expect("Failed to run migrations");
    }
}

impl Drop for TestFixture {
    fn drop(&mut self) {
        self.reset_database();
    }
}
