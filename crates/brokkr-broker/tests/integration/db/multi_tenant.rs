/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Integration tests for multi-tenant schema isolation functionality

use brokkr_broker::dal::DAL;
use brokkr_broker::db::create_shared_connection_pool;
use brokkr_models::models::agents::NewAgent;
use brokkr_utils::Settings;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sql_query;
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};
use url::Url;
use uuid::Uuid;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("../brokkr-models/migrations");

/// Helper function to create a test database
fn create_test_database(base_url: &str) -> String {
    let test_db_name = format!("test_db_{}", Uuid::new_v4().to_string().replace('-', ""));

    let manager = ConnectionManager::<PgConnection>::new(base_url);
    let pool = Pool::builder()
        .build(manager)
        .expect("Failed to create pool");
    let mut conn = pool.get().expect("Failed to get connection");

    sql_query(format!("CREATE DATABASE {}", test_db_name))
        .execute(&mut conn)
        .expect("Failed to create test database");

    test_db_name
}

/// Helper function to drop a test database
fn drop_test_database(base_url: &str, db_name: &str) {
    let manager = ConnectionManager::<PgConnection>::new(base_url);
    let pool = Pool::builder()
        .build(manager)
        .expect("Failed to create pool");
    let mut conn = pool.get().expect("Failed to get connection");

    // Terminate all connections to the database first
    sql_query(format!(
        "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '{}'",
        db_name
    ))
    .execute(&mut conn)
    .ok();

    sql_query(format!("DROP DATABASE IF EXISTS {}", db_name))
        .execute(&mut conn)
        .expect("Failed to drop test database");
}

/// Test complete data isolation between different schemas
///
/// This test verifies that:
/// - Data created in schema_a is not visible from schema_b
/// - Each schema maintains its own isolated data
/// - DAL operations respect schema boundaries
#[test]
fn test_schema_isolation() {
    let settings = Settings::new(None).expect("Failed to load settings");
    let mut url = Url::parse(&settings.database.url).expect("Invalid base URL");
    url.set_path("");
    let base_url = url.as_str();

    // Create test database
    let test_db_name = create_test_database(base_url);

    // Create two connection pools with different schemas
    let pool_a = create_shared_connection_pool(base_url, &test_db_name, 5, Some("schema_a"));
    let pool_b = create_shared_connection_pool(base_url, &test_db_name, 5, Some("schema_b"));

    // Setup schemas and run migrations
    pool_a
        .setup_schema("schema_a")
        .expect("Failed to setup schema_a");
    let mut conn_a = pool_a.get().expect("Failed to get connection");
    conn_a
        .run_pending_migrations(MIGRATIONS)
        .expect("Failed to run migrations in schema_a");
    drop(conn_a);

    pool_b
        .setup_schema("schema_b")
        .expect("Failed to setup schema_b");
    let mut conn_b = pool_b.get().expect("Failed to get connection");
    conn_b
        .run_pending_migrations(MIGRATIONS)
        .expect("Failed to run migrations in schema_b");
    drop(conn_b);

    // Create DAL instances for each schema
    let dal_a = DAL::new(pool_a.clone());
    let dal_b = DAL::new(pool_b.clone());

    // Create an agent in schema_a
    let new_agent = NewAgent::new("test_agent".to_string(), "test_cluster".to_string())
        .expect("Failed to create NewAgent");
    let agent_a = dal_a
        .agents()
        .create(&new_agent)
        .expect("Failed to create agent in schema_a");

    // Query from schema_a - should find the agent
    let agents_in_a = dal_a
        .agents()
        .list()
        .expect("Failed to list agents in schema_a");
    assert_eq!(
        agents_in_a.len(),
        1,
        "Expected 1 agent in schema_a, found {}",
        agents_in_a.len()
    );
    assert_eq!(
        agents_in_a[0].id, agent_a.id,
        "Agent ID mismatch in schema_a"
    );

    // Query from schema_b - should find NO agents (data isolation)
    let agents_in_b = dal_b
        .agents()
        .list()
        .expect("Failed to list agents in schema_b");
    assert_eq!(
        agents_in_b.len(),
        0,
        "Expected 0 agents in schema_b (data isolation), found {}",
        agents_in_b.len()
    );

    // Create a different agent in schema_b
    let new_agent_b = NewAgent::new("test_agent_b".to_string(), "test_cluster_b".to_string())
        .expect("Failed to create NewAgent");
    let agent_b = dal_b
        .agents()
        .create(&new_agent_b)
        .expect("Failed to create agent in schema_b");

    // Verify schema_a still only has 1 agent
    let agents_in_a_after = dal_a
        .agents()
        .list()
        .expect("Failed to list agents in schema_a after");
    assert_eq!(
        agents_in_a_after.len(),
        1,
        "Expected 1 agent in schema_a after schema_b insert, found {}",
        agents_in_a_after.len()
    );

    // Verify schema_b has 1 agent (different from schema_a)
    let agents_in_b_after = dal_b
        .agents()
        .list()
        .expect("Failed to list agents in schema_b after");
    assert_eq!(
        agents_in_b_after.len(),
        1,
        "Expected 1 agent in schema_b, found {}",
        agents_in_b_after.len()
    );
    assert_eq!(
        agents_in_b_after[0].id, agent_b.id,
        "Agent ID mismatch in schema_b"
    );

    // Clean up
    drop(dal_a);
    drop(dal_b);
    drop(pool_a);
    drop(pool_b);
    drop_test_database(base_url, &test_db_name);
}

