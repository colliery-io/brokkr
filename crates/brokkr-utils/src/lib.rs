/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

pub mod config;
pub mod logging;
pub mod telemetry;

pub use config::{ResolvedTelemetry, Settings, Telemetry};
pub use logging::BrokkrLogger;

/// One lock for the tests that read the process environment, set an
/// environment variable, or move the logger's global level. Cargo runs the
/// tests of a crate in parallel in one process, so without it a reload in one
/// test sees the environment or the level of another (BROKKR-T-0378).
#[cfg(test)]
pub(crate) mod test_lock {
    use std::sync::{Mutex, MutexGuard};

    static LOCK: Mutex<()> = Mutex::new(());

    pub(crate) fn guard() -> MutexGuard<'static, ()> {
        LOCK.lock().unwrap_or_else(|e| e.into_inner())
    }
}
