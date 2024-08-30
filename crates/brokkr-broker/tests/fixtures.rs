//! This module provides a test fixture for the Brokkr project.
//!
//! It includes functionality to set up a test database, run migrations,
//! and insert test data for various entities like stacks, agents, deployment objects,
//! and agent events.
use brokkr_broker::dal::DAL;
use brokkr_broker::db::create_shared_connection_pool;
use diesel::connection::Connection;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use dotenv::dotenv;

use std::env;
use brokkr_models::models::{
    stacks::{NewStack, Stack},
    agents::{NewAgent,Agent},
    deployment_objects::{NewDeploymentObject,DeploymentObject},
    stack_labels::{NewStackLabel,StackLabel},
    stack_annotations::{NewStackAnnotation,StackAnnotation},
    agent_annotations::{NewAgentAnnotation,AgentAnnotation},
    agent_targets::{NewAgentTarget,AgentTarget}
};

use uuid::Uuid;
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
        let mut conn = connection_pool
            .pool
            .get()
            .expect("Failed to get DB connection");
        conn.begin_test_transaction()
            .expect("Failed to start test transaction");

        // This runs the migrations within the transaction
        conn.run_pending_migrations(MIGRATIONS)
            .expect("Failed to run migrations");

        TestFixture { dal }
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
        description: Option<String>
    ) -> Stack {
        let new_stack = NewStack::new(name, description)
            .expect("Failed to create NewStack");
        self.dal.stacks().create(&new_stack).expect("Failed to create stack")
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
        let new_agent = NewAgent::new(name, cluster_name)
            .expect("Failed to create NewAgent");
        self.dal.agents().create(&new_agent).expect("Failed to create agent")
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
    pub fn create_test_deployment_object(&self, stack_id: Uuid, yaml_content: String, is_deletion_marker: bool) -> DeploymentObject {
        let new_deployment_object = NewDeploymentObject::new(stack_id, yaml_content, is_deletion_marker)
            .expect("Failed to create NewDeploymentObject");
        self.dal.deployment_objects().create(&new_deployment_object)
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
        let new_label = NewStackLabel::new(stack_id, label)
            .expect("Failed to create NewStackLabel");
        self.dal.stack_labels().create(&new_label).expect("Failed to create stack label")
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
    pub fn create_test_stack_annotation(&self, stack_id: Uuid, key: &str, value: &str) -> StackAnnotation {
        let new_annotation = NewStackAnnotation {
            stack_id,
            key: key.to_string(),
            value: value.to_string(),
        };
        self.dal.stack_annotations().create(&new_annotation).expect("Failed to create stack annotation")
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
    pub fn create_test_agent_annotation(&self, agent_id: Uuid, key: String, value: String) -> AgentAnnotation {
        let new_annotation = NewAgentAnnotation::new(agent_id, key, value)
            .expect("Failed to create NewAgentAnnotation");
        self.dal.agent_annotations().create(&new_annotation).expect("Failed to create agent annotation")
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
        let new_target = NewAgentTarget::new(agent_id, stack_id)
            .expect("Failed to create NewAgentTarget");
        self.dal.agent_targets().create(&new_target).expect("Failed to create agent target")
    }

}

impl Drop for TestFixture {
    fn drop(&mut self) {
        // The test transaction will be automatically rolled back when the connection is dropped
    }
}

