/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Database connection pool management using diesel and r2d2.
//!
//! For detailed documentation, see the [Brokkr Documentation](https://brokkr.io/explanation/components#database-module).

use diesel::r2d2::{ConnectionManager, Pool};
use diesel::PgConnection;
use url::Url;

/// Represents a pool of PostgreSQL database connections.
#[derive(Clone)]
pub struct ConnectionPool {
    /// The actual connection pool.
    pub pool: Pool<ConnectionManager<PgConnection>>,
    /// Optional schema name for multi-tenant deployments.
    pub schema: Option<String>,
}

/// Creates a shared connection pool for PostgreSQL databases.
///
/// # Arguments
///
/// * `base_url` - The base URL of the database server (e.g., "postgres://username:password@localhost:5432")
/// * `database_name` - The name of the database to connect to
/// * `max_size` - The maximum number of connections the pool should maintain
/// * `schema` - Optional schema name for multi-tenant isolation
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
    schema: Option<&str>,
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

    ConnectionPool {
        pool,
        schema: schema.map(String::from),
    }
}

/// Validates a PostgreSQL schema name to prevent SQL injection.
///
/// Schema names must start with a letter and contain only alphanumeric characters and underscores.
///
/// # Arguments
///
/// * `schema` - The schema name to validate
///
/// # Returns
///
/// Returns `Ok(())` if valid, or an error message if invalid.
pub fn validate_schema_name(schema: &str) -> Result<(), String> {
    if schema.is_empty() {
        return Err("Schema name cannot be empty".to_string());
    }

    // Check first character is a letter
    if !schema.chars().next().unwrap().is_ascii_alphabetic() {
        return Err("Schema name must start with a letter".to_string());
    }

    // Check all characters are alphanumeric or underscore
    if !schema
        .chars()
        .all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err("Schema name can only contain letters, numbers, and underscores".to_string());
    }

    Ok(())
}

impl ConnectionPool {
    /// Gets a connection from the pool with automatic schema search_path configuration.
    ///
    /// If a schema is configured, this method automatically executes `SET search_path` on the
    /// connection to ensure all queries execute in the correct schema context.
    ///
    /// # Returns
    ///
    /// Returns a pooled connection ready for use.
    ///
    /// # Panics
    ///
    /// This method will panic if:
    /// * Unable to get a connection from the pool
    /// * The schema name is invalid
    /// * Failed to set the search path
    pub fn get(
        &self,
    ) -> Result<diesel::r2d2::PooledConnection<ConnectionManager<PgConnection>>, r2d2::Error> {
        use diesel::prelude::*;

        let mut conn = self.pool.get()?;

        if let Some(ref schema) = self.schema {
            // Validate schema name to prevent SQL injection
            validate_schema_name(schema).expect("Invalid schema name");

            // Set search_path for this connection
            let sql = format!("SET search_path TO {}, public", schema);
            diesel::sql_query(&sql)
                .execute(&mut conn)
                .expect("Failed to set search_path");
        }

        Ok(conn)
    }

    /// Sets up a PostgreSQL schema for multi-tenant isolation.
    ///
    /// This method creates the schema if it doesn't exist and prepares it for migrations.
    /// It should be called during application startup before running migrations.
    ///
    /// # Arguments
    ///
    /// * `schema` - The schema name to set up
    ///
    /// # Returns
    ///
    /// Returns `Ok(())` on success, or an error if schema setup fails.
    pub fn setup_schema(&self, schema: &str) -> Result<(), String> {
        use diesel::prelude::*;

        // Validate schema name
        validate_schema_name(schema).map_err(|e| format!("Invalid schema name: {}", e))?;

        let mut conn = self
            .pool
            .get()
            .map_err(|e| format!("Failed to get connection: {}", e))?;

        // Create schema if it doesn't exist
        let create_schema_sql = format!("CREATE SCHEMA IF NOT EXISTS {}", schema);
        diesel::sql_query(&create_schema_sql)
            .execute(&mut conn)
            .map_err(|e| format!("Failed to create schema '{}': {}", schema, e))?;

        // Set search_path for subsequent operations
        let set_search_path_sql = format!("SET search_path TO {}, public", schema);
        diesel::sql_query(&set_search_path_sql)
            .execute(&mut conn)
            .map_err(|e| format!("Failed to set search path: {}", e))?;

        Ok(())
    }
}
