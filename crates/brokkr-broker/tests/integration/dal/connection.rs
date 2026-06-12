/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! BROKKR-T-0222: DAL connection acquisition must surface pool exhaustion as a
//! normal error (which the API maps to a 500 via `ApiError::from_diesel`), not
//! a panic. Previously every DAL method did `pool.get().expect(...)`, so an
//! exhausted pool or a DB outage unwound inside the handler.

use brokkr_broker::dal::DAL;
use brokkr_broker::db::ConnectionPool;
use brokkr_utils::Settings;
use diesel::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};
use std::time::Duration;

#[test]
fn test_conn_pool_exhaustion_returns_error_not_panic() {
    dotenv::dotenv().ok();
    let settings = Settings::new(None).expect("Failed to load settings");

    // A pool of exactly one connection with a short acquisition timeout, so the
    // exhausted second request fails fast instead of blocking the 30s default.
    let manager = ConnectionManager::<PgConnection>::new(settings.database.url.as_str());
    let pool = Pool::builder()
        .max_size(1)
        .connection_timeout(Duration::from_secs(2))
        .build(manager)
        .expect("failed to build size-1 pool");
    let dal = DAL::new(ConnectionPool { pool, schema: None });

    // Hold the only connection for the duration of the test.
    let held = dal.conn().expect("first connection should succeed");

    // A DAL operation now has no connection available. It must return Err
    // (the r2d2 timeout mapped to a diesel DatabaseError), NOT panic.
    let result = dal.agents().list();
    assert!(result.is_err(), "expected pool-exhaustion error, got Ok(_)");

    // Releasing the held connection makes the DAL usable again.
    drop(held);
    dal.agents()
        .list()
        .expect("connection should be reusable after release");
}
