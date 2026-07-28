/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Integration tests for the default-admin-PAK startup backstop (BROKKR-T-0298).
//!
//! `crates/brokkr-utils/default.toml` ships an admin PAK hash whose raw PAK is
//! published in the same file, so a broker that never overrides it accepts a
//! credential anyone can read out of the repository. The broker warns rather
//! than refusing (see the DECISION on BROKKR-T-0298), which means the detection
//! itself is the whole guarantee — these tests exercise it against a real
//! database, through the same `upsert_admin` write path `serve` uses.
//!
//! `serve` itself is not testable here: it binds :3000 and installs
//! process-global state (encryption key, audit logger, UI PAK). So the tests
//! drive the two halves `serve` composes — `upsert_admin` to establish the
//! stored credential, then `stored_admin_pak_hash` + `detect_default_admin_pak_hash`
//! to make the decision. The wording of the warning and the `/metrics` gauge are
//! unit-tested in `brokkr_broker::utils`.
//!
//! Schema provisioning follows `db/cli_schema.rs`: each test migrates a
//! throwaway schema inside the `brokkr` database and drops it afterwards.

use brokkr_broker::db::create_shared_connection_pool;
use brokkr_broker::utils::{detect_default_admin_pak_hash, stored_admin_pak_hash, upsert_admin};
use brokkr_utils::Settings;
use brokkr_utils::config::DEFAULT_ADMIN_PAK_HASH;
use diesel::prelude::*;
use diesel::sql_query;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use uuid::Uuid;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("../brokkr-models/migrations");

/// A valid-shaped SHA-256 hash that is not the shipped default — what an
/// operator who followed the hardening guide would have configured.
const OVERRIDE_HASH: &str = "dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd";

fn unique_schema(prefix: &str) -> String {
    format!("{}_{}", prefix, Uuid::new_v4().simple())
}

/// Creates `schema` and migrates the full schema into it. `admin_role` is left
/// empty so the caller decides how the admin credential comes to exist.
fn provision_schema(settings: &Settings, schema: &str) {
    let pool = create_shared_connection_pool(&settings.database.url, "brokkr", 2, Some(schema));
    pool.setup_schema(schema)
        .unwrap_or_else(|e| panic!("failed to set up schema {}: {}", schema, e));

    let mut conn = pool.get().expect("failed to get connection");
    conn.run_pending_migrations(MIGRATIONS)
        .unwrap_or_else(|e| panic!("failed to migrate schema {}: {}", schema, e));
}

fn drop_schema(settings: &Settings, schema: &str) {
    let pool = create_shared_connection_pool(&settings.database.url, "brokkr", 1, None);
    let mut conn = pool.get().expect("failed to get connection");
    sql_query(format!("DROP SCHEMA IF EXISTS {} CASCADE", schema))
        .execute(&mut conn)
        .ok();
}

/// Settings scoped to `schema` with `broker.pak_hash` pinned to `hash`.
fn settings_for(base: &Settings, schema: &str, hash: &str) -> Settings {
    let mut settings = base.clone();
    settings.database.schema = Some(schema.to_string());
    settings.broker.pak_hash = Some(hash.to_string());
    settings
}

/// The zero-configuration path: `angreal local up`, the docker-compose harness
/// and the e2e Helm values all boot the broker with `default.toml`'s
/// `broker.pak_hash`. Startup must recognize the shipped default in both the
/// configured value and the row `upsert_admin` writes from it.
#[test]
fn test_default_admin_pak_hash_is_detected_after_first_startup() {
    let base = Settings::new(None).expect("Failed to load settings");
    let schema = unique_schema("default_pak_shipped");
    provision_schema(&base, &schema);

    let settings = settings_for(&base, &schema, DEFAULT_ADMIN_PAK_HASH);

    let pool = create_shared_connection_pool(&base.database.url, "brokkr", 1, Some(&schema));
    let mut conn = pool.get().expect("failed to get connection");

    // Same call `serve` makes on first startup.
    let upsert = upsert_admin(&mut conn, &settings);
    let stored = stored_admin_pak_hash(&mut conn);

    drop(conn);
    drop(pool);
    drop_schema(&base, &schema);

    upsert.expect("upsert_admin must accept the shipped default hash");
    let stored = stored.expect("reading admin_role must succeed");

    assert_eq!(
        stored.as_deref(),
        Some(DEFAULT_ADMIN_PAK_HASH),
        "first startup with no override must persist the shipped default hash"
    );

    let status =
        detect_default_admin_pak_hash(settings.broker.pak_hash.as_deref(), stored.as_deref());
    assert!(
        status.configured,
        "the configured hash is the shipped default and must be flagged"
    );
    assert!(
        status.stored,
        "the persisted hash is the shipped default and must be flagged"
    );
    assert!(status.in_use(), "startup must warn on this install");
}

