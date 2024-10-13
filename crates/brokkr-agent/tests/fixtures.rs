// //! This module provides a test fixture for the Brokkr project.
// //!
// //! It includes functionality to set up a test database, run migrations,
// //! and insert test data for various entities like stacks, agents, deployment objects,
// //! and agent events.
// use axum::Router;
// use brokkr_broker::api;
// use brokkr_broker::dal::DAL;
// use brokkr_broker::db::create_shared_connection_pool;
// use brokkr_broker::utils;
// use brokkr_broker::utils::pak;
// use brokkr_models::models::{
//     agent_annotations::{AgentAnnotation, NewAgentAnnotation},
//     agent_events::{AgentEvent, NewAgentEvent},
//     agent_labels::{AgentLabel, NewAgentLabel},
//     agent_targets::{AgentTarget, NewAgentTarget},
//     agents::{Agent, NewAgent},
//     deployment_objects::{DeploymentObject, NewDeploymentObject},
//     generator::{Generator, NewGenerator},
//     stack_annotations::{NewStackAnnotation, StackAnnotation},
//     stack_labels::{NewStackLabel, StackLabel},
//     stacks::{NewStack, Stack},
// };
// use brokkr_utils::Settings;
// use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
// use dotenv::dotenv;
// use std::env;
// use uuid::Uuid;

// /// Embedded migrations for the test database.
// pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("../brokkr-models/migrations");

// /// Represents a test fixture for the Brokkr project.

// #[allow(dead_code)]
// #[derive(Clone)]
// pub struct TestFixture {
//     /// The Data Access Layer (DAL) instance for database operations.
//     pub dal: DAL,
//     pub settings: Settings,
//     pub admin_pak: String,
//     pub admin_generator: Generator,
// }

// impl Default for TestFixture {
//     fn default() -> Self {
//         Self::new()
//     }
// }

// impl TestFixture {
//     /// Creates and returns an Axum Router with configured API routes.
//     ///
//     /// # Returns
//     ///
//     /// Returns a configured Axum Router.
//     #[allow(dead_code)]
//     pub fn create_test_router(&self) -> Router<DAL> {
//         api::configure_api_routes(self.dal.clone())
//     }

//     /// Creates a new TestFixture instance.
//     ///
//     /// This method sets up a test database connection, runs migrations,
//     /// and prepares the environment for testing.
//     ///
//     /// # Returns
//     ///
//     /// Returns a new TestFixture instance.
//     ///
//     /// # Panics
//     ///
//     /// This method will panic if:
//     /// * The DATABASE_URL environment variable is not set
//     /// * It fails to create a database connection
//     /// * It fails to run migrations
//     pub fn new() -> Self {
//         dotenv().ok();
//         let settings = Settings::new(None).expect("Failed to load settings");
//         let connection_pool = create_shared_connection_pool(&settings.database.url  , "brokkr", 5);
//         // Run migrations
//         let mut conn = connection_pool
//             .pool
//             .get()
//             .expect("Failed to get DB connection");

//         // This runs the migrations within the transaction
//         conn.run_pending_migrations(MIGRATIONS)
//             .expect("Failed to run migrations");


//         // this initializes the PAK controller and runs the initial startup logic for the broker
//         utils::pak::create_pak_controller(Some(&settings))
//             .expect("Failed to create PAK controller");
//         utils::first_startup(&mut conn).expect("Failed to run first startup");

//         // Read the admin PAK from the temporary file
//         let admin_pak_path = std::env::temp_dir().join("/tmp/brokkr-keys/key.txt");
//         let admin_pak = std::fs::read_to_string(admin_pak_path)
//             .expect("Failed to read admin PAK from temporary file")
//             .trim()
//             .to_string();

//         let dal = DAL::new(connection_pool.pool.clone());

//         // Fetch the admin generator
//         let admin_generator = dal
//             .generators()
//             .get_by_name("admin-generator")
//             .expect("Failed to get admin generator")
//             .expect("Admin generator not found");

//         TestFixture {
//             dal,
//             settings,
//             admin_pak,
//             admin_generator,
//         }
//     }

 

//     fn reset_database(&self) {
//         let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
//         // Revert all migrations
//         conn.revert_all_migrations(MIGRATIONS)
//             .expect("Failed to revert migrations");

//         // Run all migrations forward
//         conn.run_pending_migrations(MIGRATIONS)
//             .expect("Failed to run migrations");
//     }
// }

// impl Drop for TestFixture {
//     fn drop(&mut self) {
//         self.reset_database();
//     }
// }
