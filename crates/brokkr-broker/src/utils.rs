use std::fs;
use tokio::sync::oneshot;
use brokkr_utils::config::Settings;
use prefixed_api_key::PrefixedApiKeyController;
use diesel::prelude::*;
use brokkr_models::schema::admin_role;
use uuid::Uuid;
use chrono::Utc;



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

pub fn first_startup(config: &Settings, conn: &mut PgConnection) -> Result<(), Box<dyn std::error::Error>> {
    upsert_admin(config, conn)
}

fn upsert_admin(config: &Settings, conn: &mut PgConnection) -> Result<(), Box<dyn std::error::Error>> {
    // Configure PAK controller
    let mut builder = PrefixedApiKeyController::configure()
        .prefix(config.pak.prefix.clone().unwrap())
        .rng_osrng()
        .short_token_length(config.pak.short_token_length.unwrap())
        .short_token_prefix(config.pak.short_token_prefix.clone())
        .long_token_length(config.pak.long_token_length.unwrap());
    
    let rng = config.pak.rng.clone().unwrap();

    builder = match rng.as_str() {
        "osrng" => builder.rng_osrng(),
        "sha256" => builder.digest_sha256(),
        _ => panic!("Invalid RNG type"),
    };

    let controller = builder.finalize().expect("failed to create pak controller");

    // Generate PAK and hash
    let (pak, hash) = controller.try_generate_key_and_hash()?;

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
                .values(&NewAdminKey { pak_hash: hash.clone() })
                .execute(conn)?;
        }
    }
    // Write PAK to /tmp/key.txt
    fs::write("/tmp/key.txt", pak.to_string())?;

    Ok(())
}