/// The hardened path: an operator who ran `generate-pak` and set
/// `BROKKR__BROKER__PAK_HASH` before first startup must get a clean broker —
/// no warning, and `brokkr_default_admin_pak_hash_in_use` at 0.
#[test]
fn test_overridden_admin_pak_hash_is_not_detected_after_first_startup() {
    let base = Settings::new(None).expect("Failed to load settings");
    let schema = unique_schema("default_pak_override");
    provision_schema(&base, &schema);

    let settings = settings_for(&base, &schema, OVERRIDE_HASH);

    let pool = create_shared_connection_pool(&base.database.url, "brokkr", 1, Some(&schema));
    let mut conn = pool.get().expect("failed to get connection");

    let upsert = upsert_admin(&mut conn, &settings);
    let stored = stored_admin_pak_hash(&mut conn);

    drop(conn);
    drop(pool);
    drop_schema(&base, &schema);

    upsert.expect("upsert_admin must accept an overridden hash");
    let stored = stored.expect("reading admin_role must succeed");

    assert_eq!(
        stored.as_deref(),
        Some(OVERRIDE_HASH),
        "first startup must persist the configured override"
    );

    let status =
        detect_default_admin_pak_hash(settings.broker.pak_hash.as_deref(), stored.as_deref());
    assert!(
        !status.in_use(),
        "a broker configured per the hardening guide must not warn"
    );
}

/// The case a config-only check would miss, and the reason the stored hash is
/// read at all: an install that first booted with the default and *later* had
/// `BROKKR__BROKER__PAK_HASH` set. `upsert_admin` does not run again on
/// subsequent startups, so `admin_role` still holds the public hash and the
/// public PAK still authenticates. Detection must fire on the stored value even
/// though the configuration looks correct.
#[test]
fn test_stale_stored_default_is_detected_when_config_was_corrected_later() {
    let base = Settings::new(None).expect("Failed to load settings");
    let schema = unique_schema("default_pak_stale");
    provision_schema(&base, &schema);

    let pool = create_shared_connection_pool(&base.database.url, "brokkr", 1, Some(&schema));
    let mut conn = pool.get().expect("failed to get connection");

    // First startup ran with the default and wrote it to admin_role.
    let first = upsert_admin(
        &mut conn,
        &settings_for(&base, &schema, DEFAULT_ADMIN_PAK_HASH),
    );

    // A later restart has the override configured — but `upsert_admin` is a
    // first-run-only call, so nothing rewrites the row.
    let corrected = settings_for(&base, &schema, OVERRIDE_HASH);
    let stored = stored_admin_pak_hash(&mut conn);

    drop(conn);
    drop(pool);
    drop_schema(&base, &schema);

    first.expect("first startup must succeed");
    let stored = stored.expect("reading admin_role must succeed");

    let status =
        detect_default_admin_pak_hash(corrected.broker.pak_hash.as_deref(), stored.as_deref());
    assert!(
        !status.configured,
        "the configuration was corrected, so the configured half must be clean"
    );
    assert!(
        status.stored,
        "the un-rotated stored hash is still the public default and must be flagged"
    );
    assert!(
        status.in_use(),
        "the public PAK still authenticates here; startup must warn"
    );
}
