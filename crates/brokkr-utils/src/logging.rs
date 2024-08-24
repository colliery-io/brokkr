use log::{ LevelFilter, Metadata, Record, SetLoggerError};
use std::sync::atomic::{AtomicUsize, Ordering};
use once_cell::sync::OnceCell;


static LOGGER: BrokkrLogger = BrokkrLogger;
static CURRENT_LEVEL: AtomicUsize = AtomicUsize::new(LevelFilter::Info as usize);
static INIT: OnceCell<()> = OnceCell::new();

/// Macro to log messages at various levels
#[macro_export]
macro_rules! log {
    ($level:expr, $($arg:tt)+) => {
        log::log!($level, $($arg)+)
    };
}

/// Macro to log debug messages
#[macro_export]
macro_rules! debug {
    ($($arg:tt)+) => {
        log!(log::Level::Debug, $($arg)+)
    };
}

/// Macro to log info messages
#[macro_export]
macro_rules! info {
    ($($arg:tt)+) => {
        log!(log::Level::Info, $($arg)+)
    };
}

/// Macro to log warning messages
#[macro_export]
macro_rules! warn {
    ($($arg:tt)+) => {
        log!(log::Level::Warn, $($arg)+)
    };
}

/// Macro to log error messages
#[macro_export]
macro_rules! error {
    ($($arg:tt)+) => {
        log!(log::Level::Error, $($arg)+)
    };
}


/// Custom logger for the Brokkr application
pub struct BrokkrLogger;

impl log::Log for BrokkrLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.level() <= level_filter_from_u8(CURRENT_LEVEL.load(Ordering::Relaxed).try_into().unwrap())
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            eprintln!("{} - {}: {}", 
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.args()
            );
        }
    }

    fn flush(&self) {}
}

pub fn init(level: &str) -> Result<(), SetLoggerError> {
    let level_filter = str_to_level_filter(level);
    
    INIT.get_or_init(|| {
        log::set_logger(&LOGGER)
            .map(|()| log::set_max_level(LevelFilter::Trace))
            .expect("Failed to set logger");
    });

    CURRENT_LEVEL.store(level_filter as usize, Ordering::Relaxed);
    log::set_max_level(level_filter);
    Ok(())
}

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

#[cfg(test)]
mod tests {
    use super::*;
    use log::LevelFilter;
    use std::sync::mpsc;
    use std::time::Instant;

    use std::thread;
    use std::time::Duration;
    

    #[test]
    fn test_init() {
        assert!(init("info").is_ok());
        assert_eq!(CURRENT_LEVEL.load(Ordering::Relaxed), LevelFilter::Info as usize);
    }

    #[test]
    fn test_update_log_level() {
        init("info").expect("Failed to initialize logger");
        
        assert!(update_log_level("debug").is_ok());
        assert_eq!(CURRENT_LEVEL.load(Ordering::Relaxed), LevelFilter::Debug as usize);
        
        assert!(update_log_level("warn").is_ok());
        assert_eq!(CURRENT_LEVEL.load(Ordering::Relaxed), LevelFilter::Warn as usize);
    }

    #[test]
    fn test_invalid_log_level() {
        assert!(init("invalid_level").is_ok());
        assert_eq!(CURRENT_LEVEL.load(Ordering::Relaxed), LevelFilter::Info as usize);
        
        assert!(update_log_level("another_invalid_level").is_ok());
        assert_eq!(CURRENT_LEVEL.load(Ordering::Relaxed), LevelFilter::Info as usize);
    }

    #[test]
    fn test_log_macros() {
        init("debug").expect("Failed to initialize logger");
        
        debug!("This is a debug message");
        info!("This is an info message");
        warn!("This is a warning message");
        error!("This is an error message");

        assert!(true);
    }
    #[test]
    fn test_thread_safety_and_performance() {
        // this test exists because of a dead lock we ran into while doing some basic testing. it might be over kill
        // but overkill is under rated
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

                    tx.send((log_operations, level_changes, max_log_time)).expect("Failed to send result");
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
        assert!(max_log_time < Duration::from_millis(10), "A log operation took too long: {:?}", max_log_time);

        // Verify that the total operation count is correct
        assert_eq!(total_log_operations + total_level_changes, thread_count * operations_per_thread);
    }
}