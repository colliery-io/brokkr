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
use once_cell::sync::Lazy;
use prometheus::{IntGauge, Opts};
use std::fs;
use std::path::Path;
use std::time::Duration;
use tokio::sync::oneshot;
use tracing::{info, warn};
use uuid::Uuid;
pub mod audit;
pub mod background_tasks;
pub mod config_watcher;
pub mod encryption;
pub mod event_bus;
pub mod matching;
pub mod pak;
pub mod templating;
pub mod ui_pak;
use brokkr_utils::config::{DEFAULT_ADMIN_PAK_HASH, Settings};

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

// =============================================================================
// Default admin PAK backstop (BROKKR-T-0298)
// =============================================================================

/// `1` when this broker's admin credential is the publicly-known development
/// PAK shipped in `default.toml`, `0` when it has been replaced.
///
/// This is the console/monitoring-visible half of the backstop. It is a gauge on
/// the existing `/metrics` endpoint rather than a new route or health field
/// because it costs one metric definition, needs no API surface or client
/// change, and is directly alertable (`brokkr_default_admin_pak_hash_in_use == 1`).
/// It is set exactly once, at startup, and never changes for the life of the
/// process — the admin hash is not hot-reloadable.
///
/// It is registered here rather than in `crate::metrics` so the definition sits
/// next to the detection logic it describes; the registry it publishes into is
/// the same one `/metrics` encodes.
static DEFAULT_ADMIN_PAK_HASH_IN_USE: Lazy<IntGauge> = Lazy::new(|| {
    let opts = Opts::new(
        "brokkr_default_admin_pak_hash_in_use",
        "1 if the broker's admin PAK hash is the publicly-known default shipped in default.toml, 0 otherwise",
    );
    let gauge = IntGauge::with_opts(opts).expect("Failed to create default admin PAK hash gauge");
    crate::metrics::REGISTRY
        .register(Box::new(gauge.clone()))
        .expect("Failed to register default admin PAK hash gauge");
    gauge
});

/// How often the startup warning is repeated while the default admin PAK is
/// still in use.
///
/// Hourly, deliberately. The startup banner alone scrolls out of a busy log
/// within minutes, so an operator who attaches to the logs of a broker that has
/// been up for a week would see nothing; repeating keeps the condition
/// discoverable at any point in the process's life. But a warning on a short
/// cycle is noise that trains people to filter the line out — the failure mode
/// the banner exists to avoid. Once an hour is ~24 lines a day: impossible to
/// miss when grepping, impossible to drown in. Continuous monitoring is the
/// gauge's job, not the log's.
const DEFAULT_ADMIN_PAK_REMINDER_INTERVAL: Duration = Duration::from_secs(3600);

/// Which admin credential sources still carry the shipped default hash.
///
/// The two are tracked separately because they have different causes and
/// different fixes. `configured` is the live config value; `stored` is what is
/// actually in `admin_role.pak_hash` — and those can disagree, because
/// `upsert_admin` only runs on first startup (or an explicit `rotate admin`).
/// An install that first booted with the default and *later* had
/// `BROKKR__BROKER__PAK_HASH` set keeps accepting the public PAK until the
/// admin row is rotated, which is the case a config-only check would miss.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DefaultAdminPakStatus {
    /// The configured `broker.pak_hash` is the shipped default.
    pub configured: bool,
    /// The admin hash stored in the database is the shipped default.
    pub stored: bool,
}

impl DefaultAdminPakStatus {
    /// True when the publicly-known PAK is accepted as admin by this broker.
    pub fn in_use(&self) -> bool {
        self.configured || self.stored
    }
}

/// Compares the effective admin credential against the shipped default.
///
/// Pure and exact: byte equality against [`DEFAULT_ADMIN_PAK_HASH`], no
/// heuristics, no I/O. `serve` binds a port and installs process-global state,
/// so the decision lives here where it can be tested directly.
///
/// # Arguments
/// * `configured` - `config.broker.pak_hash` after the full config layering.
/// * `stored` - `admin_role.pak_hash` as currently persisted, if the row exists.
pub fn detect_default_admin_pak_hash(
    configured: Option<&str>,
    stored: Option<&str>,
) -> DefaultAdminPakStatus {
    DefaultAdminPakStatus {
        configured: configured == Some(DEFAULT_ADMIN_PAK_HASH),
        stored: stored == Some(DEFAULT_ADMIN_PAK_HASH),
    }
}

