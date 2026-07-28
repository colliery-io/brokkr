/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Integration tests for CLI schema handling (BROKKR-T-0297).
//!
//! CLI subcommands used to open a bare `PgConnection` that never applied
//! `BROKKR__DATABASE__SCHEMA`, so `rotate admin` silently operated on the
//! `public` schema on a schema-configured install. These tests pin the fixed
//! behavior: a CLI-path operation must land in the configured schema, and only
//! in the configured schema.
//!
//! The pattern here follows `db/multi_tenant.rs` (schemas provisioned with
//! `setup_schema` + migrations run through `ConnectionPool::get`), with one
//! difference: `connection_pool_from_settings` always targets the `brokkr`
//! database (as `serve` does), so these tests isolate on non-`public` schemas
//! inside that database rather than on a throwaway database.

use brokkr_broker::cli::commands::{connection_pool_from_settings, rotate_admin};
use brokkr_broker::db::create_shared_connection_pool;
use brokkr_utils::Settings;
use diesel::prelude::*;
use diesel::sql_query;
use diesel::sql_types::Text;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use uuid::Uuid;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("../brokkr-models/migrations");

/// Sentinel `admin_role.pak_hash` values. `upsert_admin` validates the
/// configured hash as 64 hex characters, so every value here must be a valid
/// SHA-256-shaped string and all three must be distinct.
const SENTINEL_A: &str = "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
const SENTINEL_B: &str = "bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb";
const ROTATED_HASH: &str = "cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc";

#[derive(QueryableByName)]
struct HashRow {
    #[diesel(sql_type = Text)]
    pak_hash: String,
}

#[derive(QueryableByName)]
struct SearchPathRow {
    #[diesel(sql_type = Text)]
    search_path: String,
}

/// Generates a collision-free schema name. Tests in this binary run in
/// parallel and may run concurrently with other suites against the same
/// database, so schema names must be unique per test invocation.
fn unique_schema(prefix: &str) -> String {
    format!("{}_{}", prefix, Uuid::new_v4().simple())
}

/// Creates `schema`, migrates the full schema into it, and seeds `admin_role`
/// with `sentinel` so a subsequent rotation is observable.
fn provision_schema(settings: &Settings, schema: &str, sentinel: &str) {
    let pool = create_shared_connection_pool(&settings.database.url, "brokkr", 2, Some(schema));
    pool.setup_schema(schema)
        .unwrap_or_else(|e| panic!("failed to set up schema {}: {}", schema, e));

    // `get()` sets `search_path TO <schema>, public`, so the migrations (and
    // diesel's own `__diesel_schema_migrations` bookkeeping table) are created
    // in `schema`, shadowing anything already migrated into `public`.
    let mut conn = pool.get().expect("failed to get connection");
    conn.run_pending_migrations(MIGRATIONS)
        .unwrap_or_else(|e| panic!("failed to migrate schema {}: {}", schema, e));

    sql_query(format!(
        "INSERT INTO admin_role (pak_hash) VALUES ('{}')",
        sentinel
    ))
    .execute(&mut conn)
    .unwrap_or_else(|e| panic!("failed to seed admin_role in {}: {}", schema, e));
}

/// Reads the single `admin_role.pak_hash` from `schema`.
fn admin_hash_in_schema(settings: &Settings, schema: &str) -> String {
    let pool = create_shared_connection_pool(&settings.database.url, "brokkr", 1, Some(schema));
    let mut conn = pool.get().expect("failed to get connection");
    let row: HashRow = sql_query("SELECT pak_hash FROM admin_role LIMIT 1")
        .get_result(&mut conn)
        .unwrap_or_else(|e| panic!("failed to read admin_role in {}: {}", schema, e));
    row.pak_hash
}

/// Reads `public`'s `admin_role.pak_hash`, if there is one. Returns `None` when
/// `public` has no row — or no `admin_role` table at all, which is the case if
/// this test runs before anything migrates `public`.
fn admin_hash_in_public(settings: &Settings) -> Option<String> {
    let pool = create_shared_connection_pool(&settings.database.url, "brokkr", 1, None);
    let mut conn = pool.get().expect("failed to get connection");
    sql_query("SELECT pak_hash FROM admin_role LIMIT 1")
        .get_result::<HashRow>(&mut conn)
        .ok()
        .map(|row| row.pak_hash)
}

