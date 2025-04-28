/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! This module provides functionality for creating and managing a PostgreSQL connection pool.
//!
//! It uses the diesel and r2d2 crates to manage database connections efficiently.

use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use url::Url;

/// Represents a pool of PostgreSQL database connections.
#[derive(Clone)]
pub struct ConnectionPool {
    /// The actual connection pool.
    pub pool: Pool<ConnectionManager<PgConnection>>,
}

/// Creates a shared connection pool for PostgreSQL databases.
///
/// This function constructs a URL from the provided base URL and database name,
/// then creates a connection pool with the specified maximum size.
///
/// # Arguments
///
/// * `base_url` - The base URL of the database server (e.g., "postgres://username:password@localhost:5432")
/// * `database_name` - The name of the database to connect to
/// * `max_size` - The maximum number of connections the pool should maintain
///
/// # Returns
///
/// Returns a `ConnectionPool` instance containing the created connection pool.
///
/// # Panics
///
/// This function will panic if:
/// * The base URL is invalid
/// * The connection pool creation fails
pub fn create_shared_connection_pool(
    base_url: &str,
    database_name: &str,
    max_size: u32,
) -> ConnectionPool {
    // Parse the base URL and set the database name
    let mut url = Url::parse(base_url).expect("Invalid base URL");
    url.set_path(database_name);

    // Create a connection manager
    let manager = ConnectionManager::<PgConnection>::new(url.as_str());

    // Build the connection pool
    let pool = Pool::builder()
        .max_size(max_size)
        .build(manager)
        .expect("Failed to create connection pool");

    ConnectionPool { pool }
}
