/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Each database path of the broker must use the configured schema
//! (BROKKR-T-0327, ADR BROKKR-A-0004).
//!
//! Each test uses a new database. In it, `public` and two schemas each hold a
//! full set of Brokkr tables with the same names. A write through one path must
//! land in its schema only, and the other schemas must not see it. Because
//! `public` is migrated too, a path that falls back to `public` fails here; in
//! a fresh database it would fail with "relation does not exist" and hide the
//! fault.

use brokkr_broker::cli::commands::{
    create_agent, create_generator, rotate_agent_key, rotate_generator_key,
};
use brokkr_broker::dal::DAL;
use brokkr_broker::db::{ConnectionPool, create_shared_connection_pool};
use brokkr_broker::utils;
use brokkr_models::models::audit_logs::{
    ACTION_PAK_ROTATED, ACTOR_TYPE_ADMIN, NewAuditLog, RESOURCE_TYPE_GENERATOR,
};
use brokkr_utils::Settings;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::sql_query;
use diesel::sql_types::Bool;
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};
use url::Url;
use uuid::Uuid;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("../brokkr-models/migrations");

const SCHEMA_A: &str = "tenant_a";
const SCHEMA_B: &str = "tenant_b";

#[derive(QueryableByName)]
struct Present {
    #[diesel(sql_type = Bool)]
    present: bool,
}

/// A new database for one test. It is dropped on `Drop`, also when an
/// assertion fails.
struct TestDb {
    server_url: String,
    name: String,
}

impl TestDb {
    fn new() -> Self {
        let settings = Settings::new(None).expect("Failed to load settings");
        let mut url = Url::parse(&settings.database.url).expect("Invalid base URL");
        url.set_path("");
        let server_url = url.to_string();
        let name = format!("schema_paths_{}", Uuid::new_v4().simple());

        let mut conn = server_conn(&server_url);
        sql_query(format!("CREATE DATABASE {}", name))
            .execute(&mut conn)
            .expect("Failed to create test database");

        TestDb { server_url, name }
    }

    /// The URL of this database, as an operator would set `database.url`.
    fn url(&self) -> String {
        let mut url = Url::parse(&self.server_url).expect("Invalid base URL");
        url.set_path(&self.name);
        url.to_string()
    }

    fn pool(&self, schema: Option<&str>) -> ConnectionPool {
        create_shared_connection_pool(&self.server_url, Some(&self.name), 3, schema)
    }

    fn dal(&self, schema: Option<&str>) -> DAL {
        DAL::new(self.pool(schema))
    }

    /// Creates `schema` (when given) and migrates it, as `serve` does.
    fn migrate(&self, schema: Option<&str>) {
        let pool = self.pool(schema);
        if let Some(schema) = schema {
            pool.setup_schema(schema).expect("Failed to set up schema");
        }
        let mut conn = pool.get().expect("Failed to get connection");
        conn.run_pending_migrations(MIGRATIONS)
            .expect("Failed to run migrations");
    }

    /// Settings for a CLI command against this database and `schema`.
    fn settings(&self, schema: Option<&str>) -> Settings {
        let mut settings = Settings::new(None).expect("Failed to load settings");
        settings.database.url = self.url();
        settings.database.schema = schema.map(String::from);
        utils::pak::create_pak_controller(Some(&settings))
            .expect("Failed to create PAK controller");
        settings
    }

    /// True when the table `qualified` (for example `tenant_a.admin_role`)
    /// exists. The name is qualified, so the `public` fallback cannot answer.
    fn table_exists(&self, qualified: &str) -> bool {
        let mut conn = self.pool(None).get().expect("Failed to get connection");
        let row: Present = sql_query(format!(
            "SELECT to_regclass('{}') IS NOT NULL AS present",
            qualified
        ))
        .get_result(&mut conn)
        .expect("Failed to check table");
        row.present
    }
}

