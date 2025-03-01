/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

use brokkr_utils::Settings;
use diesel::deserialize::QueryableByName;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sql_query;
use diesel::sql_types::{Integer, Text};
use std::sync::Arc;
use std::thread;
use std::time::Duration;
use url::Url;
use uuid::Uuid;

use brokkr_broker::db::create_shared_connection_pool;

/// Represents a record in the test database table.
#[derive(QueryableByName, PartialEq, Debug)]
struct TestRecord {
    #[diesel(sql_type = Integer)]
    id: i32,
    #[diesel(sql_type = Text)]
    name: String,
}

/// Integration test for the connection pool functionality.
///
/// This test performs the following steps:
/// 1. Creates a temporary test database
/// 2. Sets up a connection pool for the test database
/// 3. Runs multiple tests to verify database operations and connection pool behavior
/// 4. Cleans up by dropping the test database
///
/// The test covers:
/// - Creating a table
/// - Inserting and querying data
/// - Verifying the maximum number of connections
/// - Testing connection timeout when exceeding the maximum connections
#[test]
fn test_connection_pool_integration() {
    // Load settings from default configuration
    let settings = Settings::new(None).expect("Failed to load settings");
    let test_db_name = format!("test_db_{}", Uuid::new_v4().to_string().replace('-', ""));

    let mut url = Url::parse(&settings.database.url).expect("Invalid base URL");
    url.set_path("");
    let base_url_without_db = url.as_str();

    // Create test database
    let manager = ConnectionManager::<PgConnection>::new(base_url_without_db);
    let pool = Pool::builder()
        .build(manager)
        .expect("Failed to create pool");
    let mut conn = pool.get().expect("Failed to get connection");

    sql_query(format!("CREATE DATABASE {}", test_db_name))
        .execute(&mut conn)
        .expect("Failed to create test database");

    // Create connection pool for test database
    let max_size: usize = 10;
    let test_pool =
        create_shared_connection_pool(base_url_without_db, &test_db_name, max_size as u32);
    let test_pool = Arc::new(test_pool);

    // Run tests
    {
        let mut test_conn = test_pool
            .pool
            .get()
            .expect("Failed to get initial connection from test pool");

        // Test 1: Create a table
        sql_query("CREATE TABLE test_table (id SERIAL PRIMARY KEY, name TEXT NOT NULL)")
            .execute(&mut test_conn)
            .expect("Failed to create test table");

        // Test 2: Insert data
        sql_query("INSERT INTO test_table (name) VALUES ('Test Name')")
            .execute(&mut test_conn)
            .expect("Failed to insert data");

        // Test 3: Query data
        let result: TestRecord = sql_query("SELECT id, name FROM test_table")
            .get_result(&mut test_conn)
            .expect("Failed to query data");

        assert_eq!(result.name, "Test Name", "Unexpected query result");

        // Release the initial connection
        drop(test_conn);

        // Test 4: Verify max connections
        let handles: Vec<_> = (0..max_size)
            .map(|i| {
                let pool_clone = Arc::clone(&test_pool);
                thread::spawn(move || {
                    let start = std::time::Instant::now();
                    let conn = pool_clone.pool.get();
                    let duration = start.elapsed();
                    (i, conn, duration)
                })
            })
            .collect();

        let results: Vec<_> = handles
            .into_iter()
            .map(|h| h.join().expect("Thread panicked"))
            .collect();

        let success_count = results.iter().filter(|(_, r, _)| r.is_ok()).count();

        assert_eq!(
            success_count, max_size,
            "Failed to get all {} connections. Got {} successful connections.",
            max_size, success_count
        );

        // Test 5: Attempt to exceed max connections
        let timeout_dur = Duration::from_secs(1);
        let extra_conn = test_pool.pool.get_timeout(timeout_dur);
        assert!(
            extra_conn.is_err(),
            "Expected timeout error when exceeding max connections, but got a connection"
        );
    }

    // Clean up
    drop(test_pool); // Ensure all connections are dropped before dropping the database

    let mut conn = pool.get().expect("Failed to get connection");
    sql_query(format!("DROP DATABASE IF EXISTS {}", test_db_name))
        .execute(&mut conn)
        .expect("Failed to drop test database");
}
