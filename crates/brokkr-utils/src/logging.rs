/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! # Brokkr Logging Module
//!
//! This module provides a custom logging framework for the Brokkr application.
//!
//! ## Features
//! - Thread-safe logging
//! - Dynamic log level adjustment
//! - Custom log macros for easy use
//!
//! ## Usage
//!
//! 1. Initialize the logger:
//!    ```
//!    brokkr_logger::init("info").expect("Failed to initialize logger");
//!    ```
//!
//! 2. Use the log macros throughout your code:
//!    ```
//!    debug!("This is a debug message");
//!    info!("This is an info message");
//!    warn!("This is a warning message");
//!    error!("This is an error message");
//!    ```
//!
//! 3. Update log level at runtime if needed:
//!    ```
//!    brokkr_logger::update_log_level("debug").expect("Failed to update log level");
//!    ```
//!
//! ## Log Levels
//!
//! The available log levels are:
//! - "off": Turn off all logging
//! - "error": Log only errors
//! - "warn": Log warnings and errors
//! - "info": Log info, warnings, and errors (default)
//! - "debug": Log debug messages and all above
//! - "trace": Log trace messages and all above
//!
//! ## Thread Safety
//!
//! This logger is designed to be thread-safe and can handle concurrent logging
//! operations and log level changes from multiple threads.

use log::{LevelFilter, Metadata, Record, SetLoggerError};
use once_cell::sync::OnceCell;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

pub use log::{debug, error, info, trace, warn};

static LOGGER: BrokkrLogger = BrokkrLogger;
static CURRENT_LEVEL: AtomicUsize = AtomicUsize::new(LevelFilter::Info as usize);
static JSON_FORMAT: AtomicBool = AtomicBool::new(false);
static INIT: OnceCell<()> = OnceCell::new();

/// Custom logger for the Brokkr application
pub struct BrokkrLogger;

impl log::Log for BrokkrLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level()
            <= level_filter_from_u8(CURRENT_LEVEL.load(Ordering::Relaxed).try_into().unwrap())
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            if JSON_FORMAT.load(Ordering::Relaxed) {
                // JSON structured logging format
                let log_entry = serde_json::json!({
                    "timestamp": chrono::Utc::now().to_rfc3339(),
                    "level": record.level().to_string().to_lowercase(),
                    "target": record.target(),
                    "message": format!("{}", record.args()),
                    "module": record.module_path(),
                    "file": record.file(),
                    "line": record.line()
                });
                eprintln!("{}", log_entry);
            } else {
                // Human-readable text format
                eprintln!(
                    "{} - {}: {}",
                    chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                    record.level(),
                    record.args()
                );
            }
        }
    }

    fn flush(&self) {}
}

/// Initializes the Brokkr logging system with the specified log level.
///
/// Sets up a custom logger that handles structured logging with timestamps,
/// log levels, and module paths. Supports multiple output formats and
/// concurrent logging from multiple threads.
///
/// # Arguments
/// * `level` - String representation of the log level ("debug", "info", "warn", "error")
///
/// # Returns
/// * `Result<(), Box<dyn Error>>` - Success/failure of logger initialization
///
/// # Example
/// ```
/// use brokkr_utils::logging;
///
/// logging::init("debug")?;
/// log::info!("Logger initialized successfully");
/// ```
///
/// # Features
/// - Thread-safe logging
/// - Configurable log levels
/// - Module path tracking
/// - Timestamp inclusion
/// - Error context preservation
///
/// # Error Cases
/// - Invalid log level string
/// - Logger already initialized
/// - System permission issues
pub fn init(level: &str) -> Result<(), SetLoggerError> {
    init_with_format(level, "text")
}

/// Initializes the Brokkr logging system with the specified log level and format.
///
/// # Arguments
/// * `level` - String representation of the log level ("debug", "info", "warn", "error")
/// * `format` - Log output format ("text" for human-readable, "json" for structured JSON)
///
/// # Returns
/// * `Result<(), SetLoggerError>` - Success/failure of logger initialization
pub fn init_with_format(level: &str, format: &str) -> Result<(), SetLoggerError> {
    let level_filter = str_to_level_filter(level);
    let use_json = format.eq_ignore_ascii_case("json");

    INIT.get_or_init(|| {
        log::set_logger(&LOGGER)
            .map(|()| log::set_max_level(LevelFilter::Trace))
            .expect("Failed to set logger");
    });

    JSON_FORMAT.store(use_json, Ordering::Relaxed);
    CURRENT_LEVEL.store(level_filter as usize, Ordering::Relaxed);
    log::set_max_level(level_filter);
    Ok(())
}

/// Updates the current log level.
///
/// # Arguments
///
/// * `level` - A string slice that holds the new desired log level.
///
/// # Returns
///
/// * `Ok(())` if the log level was successfully updated.
/// * `Err(String)` if there was an error updating the log level.
///
/// # Examples
///
/// ```
/// use brokkr_logger;
///
///     brokkr_logger::init("info").expect("Failed to initialize logger");
///     info!("This will be logged");
///
///     brokkr_logger::update_log_level("warn").expect("Failed to update log level");
///     info!("This will not be logged");
///     warn!("But this will be logged");
/// ```
pub fn update_log_level(level: &str) -> Result<(), String> {
    let new_level = str_to_level_filter(level);
    CURRENT_LEVEL.store(new_level as usize, Ordering::Relaxed);
    log::set_max_level(new_level);
    Ok(())
}

fn str_to_level_filter(level: &str) -> LevelFilter {
    match level.to_lowercase().as_str() {
        "off" => LevelFilter::Off,
        "error" => LevelFilter::Error,
        "warn" => LevelFilter::Warn,
        "info" => LevelFilter::Info,
        "debug" => LevelFilter::Debug,
        "trace" => LevelFilter::Trace,
        _ => LevelFilter::Info,
    }
}