/// Reads the persisted admin PAK hash, if the admin role has been provisioned.
///
/// Runs against whatever schema the connection's `search_path` selects, so on a
/// multi-tenant install it reports the tenant the broker is actually serving.
pub fn stored_admin_pak_hash(conn: &mut PgConnection) -> QueryResult<Option<String>> {
    admin_role::table
        .select(admin_role::pak_hash)
        .first::<String>(conn)
        .optional()
}

/// Renders the operator-facing warning for a default-PAK `status`.
///
/// Split out from the logging so the wording — specifically, that it always
/// carries a remediation — can be asserted in tests. Returns `None` when the
/// default is not in use.
fn default_admin_pak_warning(status: &DefaultAdminPakStatus) -> Option<String> {
    if !status.in_use() {
        return None;
    }

    // Every line is hard-wrapped to the banner width: the block only reads as a
    // banner if nothing in it wraps unpredictably in a terminal or log viewer.
    let what = match (status.configured, status.stored) {
        (true, _) => "WHAT: broker.pak_hash is set to the value shipped in Brokkr's default.toml.",
        (false, true) => {
            "WHAT: the admin PAK hash stored in the database is Brokkr's shipped default.\n\
             The configured hash was overridden, but the stored one was never rotated —\n\
             admin_role is written only on first startup or by an explicit rotation, so\n\
             the public PAK is still accepted."
        }
        (false, false) => unreachable!("in_use() is true"),
    };

    let fix = if status.stored && !status.configured {
        "FIX: run `brokkr-broker generate-pak`; set BROKKR__BROKER__PAK_HASH to the minted\n\
         hash (Helm: broker.pakHash, or broker.pakHashExistingSecret to source it from an\n\
         existing Secret); then run `brokkr-broker rotate admin` to write it to the\n\
         database. Restarting alone will NOT fix this."
    } else {
        "FIX: run `brokkr-broker generate-pak`; set BROKKR__BROKER__PAK_HASH to the minted\n\
         hash (Helm: broker.pakHash, or broker.pakHashExistingSecret to source it from an\n\
         existing Secret); restart. On an install that has already started once, also run\n\
         `brokkr-broker rotate admin` so the stored hash is replaced."
    };

    Some(format!(
        "\n\
         ==============================================================================\n\
         !! SECURITY: THIS BROKER ACCEPTS A PUBLICLY-KNOWN ADMIN CREDENTIAL !!\n\
         ==============================================================================\n\
         {what}\n\
         WHY IT MATTERS: the PAK matching that hash is published in Brokkr's public\n\
         source tree. Anyone who can reach this broker's API can authenticate as admin:\n\
         full read/write access to every agent, generator, stack and secret it serves.\n\
         {fix}\n\
         This is expected in local development and in the test harnesses, which rely on\n\
         the default. It must never be true in production.\n\
         =============================================================================="
    ))
}

/// Logs the default-admin-PAK banner (if applicable) and publishes
/// `brokkr_default_admin_pak_hash_in_use`.
///
/// Always sets the gauge, including to `0`, so the metric is present and
/// alertable on a correctly-configured broker rather than only appearing on a
/// broken one.
pub fn report_default_admin_pak_hash(status: &DefaultAdminPakStatus) {
    DEFAULT_ADMIN_PAK_HASH_IN_USE.set(i64::from(status.in_use()));

    match default_admin_pak_warning(status) {
        Some(banner) => warn!("{}", banner),
        None => info!("Admin PAK hash is not the shipped default"),
    }
}