impl Drop for TestDb {
    fn drop(&mut self) {
        let mut conn = server_conn(&self.server_url);
        sql_query(format!(
            "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '{}'",
            self.name
        ))
        .execute(&mut conn)
        .ok();
        sql_query(format!("DROP DATABASE IF EXISTS {}", self.name))
            .execute(&mut conn)
            .ok();
    }
}

fn server_conn(
    server_url: &str,
) -> diesel::r2d2::PooledConnection<ConnectionManager<PgConnection>> {
    Pool::builder()
        .max_size(1)
        .build(ConnectionManager::<PgConnection>::new(server_url))
        .expect("Failed to create pool")
        .get()
        .expect("Failed to get connection")
}

/// A database with `public`, `tenant_a` and `tenant_b` all migrated.
fn three_schema_db() -> TestDb {
    let db = TestDb::new();
    db.migrate(None);
    db.migrate(Some(SCHEMA_A));
    db.migrate(Some(SCHEMA_B));
    db
}

/// Startup path: `setup_schema` and the migrations through
/// `ConnectionPool::get` create the tables in the configured schema, not in
/// `public`.
#[test]
fn test_startup_migrations_create_tables_in_configured_schema() {
    let db = TestDb::new();
    db.migrate(Some(SCHEMA_A));

    assert!(db.table_exists("tenant_a.admin_role"));
    assert!(db.table_exists("tenant_a.__diesel_schema_migrations"));
    assert!(
        !db.table_exists("public.admin_role"),
        "migrations for a schema must not create tables in public"
    );
    assert!(!db.table_exists("public.__diesel_schema_migrations"));
}

/// Startup path: the system generator is made through the DAL of the
/// configured schema only.
#[test]
fn test_startup_system_generator_is_provisioned_in_configured_schema() {
    let db = three_schema_db();

    let id = db
        .dal(Some(SCHEMA_A))
        .generators()
        .provision_system_generator()
        .expect("Failed to provision system generator");

    assert_eq!(
        db.dal(Some(SCHEMA_A))
            .generators()
            .get_system_generator_id()
            .unwrap(),
        Some(id)
    );
    assert_eq!(
        db.dal(Some(SCHEMA_B))
            .generators()
            .get_system_generator_id()
            .unwrap(),
        None
    );
    assert_eq!(
        db.dal(None).generators().get_system_generator_id().unwrap(),
        None
    );
}

/// Background task path: the audit log cleanup task calls
/// `cleanup_old_logs` on the DAL that `serve` built. It must delete rows in
/// its schema only.
#[test]
fn test_background_cleanup_deletes_in_configured_schema_only() {
    let db = three_schema_db();

    for schema in [Some(SCHEMA_A), Some(SCHEMA_B), None] {
        let log = NewAuditLog::new(
            ACTOR_TYPE_ADMIN,
            None,
            ACTION_PAK_ROTATED,
            RESOURCE_TYPE_GENERATOR,
            None,
        )
        .unwrap();
        db.dal(schema).audit_logs().create(&log).unwrap();
    }

    // A negative retention puts the cutoff in the future, so every row in the
    // schema is old enough to delete.
    let deleted = db
        .dal(Some(SCHEMA_A))
        .audit_logs()
        .cleanup_old_logs(-1)
        .unwrap();

    assert_eq!(deleted, 1, "only the row in tenant_a may be deleted");
    assert_eq!(db.dal(Some(SCHEMA_A)).audit_logs().count(None).unwrap(), 0);
    assert_eq!(db.dal(Some(SCHEMA_B)).audit_logs().count(None).unwrap(), 1);
    assert_eq!(db.dal(None).audit_logs().count(None).unwrap(), 1);
}