fn level_filter_from_u8(v: u8) -> LevelFilter {
    match v {
        0 => LevelFilter::Off,
        1 => LevelFilter::Error,
        2 => LevelFilter::Warn,
        3 => LevelFilter::Info,
        4 => LevelFilter::Debug,
        5 => LevelFilter::Trace,
        _ => LevelFilter::Off,
    }
}

pub mod prelude {
    pub use log::{debug, error, info, trace, warn};
}
#[cfg(test)]
mod tests {
    use super::*;
    use log::LevelFilter;
    use std::sync::mpsc;
    use std::thread;
    use std::time::Duration;
    use std::time::Instant;

    #[test]
    /// Verifies that the logger initializes correctly with the specified log level.
    ///
    /// This test:
    /// 1. Initializes the logger with the "info" level
    /// 2. Checks that initialization is successful
    /// 3. Verifies that the current log level is set to Info
    fn test_init() {
        assert!(init("info").is_ok());
        assert_eq!(
            CURRENT_LEVEL.load(Ordering::Relaxed),
            LevelFilter::Info as usize
        );
    }

    #[test]
    /// Tests the ability to update the log level after initialization.
    ///
    /// This test:
    /// 1. Initializes the logger with "info" level
    /// 2. Updates the log level to "debug" and verifies the change
    /// 3. Updates the log level to "warn" and verifies the change
    fn test_update_log_level() {
        init("info").expect("Failed to initialize logger");

        assert!(update_log_level("debug").is_ok());
        assert_eq!(
            CURRENT_LEVEL.load(Ordering::Relaxed),
            LevelFilter::Debug as usize
        );

        assert!(update_log_level("warn").is_ok());
        assert_eq!(
            CURRENT_LEVEL.load(Ordering::Relaxed),
            LevelFilter::Warn as usize
        );
    }

    #[test]
    /// Checks the logger's behavior when given invalid log levels.
    ///
    /// This test:
    /// 1. Attempts to initialize with an invalid level, expecting it to default to Info
    /// 2. Tries to update to another invalid level, expecting no change
    fn test_invalid_log_level() {
        assert!(init("invalid_level").is_ok());
        assert_eq!(
            CURRENT_LEVEL.load(Ordering::Relaxed),
            LevelFilter::Info as usize
        );

        assert!(update_log_level("another_invalid_level").is_ok());
        assert_eq!(
            CURRENT_LEVEL.load(Ordering::Relaxed),
            LevelFilter::Info as usize
        );
    }

    #[test]
    /// Ensures that all log macros can be called without errors.
    ///
    /// This test initializes the logger at the debug level and calls each log macro.
    /// It doesn't verify the output, only that the calls don't cause errors.
    #[allow(clippy::assertions_on_constants)]
    fn test_log_macros() {
        init("debug").expect("Failed to initialize logger");

        debug!("This is a debug message");
        info!("This is an info message");
        warn!("This is a warning message");
        error!("This is an error message");

        assert!(true);
    }

    #[test]
    /// Tests thread safety and performance of the logger under concurrent usage.
    ///
    /// This test:
    /// 1. Spawns multiple threads that perform logging operations and level changes
    /// 2. Measures the time taken for operations and the maximum time for a single log
    /// 3. Verifies that all operations complete without errors or deadlocks
    /// 4. Checks that no single log operation takes an unusually long time
    fn test_thread_safety_and_performance() {
        init("info").expect("Failed to initialize logger");

        let thread_count = 10;
        let operations_per_thread = 10000;
        let (tx, rx) = mpsc::channel();

        let start_time = Instant::now();

        let threads: Vec<_> = (0..thread_count)
            .map(|_| {
                let tx = tx.clone();
                thread::spawn(move || {
                    let mut log_operations = 0;
                    let mut level_changes = 0;
                    let mut max_log_time = Duration::from_secs(0);

                    for _ in 0..operations_per_thread {
                        if rand::random::<bool>() {
                            let log_start = Instant::now();
                            info!("Test message");
                            let log_duration = log_start.elapsed();
                            max_log_time = max_log_time.max(log_duration);
                            log_operations += 1;
                        } else {
                            let new_level = match rand::random::<u8>() % 5 {
                                0 => "error",
                                1 => "warn",
                                2 => "info",
                                3 => "debug",
                                _ => "trace",
                            };
                            update_log_level(new_level).expect("Failed to update log level");
                            level_changes += 1;
                        }
                    }

                    tx.send((log_operations, level_changes, max_log_time))
                        .expect("Failed to send result");
                })
            })
            .collect();

        for thread in threads {
            thread.join().expect("Thread panicked");
        }

        drop(tx);

        let total_duration = start_time.elapsed();
        let results: Vec<(usize, usize, Duration)> = rx.iter().collect();

        assert_eq!(results.len(), thread_count, "Not all threads reported back");

        let total_log_operations: usize = results.iter().map(|&(logs, _, _)| logs).sum();
        let total_level_changes: usize = results.iter().map(|&(_, changes, _)| changes).sum();
        let max_log_time = results.iter().map(|&(_, _, time)| time).max().unwrap();

        println!("Test completed in {:?}", total_duration);
        println!("Total log operations: {}", total_log_operations);
        println!("Total level changes: {}", total_level_changes);
        println!("Max time for a single log operation: {:?}", max_log_time);

        // Check that no single log operation took an unusually long time (adjust threshold as needed)
        assert!(
            max_log_time < Duration::from_millis(10),
            "A log operation took too long: {:?}",
            max_log_time
        );

        // Verify that the total operation count is correct
        assert_eq!(
            total_log_operations + total_level_changes,
            thread_count * operations_per_thread
        );
    }
}
