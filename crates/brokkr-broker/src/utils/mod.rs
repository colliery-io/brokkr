/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Utility functions and structures for the Brokkr broker.
//!
//! This module contains various helper functions and structures used throughout
//! the broker, including admin key management and shutdown procedures.

use brokkr_models::schema::admin_role;
use chrono::Utc;
use diesel::prelude::*;
use std::fs;
use std::path::Path;
use tokio::sync::oneshot;
use tracing::info;
use uuid::Uuid;
pub mod audit;
pub mod background_tasks;
pub mod config_watcher;
pub mod encryption;
pub mod event_bus;
pub mod matching;
pub mod pak;
pub mod templating;
use brokkr_utils::config::Settings;

/// Path of the bootstrap key file written when the broker generates an admin
/// PAK itself (i.e. no `pak_hash` was configured). Defined once so the write in
/// `upsert_admin` and the cleanup in `shutdown` can never drift apart.
const BOOTSTRAP_KEY_FILE: &str = "/tmp/brokkr-keys/key.txt";

/// Handles the shutdown process for the broker.
///
/// This function waits for a shutdown signal and then performs cleanup tasks.
pub async fn shutdown(shutdown_rx: oneshot::Receiver<()>) {
    let _ = shutdown_rx.await;
    // Remove the bootstrap key file dropped by `upsert_admin` on first startup.
    let _ = fs::remove_file(BOOTSTRAP_KEY_FILE);
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
            let key_path = Path::new(BOOTSTRAP_KEY_FILE);
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

#[cfg(test)]
mod tests {
    use super::*;

    /// The offline `generate-pak` day-zero flow only works if the hash it mints
    /// is accepted by `upsert_admin`'s configured-hash branch. Guard that
    /// contract directly: a freshly minted hash must satisfy `validate_pak_hash`
    /// so an operator can feed it back via `BROKKR__BROKER__PAK_HASH`.
    #[test]
    fn minted_hash_passes_config_validation() {
        let config = Settings::new(None).expect("Failed to load configuration");
        pak::create_pak_controller(Some(&config)).expect("Failed to init PAK controller");

        let (_pak, hash) = pak::create_pak().expect("Failed to mint PAK");

        assert!(
            validate_pak_hash(&hash),
            "minted hash {hash:?} must satisfy validate_pak_hash so the \
             BROKKR__BROKER__PAK_HASH bootstrap path accepts it"
        );
    }
}
