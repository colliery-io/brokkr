/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! # Brokkr-Models
//!
//! This module provides functionality for establishing a connection to a PostgreSQL database
//! and declares the models and schema modules used in the application.

use diesel::pg::PgConnection;
use diesel::prelude::*;

/// Declares the models module, which likely contains the data structures representing database tables.
pub mod models;

/// Declares the schema module, which likely contains the database schema definitions.
pub mod schema;

#[allow(dead_code)]
/// Establishes a connection to the PostgreSQL database.
///
/// This function exists to manage migrations and perform basic testing in this crate
/// without a specific Data Access Layer (DAL) in place.
///
/// # Arguments
///
/// * `database_url` - A string slice that holds the URL of the database to connect to.
///
/// # Returns
///
/// * `PgConnection` - A connection to the PostgreSQL database.
///
/// # Panics
///
/// This function will panic if it fails to establish a connection to the database.
///
pub(crate) fn establish_connection(database_url: String) -> PgConnection {
    PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url))
}
