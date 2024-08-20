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

// Import the module containing your connection pool code
use brokkr_broker::db::create_shared_connection_pool;

#[derive(QueryableByName, PartialEq, Debug)]
struct TestRecord {
    #[diesel(sql_type = Integer)]
    id: i32,
    #[diesel(sql_type = Text)]
    name: String,
}

#[test]
fn test_connection_pool_integration() {
    // Set up
    let base_url = std::env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set for integration tests");
    let test_db_name = format!("test_db_{}", Uuid::new_v4().to_string().replace("-", ""));
    
    let mut url = Url::parse(&base_url).expect("Invalid base URL");
    url.set_path("");
    let base_url_without_db = url.as_str();

    // Create test database
    let manager = ConnectionManager::<PgConnection>::new(base_url_without_db);
    let pool = Pool::builder().build(manager).expect("Failed to create pool");
    let mut conn = pool.get().expect("Failed to get connection");

    sql_query(format!("CREATE DATABASE {}", test_db_name))
        .execute(&mut conn)
        .expect("Failed to create test database");

    // Create connection pool for test database
    let max_size: usize = 10;
    let test_pool = create_shared_connection_pool(base_url_without_db, &test_db_name, max_size as u32);
    let test_pool = Arc::new(test_pool);

    // Run tests
    {
        let mut test_conn = test_pool.pool.get().expect("Failed to get initial connection from test pool");

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
                    match &conn {
                        Ok(_) => println!("Thread {} got a connection after {:?}", i, duration),
                        Err(e) => println!("Thread {} failed to get a connection after {:?}: {:?}", i, duration, e),
                    }
                    (i, conn, duration)
                })
            })
            .collect();

        let results: Vec<_> = handles
            .into_iter()
            .map(|h| h.join().expect("Thread panicked"))
            .collect();

        let success_count = results.iter().filter(|(_, r, _)| r.is_ok()).count();
       
        for (i, result, duration) in &results {
            match result {
                Ok(_) => println!("Connection {} succeeded after {:?}", i, duration),
                Err(e) => println!("Connection {} failed after {:?}: {:?}", i, duration, e),
            }
        }

        assert_eq!(success_count, max_size, "Failed to get all {} connections. Got {} successful connections.", max_size, success_count);

        // Attempting to get one more connection should time out
        let timeout_dur = Duration::from_secs(1);
        let extra_conn = test_pool.pool.get_timeout(timeout_dur);
        assert!(extra_conn.is_err(), "Expected timeout error when exceeding max connections, but got a connection");
        if let Err(e) = extra_conn {
            println!("Got expected timeout error: {:?}", e);
        }
    }

    // Clean up
    drop(test_pool); // Ensure all connections are dropped before dropping the database

    let mut conn = pool.get().expect("Failed to get connection");
    sql_query(format!("DROP DATABASE IF EXISTS {}", test_db_name))
        .execute(&mut conn)
        .expect("Failed to drop test database");

}