/// Spawns the hourly re-warning task described on
/// [`DEFAULT_ADMIN_PAK_REMINDER_INTERVAL`]. No-op when the default is not in
/// use, so a correctly-configured broker carries no extra task.
///
/// Must be called from within a Tokio runtime.
pub fn start_default_admin_pak_reminder_task(status: DefaultAdminPakStatus) {
    let Some(banner) = default_admin_pak_warning(&status) else {
        return;
    };

    tokio::spawn(async move {
        loop {
            tokio::time::sleep(DEFAULT_ADMIN_PAK_REMINDER_INTERVAL).await;
            warn!("{}", banner);
        }
    });
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

    /// A hash that is neither the default nor equal to any other fixture.
    const OVERRIDE_HASH: &str = "1111111111111111111111111111111111111111111111111111111111111111";

    #[test]
    fn default_configured_admin_pak_hash_is_detected() {
        let status = detect_default_admin_pak_hash(
            Some(DEFAULT_ADMIN_PAK_HASH),
            Some(DEFAULT_ADMIN_PAK_HASH),
        );

        assert!(status.configured, "configured default must be flagged");
        assert!(status.stored, "stored default must be flagged");
        assert!(status.in_use());
    }

    #[test]
    fn overridden_admin_pak_hash_is_not_detected() {
        let status = detect_default_admin_pak_hash(Some(OVERRIDE_HASH), Some(OVERRIDE_HASH));

        assert!(!status.configured);
        assert!(!status.stored);
        assert!(!status.in_use(), "an overridden hash must not warn");
        assert!(
            default_admin_pak_warning(&status).is_none(),
            "no banner when the default has been replaced"
        );
    }

    /// The case a config-only check misses: the operator set
    /// `BROKKR__BROKER__PAK_HASH` on an install that first booted with the
    /// default, so `admin_role` still holds — and still accepts — the public
    /// hash.
    #[test]
    fn stale_stored_default_is_detected_despite_overridden_config() {
        let status =
            detect_default_admin_pak_hash(Some(OVERRIDE_HASH), Some(DEFAULT_ADMIN_PAK_HASH));

        assert!(!status.configured);
        assert!(status.stored);
        assert!(status.in_use());

        let banner = default_admin_pak_warning(&status).expect("stale stored default must warn");
        assert!(
            banner.contains("rotate admin"),
            "the stored-only case is not fixed by a restart, so the banner must \
             prescribe `rotate admin`: {banner}"
        );
        assert!(
            banner.contains("Restarting alone will NOT fix this"),
            "banner must say that a restart is insufficient: {banner}"
        );
    }

    /// An unset (or empty) hash means the broker mints its own PAK on first
    /// startup — not the public credential — so it must not trip the backstop.
    #[test]
    fn unset_admin_pak_hash_is_not_detected() {
        let status = detect_default_admin_pak_hash(None, None);

        assert!(!status.in_use());
        assert!(default_admin_pak_warning(&status).is_none());
    }

    /// "Unmissable" is a wording property, so pin it: the banner must say what
    /// is wrong, why it matters, and every supported way to fix it.
    #[test]
    fn default_admin_pak_warning_carries_full_remediation() {
        let status = detect_default_admin_pak_hash(
            Some(DEFAULT_ADMIN_PAK_HASH),
            Some(DEFAULT_ADMIN_PAK_HASH),
        );
        let banner = default_admin_pak_warning(&status).expect("default in use must warn");

        for needle in [
            "SECURITY",
            "PUBLICLY-KNOWN",
            "generate-pak",
            "BROKKR__BROKER__PAK_HASH",
            "broker.pakHash",
            "broker.pakHashExistingSecret",
        ] {
            assert!(
                banner.contains(needle),
                "banner must mention {needle:?}: {banner}"
            );
        }
        assert!(
            banner.lines().count() >= 8,
            "banner must be a multi-line block that is hard to scroll past: {banner}"
        );
    }

    /// The gauge must exist on `/metrics` in both states, so an alert on
    /// `brokkr_default_admin_pak_hash_in_use == 1` is meaningful rather than
    /// silently absent on a healthy broker.
    #[test]
    fn report_default_admin_pak_hash_publishes_gauge() {
        let clean = detect_default_admin_pak_hash(Some(OVERRIDE_HASH), Some(OVERRIDE_HASH));
        report_default_admin_pak_hash(&clean);

        let output = crate::metrics::encode_metrics();
        assert!(
            output.contains("brokkr_default_admin_pak_hash_in_use 0"),
            "gauge must be exported as 0 on a hardened broker: {output}"
        );

        let dirty = detect_default_admin_pak_hash(
            Some(DEFAULT_ADMIN_PAK_HASH),
            Some(DEFAULT_ADMIN_PAK_HASH),
        );
        report_default_admin_pak_hash(&dirty);

        let output = crate::metrics::encode_metrics();
        assert!(
            output.contains("brokkr_default_admin_pak_hash_in_use 1"),
            "gauge must be exported as 1 while the default is in use: {output}"
        );
    }
}
