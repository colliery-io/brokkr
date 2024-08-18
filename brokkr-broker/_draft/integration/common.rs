use diesel::{sql_query, Connection, PgConnection, RunQueryDsl};
use dotenv::dotenv;
use std::env;
use std::sync::atomic::AtomicU32;
use std::sync::Arc;
use std::sync::Mutex;
use url::Url;

#[allow(dead_code)]
static TEST_DB_COUNTER: AtomicU32 = AtomicU32::new(0);
#[allow(dead_code)]
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");
type DB = diesel::pg::Pg;

pub fn run_db_migrations(conn: &mut impl MigrationHarness<DB>) {
    conn.run_pending_migrations(MIGRATIONS)
        .expect("Could not run migrations");
}

pub fn setup_test_db() -> (TestDb, Arc<Mutex<PgConnection>>) {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let test_db = TestDb::new(database_url);
    let conn = test_db.conn();
    (test_db, Arc::new(Mutex::new(conn)))
}

#[allow(dead_code)]
pub struct TestDb {
    database_url: String,
    url: String,
    name: String,
    delete_on_drop: bool,
}
#[allow(dead_code)]
impl TestDb {
    pub fn new(database_url: String) -> Self {
        let name = format!(
            "test_db_{}_{}",
            std::process::id(),
            TEST_DB_COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst)
        );

        let mut conn = PgConnection::establish(&database_url).unwrap();

        sql_query(format!("CREATE DATABASE {};", name))
            .execute(&mut conn)
            .unwrap();

        let mut url = Url::parse(&database_url).unwrap();
        url.set_path(&name);

        Self {
            database_url,
            url: url.to_string(),
            name,
            delete_on_drop: false,
        }
    }

    pub fn url(&self) -> &str {
        &self.url
    }

    pub fn conn(&self) -> PgConnection {
        let mut conn = PgConnection::establish(self.url.as_str()).unwrap();
        run_db_migrations(&mut conn);
        conn
    }

    pub fn leak(&mut self) {
        self.delete_on_drop = false;
    }
}
impl Drop for TestDb {
    fn drop(&mut self) {
        let mut conn = PgConnection::establish(&self.database_url).unwrap();
        sql_query(format!(
            "SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE datname = '{}'",
            self.name
        ))
        .execute(&mut conn)
        .unwrap();
        sql_query(format!("DROP DATABASE {}", self.name))
            .execute(&mut conn)
            .unwrap();
    }
}