/// CLI path: `create generator`, `rotate generator`, `create agent` and
/// `rotate agent` read and write the configured schema only.
#[test]
fn test_cli_commands_use_configured_schema() {
    let db = three_schema_db();
    let settings_a = db.settings(Some(SCHEMA_A));
    let settings_b = db.settings(Some(SCHEMA_B));

    let generator_name = format!("gen-{}", Uuid::new_v4().simple());
    create_generator(&settings_a, generator_name.clone(), None)
        .expect("create generator should succeed in tenant_a");

    let generator = db
        .dal(Some(SCHEMA_A))
        .generators()
        .get_by_name(&generator_name)
        .unwrap()
        .expect("the generator must be in tenant_a");
    assert!(
        db.dal(Some(SCHEMA_B))
            .generators()
            .get_by_name(&generator_name)
            .unwrap()
            .is_none(),
        "the generator must not be in tenant_b"
    );
    assert!(
        db.dal(None)
            .generators()
            .get_by_name(&generator_name)
            .unwrap()
            .is_none(),
        "the generator must not be in public"
    );

    rotate_generator_key(&settings_a, generator.id)
        .expect("rotate generator should find the generator in tenant_a");
    assert!(
        rotate_generator_key(&settings_b, generator.id).is_err(),
        "rotate generator with tenant_b must not find a generator of tenant_a"
    );

    let agent_name = format!("agent-{}", Uuid::new_v4().simple());
    create_agent(
        &settings_a,
        agent_name.clone(),
        "cluster".to_string(),
        vec![generator.id],
    )
    .expect("create agent should succeed in tenant_a");

    let agent = db
        .dal(Some(SCHEMA_A))
        .agents()
        .list()
        .unwrap()
        .into_iter()
        .find(|a| a.name == agent_name)
        .expect("the agent must be in tenant_a");
    for schema in [Some(SCHEMA_B), None] {
        assert!(
            !db.dal(schema)
                .agents()
                .list()
                .unwrap()
                .iter()
                .any(|a| a.name == agent_name),
            "the agent must not be in {:?}",
            schema
        );
    }

    rotate_agent_key(&settings_a, agent.id)
        .expect("rotate agent should find the agent in tenant_a");
    assert!(
        rotate_agent_key(&settings_b, agent.id).is_err(),
        "rotate agent with tenant_b must not find an agent of tenant_a"
    );
}

/// The fault that BROKKR-T-0327 found: `search_path` is `<schema>, public`,
/// and PostgreSQL accepts a schema that does not exist. A CLI command run with
/// a misspelled or not migrated schema then wrote into `public` and reported
/// success. It must now fail, and write nothing.
#[test]
fn test_cli_refuses_schema_without_tables() {
    let db = TestDb::new();
    db.migrate(None);

    // A schema that does not exist.
    let missing = db.settings(Some("tenant_missing"));
    let name = format!("gen-{}", Uuid::new_v4().simple());
    let result = create_generator(&missing, name.clone(), None);
    assert!(
        db.dal(None)
            .generators()
            .get_by_name(&name)
            .unwrap()
            .is_none(),
        "a command with a missing schema must not write into public"
    );
    let err = result.expect_err("a command with a missing schema must fail");
    assert!(
        err.to_string().contains("tenant_missing"),
        "the error must name the schema, got: {}",
        err
    );

    // A schema that exists but has no tables.
    db.pool(Some(SCHEMA_A))
        .setup_schema(SCHEMA_A)
        .expect("Failed to set up schema");
    let empty = db.settings(Some(SCHEMA_A));
    let name = format!("gen-{}", Uuid::new_v4().simple());
    let result = create_generator(&empty, name.clone(), None);
    assert!(
        db.dal(None)
            .generators()
            .get_by_name(&name)
            .unwrap()
            .is_none(),
        "a command with a schema that is not migrated must not write into public"
    );
    assert!(
        result.is_err(),
        "a command with a schema that is not migrated must fail"
    );

    // With no schema configured, the commands use public as before.
    let public = db.settings(None);
    let name = format!("gen-{}", Uuid::new_v4().simple());
    create_generator(&public, name.clone(), None).expect("public must still work");
    assert!(
        db.dal(None)
            .generators()
            .get_by_name(&name)
            .unwrap()
            .is_some()
    );
}