fn drop_schema(settings: &Settings, schema: &str) {
    let pool = create_shared_connection_pool(&settings.database.url, "brokkr", 1, None);
    let mut conn = pool.get().expect("failed to get connection");
    sql_query(format!("DROP SCHEMA IF EXISTS {} CASCADE", schema))
        .execute(&mut conn)
        .ok();
}

/// `rotate admin` must write to the schema named by `database.schema` and to no
/// other schema.
///
/// This is the regression test for BROKKR-T-0297: before the fix, `rotate_admin`
/// established a bare connection with no `search_path`, so the rotation landed
/// on `public` — leaving the configured schema's `admin_role` untouched and
/// rewriting `public`'s. Both halves are asserted, plus a second migrated
/// schema to prove the write does not leak sideways.
#[test]
fn test_rotate_admin_writes_to_configured_schema() {
    let base = Settings::new(None).expect("Failed to load settings");
    let schema_a = unique_schema("cli_rotate_a");
    let schema_b = unique_schema("cli_rotate_b");

    provision_schema(&base, &schema_a, SENTINEL_A);
    provision_schema(&base, &schema_b, SENTINEL_B);

    // Snapshot `public` so we can prove the rotation never touched it.
    let public_before = admin_hash_in_public(&base);

    // Rotate with schema_a configured. Pinning `broker.pak_hash` makes
    // `upsert_admin` take its configured-hash branch, so the resulting value is
    // deterministic and no bootstrap key file is written.
    let mut settings = base.clone();
    settings.database.schema = Some(schema_a.clone());
    settings.broker.pak_hash = Some(ROTATED_HASH.to_string());

    let result = rotate_admin(&settings);

    let hash_a = admin_hash_in_schema(&base, &schema_a);
    let hash_b = admin_hash_in_schema(&base, &schema_b);
    let public_after = admin_hash_in_public(&base);

    drop_schema(&base, &schema_a);
    drop_schema(&base, &schema_b);

    result.expect("rotate_admin should succeed against the configured schema");

    assert_eq!(
        hash_a, ROTATED_HASH,
        "rotate admin must rotate admin_role in the configured schema ({})",
        schema_a
    );
    assert_eq!(
        hash_b, SENTINEL_B,
        "rotate admin must not touch admin_role in another schema ({})",
        schema_b
    );
    assert_eq!(
        public_after, public_before,
        "rotate admin must not touch admin_role in the public schema"
    );
}

/// The shared helper every CLI subcommand now uses must carry
/// `database.schema` all the way onto the connection the caller ends up with —
/// the search_path is applied per checkout, not once at pool construction.
#[test]
fn test_connection_pool_from_settings_applies_configured_schema() {
    let base = Settings::new(None).expect("Failed to load settings");
    let schema = unique_schema("cli_pool");

    let bootstrap = create_shared_connection_pool(&base.database.url, "brokkr", 1, Some(&schema));
    bootstrap
        .setup_schema(&schema)
        .unwrap_or_else(|e| panic!("failed to set up schema {}: {}", schema, e));
    drop(bootstrap);

    // With a schema configured, every connection from the pool is scoped to it.
    let mut with_schema = base.clone();
    with_schema.database.schema = Some(schema.clone());
    let scoped_pool = connection_pool_from_settings(&with_schema, 1);
    let scoped_path: SearchPathRow = {
        let mut conn = scoped_pool.get().expect("failed to get connection");
        sql_query("SHOW search_path")
            .get_result(&mut conn)
            .expect("failed to read search_path")
    };

    // With no schema configured, the pool must not force one (public behavior
    // is preserved for existing deployments).
    let mut without_schema = base.clone();
    without_schema.database.schema = None;
    let plain_pool = connection_pool_from_settings(&without_schema, 1);
    let plain_path: SearchPathRow = {
        let mut conn = plain_pool.get().expect("failed to get connection");
        sql_query("SHOW search_path")
            .get_result(&mut conn)
            .expect("failed to read search_path")
    };

    drop(scoped_pool);
    drop(plain_pool);
    drop_schema(&base, &schema);

    assert!(
        scoped_path.search_path.starts_with(&schema),
        "configured schema must lead the search_path, got '{}'",
        scoped_path.search_path
    );
    assert!(
        !plain_path.search_path.contains(&schema),
        "an unconfigured pool must not be scoped to '{}', got '{}'",
        schema,
        plain_path.search_path
    );
}
