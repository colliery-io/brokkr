//! Utility functions and structures for the Brokkr broker.
//!
//! This module contains various helper functions and structures used throughout
//! the broker, including admin key management and shutdown procedures.

use brokkr_models::schema::admin_role;
use chrono::Utc;
use diesel::prelude::*;
use std::fs;
use tokio::sync::oneshot;
use uuid::Uuid;

pub mod pak;

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
pub fn first_startup(conn: &mut PgConnection) -> Result<(), Box<dyn std::error::Error>> {
    upsert_admin(conn)
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
pub fn upsert_admin(conn: &mut PgConnection) -> Result<(), Box<dyn std::error::Error>> {
    let (pak, hash) = create_pak()?;

    // Update or insert admin key
    let existing_admin_key = admin_role::table
        .select(admin_role::id)
        .first::<Uuid>(conn)
        .optional()?;

    match existing_admin_key {
        Some(id) => {
            // Update existing admin key
            diesel::update(admin_role::table.find(id))
                .set(admin_role::pak_hash.eq(hash.clone()))
                .execute(conn)?;
        }
        None => {
            // Insert new admin key
            diesel::insert_into(admin_role::table)
                .values(&NewAdminKey {
                    pak_hash: hash.clone(),
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
            // Update existing Admin Generator
            diesel::update(generators::table.find(id))
                .set((
                    generators::pak_hash.eq(hash.clone()),
                    generators::description.eq("Linked to Admin PAK"),
                ))
                .execute(conn)?;
        }
        None => {
            // Insert new Admin Generator
            diesel::insert_into(generators::table)
                .values((
                    generators::name.eq("admin-generator"),
                    generators::description.eq("Linked to Admin PAK"),
                    generators::pak_hash.eq(hash.clone()),
                ))
                .execute(conn)?;
        }
    }

    // Write PAK to temporary file
    fs::write("/tmp/key.txt", pak)?;

    Ok(())
}
