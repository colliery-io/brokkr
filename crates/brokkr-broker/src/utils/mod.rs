use brokkr_models::schema::admin_role;
use chrono::Utc;
use diesel::prelude::*;
use std::fs;
use tokio::sync::oneshot;
use uuid::Uuid;

pub mod pak;

pub async fn shutdown(shutdown_rx: oneshot::Receiver<()>) {
    let _ = shutdown_rx.await;

    // Attempt to remove the file at /tmp/key.txt
    let _ = fs::remove_file("/tmp/key.txt");
}

#[derive(Queryable, Selectable, Identifiable, AsChangeset, Debug, Clone)]
#[diesel(table_name = admin_role)]
pub struct AdminKey {
    pub id: Uuid,
    pub created_at: chrono::DateTime<Utc>,
    pub updated_at: chrono::DateTime<Utc>,
    pub pak_hash: String,
}

#[derive(Insertable)]
#[diesel(table_name = admin_role)]
pub struct NewAdminKey {
    pub pak_hash: String,
}

pub fn first_startup(conn: &mut PgConnection) -> Result<(), Box<dyn std::error::Error>> {
    upsert_admin(conn)
}

fn create_pak() -> Result<(String, String), Box<dyn std::error::Error>> {
    // Configure PAK controller
    let controller = pak::create_pak_controller(None);

    // Generate PAK and hash
    controller
        .unwrap()
        .try_generate_key_and_hash()
        .map(|(pak, hash)| (pak.to_string(), hash))
        .map_err(|e| e.into())
}

pub fn upsert_admin(conn: &mut PgConnection) -> Result<(), Box<dyn std::error::Error>> {
    let (pak, hash) = create_pak()?;

    // Update the existing admin role with the new PAK hash, or insert if it doesn't exist
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
    // Write PAK to /tmp/key.txt
    fs::write("/tmp/key.txt", pak)?;

    Ok(())
}
