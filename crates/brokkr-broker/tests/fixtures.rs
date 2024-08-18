use brokkr_broker::dal::DAL;
use brokkr_broker::db::create_shared_connection_pool;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use diesel::connection::Connection;
use dotenv::dotenv;
use std::env;
use uuid::Uuid;

use brokkr_models::models::NewStack;
use brokkr_models::models::DeploymentObject;
use brokkr_models::models::NewDeploymentObject;



pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("../brokkr-models/migrations");

pub struct TestFixture {
    pub dal: DAL,
}

impl TestFixture {
    pub fn new() -> Self {
        dotenv().ok();
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
        
        let connection_pool = create_shared_connection_pool(&database_url, "brokkr",5);
        let dal = DAL::new(connection_pool.pool.clone());

        // Run migrations
        let mut conn = connection_pool.pool.get().expect("Failed to get DB connection");
        conn.begin_test_transaction().expect("Failed to start test transaction");
        
        // This runs the migrations within the transaction
        conn.run_pending_migrations(MIGRATIONS).expect("Failed to run migrations");

        TestFixture { dal }
    }

    pub fn create_test_stack(&self) -> Uuid {
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

    pub fn create_test_deployment_object(&self, stack_id: Uuid) -> DeploymentObject {
        let new_deployment_object = NewDeploymentObject::new(
            stack_id,
            format!("key: value{}", Uuid::new_v4()),  // Ensure unique content
            format!("checksum{}", Uuid::new_v4()),    // Ensure unique checksum
            self.get_next_sequence_id(stack_id),
            false,
        ).expect("Failed to create NewDeploymentObject");

        self.dal.deployment_objects().create(&new_deployment_object)
            .expect("Failed to create deployment object")
    }

    fn get_next_sequence_id(&self, stack_id: Uuid) -> i64 {
        let existing_objects = self.dal.deployment_objects().get_by_stack_id(stack_id)
            .expect("Failed to get existing deployment objects");
        
        existing_objects.iter()
            .map(|obj| obj.sequence_id)
            .max()
            .map_or(1, |max| max + 1)
    }
}

impl Drop for TestFixture {
    fn drop(&mut self) {
        // The test transaction will be automatically rolled back when the connection is dropped
    }
}