/// Test automatic schema provisioning on first connection
///
/// This test verifies that:
/// - Schema is created automatically when setup_schema() is called
/// - Migrations run successfully in the new schema
/// - The schema is usable for normal operations
#[test]
fn test_schema_auto_provisioning() {
    let settings = Settings::new(None).expect("Failed to load settings");
    let mut url = Url::parse(&settings.database.url).expect("Invalid base URL");
    url.set_path("");
    let base_url = url.as_str();

    // Create test database
    let test_db_name = create_test_database(base_url);

    // Create connection pool with a new schema
    let pool = create_shared_connection_pool(base_url, &test_db_name, 5, Some("auto_test"));

    // Setup schema - should create it automatically
    pool.setup_schema("auto_test")
        .expect("Failed to auto-provision schema");

    // Run migrations - should succeed
    let mut conn = pool.get().expect("Failed to get connection");
    conn.run_pending_migrations(MIGRATIONS)
        .expect("Failed to run migrations in auto-provisioned schema");
    drop(conn);

    // Verify schema is usable - create an agent
    let dal = DAL::new(pool.clone());
    let new_agent = NewAgent::new("auto_agent".to_string(), "auto_cluster".to_string())
        .expect("Failed to create NewAgent");
    let agent = dal
        .agents()
        .create(&new_agent)
        .expect("Failed to create agent in auto-provisioned schema");

    // Query to verify
    let agents = dal
        .agents()
        .list()
        .expect("Failed to list agents in auto-provisioned schema");
    assert_eq!(
        agents.len(),
        1,
        "Expected 1 agent in auto-provisioned schema"
    );
    assert_eq!(agents[0].id, agent.id, "Agent ID mismatch");

    // Clean up
    drop(dal);
    drop(pool);
    drop_test_database(base_url, &test_db_name);
}

/// Test backward compatibility with no schema (public schema)
///
/// This test verifies that:
/// - When no schema is specified (None), operations use the public schema
/// - Existing functionality continues to work without schema configuration
/// - This ensures backward compatibility with existing deployments
#[test]
fn test_backward_compatibility_no_schema() {
    let settings = Settings::new(None).expect("Failed to load settings");
    let mut url = Url::parse(&settings.database.url).expect("Invalid base URL");
    url.set_path("");
    let base_url = url.as_str();

    // Create test database
    let test_db_name = create_test_database(base_url);

    // Create connection pool with NO schema (backward compatibility)
    let pool = create_shared_connection_pool(base_url, &test_db_name, 5, None);

    // Run migrations in public schema
    let mut conn = pool.get().expect("Failed to get connection");
    conn.run_pending_migrations(MIGRATIONS)
        .expect("Failed to run migrations in public schema");
    drop(conn);

    // Create DAL and use it (should work in public schema)
    let dal = DAL::new(pool.clone());
    let new_agent = NewAgent::new("public_agent".to_string(), "public_cluster".to_string())
        .expect("Failed to create NewAgent");
    let agent = dal
        .agents()
        .create(&new_agent)
        .expect("Failed to create agent in public schema");

    // Query to verify
    let agents = dal
        .agents()
        .list()
        .expect("Failed to list agents in public schema");
    assert_eq!(agents.len(), 1, "Expected 1 agent in public schema");
    assert_eq!(agents[0].id, agent.id, "Agent ID mismatch in public schema");

    // Clean up
    drop(dal);
    drop(pool);
    drop_test_database(base_url, &test_db_name);
}

/// Test schema name validation
///
/// This test verifies that:
/// - Invalid schema names are rejected
/// - Only safe schema names (alphanumeric + underscore, starting with letter) are allowed
/// - SQL injection attempts via schema names are prevented
#[test]
fn test_invalid_schema_name() {
    use brokkr_broker::db::validate_schema_name;

    // Valid schema names
    assert!(validate_schema_name("valid_schema").is_ok());
    assert!(validate_schema_name("schema123").is_ok());
    assert!(validate_schema_name("my_schema_123").is_ok());

    // Invalid schema names
    assert!(
        validate_schema_name("").is_err(),
        "Empty string should be invalid"
    );
    assert!(
        validate_schema_name("1invalid").is_err(),
        "Schema starting with number should be invalid"
    );
    assert!(
        validate_schema_name("has-dash").is_err(),
        "Schema with dash should be invalid"
    );
    assert!(
        validate_schema_name("has space").is_err(),
        "Schema with space should be invalid"
    );
    assert!(
        validate_schema_name("has.dot").is_err(),
        "Schema with dot should be invalid"
    );
    assert!(
        validate_schema_name("has;semicolon").is_err(),
        "Schema with semicolon should be invalid (SQL injection attempt)"
    );
    assert!(
        validate_schema_name("has'quote").is_err(),
        "Schema with quote should be invalid (SQL injection attempt)"
    );
}
