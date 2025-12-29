/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Utility functions and structures for the Brokkr broker.
//!
//! This module contains various helper functions and structures used throughout
//! the broker, including admin key management and shutdown procedures.

use brokkr_models::schema::admin_role;
use brokkr_utils::logging::prelude::*;
use chrono::Utc;
use diesel::prelude::*;
use std::fs;
use std::path::Path;
use tokio::sync::oneshot;
use uuid::Uuid;
pub mod background_tasks;
pub mod encryption;
pub mod event_bus;
pub mod matching;
pub mod pak;
pub mod templating;
use brokkr_utils::config::Settings;

/// Handles the shutdown process for the broker.
///
/// This function waits for a shutdown signal and then performs cleanup tasks.
pub async fn shutdown(shutdown_rx: oneshot::Receiver<()>) {
    let _ = shutdown_rx.await;
    // Remove the temporary key file
    let _ = fs::remove_file("/tmp/key.txt");
}

/// Represents an admin key in the database.
#[derive(Queryable, Selectable, Identifiable, AsChangeset, Debug, Clone)]
#[diesel(table_name = admin_role)]
pub struct AdminKey {
    pub id: Uuid,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
    pub pak_hash: String,
}

/// Represents a new admin key to be inserted into the database.
#[derive(Insertable)]
#[diesel(table_name = admin_role)]
pub struct NewAdminKey {
    pub pak_hash: String,
}

/// Performs first-time startup operations.
///
/// This function is called when the broker starts for the first time and
/// sets up the initial admin key.
pub fn first_startup(
    conn: &mut PgConnection,
    config: &Settings,
) -> Result<(), Box<dyn std::error::Error>> {
    upsert_admin(conn, config)
}

/// Creates a new PAK (Privileged Access Key) and its hash.
///
/// This function generates a new PAK and returns both the key and its hash.
fn create_pak() -> Result<(String, String), Box<dyn std::error::Error>> {
    // Generate PAK and hash using the PAK controller
    let controller = pak::create_pak_controller(None);
    controller
        .unwrap()
        .try_generate_key_and_hash()
        .map(|(pak, hash)| (pak.to_string(), hash))
        .map_err(|e| e.into())
}

/// Updates or inserts the admin key and related generator.
///
/// This function creates or updates the admin key in the database,
/// creates or updates the associated admin generator, and writes
/// the PAK to a temporary file.
pub fn upsert_admin(
    conn: &mut PgConnection,
    config: &Settings,
) -> Result<(), Box<dyn std::error::Error>> {
    let pak_hash = match &config.broker.pak_hash {
        Some(hash) if !hash.is_empty() => {
            // Validate the provided hash
            if !validate_pak_hash(hash) {
                return Err("Invalid PAK hash provided in configuration".into());
            }
            hash.clone()
        }
        _ => {
            // Generate new PAK and hash
            let (pak, hash) = create_pak()?;

            // Write PAK to temporary file
            info!("Writing PAK to temporary file");
            let key_path = Path::new("/tmp/brokkr-keys/key.txt");
            fs::create_dir_all(key_path.parent().unwrap())?;
            fs::write(key_path, pak)?;

            hash
        }
    };

    // Update or insert admin key
    let existing_admin_key = admin_role::table
        .select(admin_role::id)
        .first::<Uuid>(conn)
        .optional()?;

    match existing_admin_key {
        Some(id) => {
            diesel::update(admin_role::table.find(id))
                .set(admin_role::pak_hash.eq(&pak_hash))
                .execute(conn)?;
        }
        None => {
            diesel::insert_into(admin_role::table)
                .values(&NewAdminKey {
                    pak_hash: pak_hash.clone(),
                })
                .execute(conn)?;
        }
    }

    // Update or insert admin generator
    use brokkr_models::schema::generators;
    let existing_admin_generator = generators::table
        .filter(generators::name.eq("admin-generator"))
        .select(generators::id)
        .first::<Uuid>(conn)
        .optional()?;

    match existing_admin_generator {
        Some(id) => {
            diesel::update(generators::table.find(id))
                .set((
                    generators::pak_hash.eq(&pak_hash),
                    generators::description.eq("Linked to Admin PAK"),
                ))
                .execute(conn)?;
        }
        None => {
            diesel::insert_into(generators::table)
                .values((
                    generators::name.eq("admin-generator"),
                    generators::description.eq("Linked to Admin PAK"),
                    generators::pak_hash.eq(&pak_hash),
                ))
                .execute(conn)?;
        }
    }

    Ok(())
}

fn validate_pak_hash(hash: &str) -> bool {
    // Implement hash validation logic here
    // For example, check if it's a valid SHA-256 hash
    hash.len() == 64 && hash.chars().all(|c| c.is_ascii_hexdigit())
}
