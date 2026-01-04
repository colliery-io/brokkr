/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Configuration file watcher for hot-reload support.
//!
//! This module provides functionality to watch for changes to the broker's configuration
//! file and trigger configuration reloads automatically.

use brokkr_utils::config::ReloadableConfig;
use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

/// Configuration for the file watcher.
#[derive(Debug, Clone)]
pub struct ConfigWatcherConfig {
    /// Path to the configuration file to watch.
    pub config_file_path: String,
    /// Debounce duration to prevent rapid successive reloads.
    pub debounce_duration: Duration,
    /// Whether the watcher is enabled.
    pub enabled: bool,
}

impl Default for ConfigWatcherConfig {
    fn default() -> Self {
        Self {
            config_file_path: String::new(),
            debounce_duration: Duration::from_secs(5),
            enabled: true,
        }
    }
}

impl ConfigWatcherConfig {
    /// Creates a new ConfigWatcherConfig from environment variables.
    ///
    /// Looks for BROKKR_CONFIG_FILE environment variable to determine the config file path.
    /// If not set, returns None (watcher disabled).
    pub fn from_environment() -> Option<Self> {
        // Check if config file path is specified
        let config_file_path = match std::env::var("BROKKR_CONFIG_FILE") {
            Ok(path) if !path.is_empty() => path,
            _ => {
                debug!("BROKKR_CONFIG_FILE not set, config file watcher disabled");
                return None;
            }
        };

        // Verify the file exists
        if !Path::new(&config_file_path).exists() {
            warn!(
                "Config file '{}' does not exist, config file watcher disabled",
                config_file_path
            );
            return None;
        }

        // Check if watcher is explicitly disabled
        let enabled = std::env::var("BROKKR_CONFIG_WATCHER_ENABLED")
            .map(|v| v.to_lowercase() != "false" && v != "0")
            .unwrap_or(true);

        if !enabled {
            info!("Config file watcher explicitly disabled via environment variable");
            return None;
        }

        // Get debounce duration from environment (in seconds)
        let debounce_secs = std::env::var("BROKKR_CONFIG_WATCHER_DEBOUNCE_SECONDS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(5);

        Some(Self {
            config_file_path,
            debounce_duration: Duration::from_secs(debounce_secs),
            enabled: true,
        })
    }
}

/// Starts the configuration file watcher as a background task.
///
/// This function spawns a tokio task that watches for changes to the specified
/// configuration file and triggers configuration reloads with debouncing.
///
/// # Arguments
///
/// * `config` - The ReloadableConfig instance to reload on changes.
/// * `watcher_config` - Configuration for the watcher.
///
/// # Returns
///
/// A handle to the spawned task, or None if the watcher couldn't be started.
pub fn start_config_watcher(
    config: ReloadableConfig,
    watcher_config: ConfigWatcherConfig,
) -> Option<tokio::task::JoinHandle<()>> {
    if !watcher_config.enabled {
        info!("Config file watcher is disabled");
        return None;
    }

    info!(
        "Starting config file watcher for '{}' with {}s debounce",
        watcher_config.config_file_path,
        watcher_config.debounce_duration.as_secs()
    );

    let handle = tokio::spawn(async move {
        if let Err(e) = run_config_watcher(config, watcher_config).await {
            error!("Config file watcher error: {}", e);
        }
    });

    Some(handle)
}

/// Internal function that runs the configuration file watcher loop.
async fn run_config_watcher(
    config: ReloadableConfig,
    watcher_config: ConfigWatcherConfig,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let config_path = watcher_config.config_file_path.clone();
    let debounce_duration = watcher_config.debounce_duration;

    // Create a channel for file events
    let (tx, rx) = mpsc::channel();

    // Create a file watcher
    let mut watcher: RecommendedWatcher = notify::recommended_watcher(move |res: Result<Event, notify::Error>| {
        if let Ok(event) = res {
            // Only send for modify/create events
            if event.kind.is_modify() || event.kind.is_create() {
                let _ = tx.send(());
            }
        }
    })?;

    // Watch the config file's parent directory (some editors replace files atomically)
    let config_path_ref = Path::new(&config_path);
    let watch_path = config_path_ref
        .parent()
        .unwrap_or(config_path_ref);

    watcher.watch(watch_path, RecursiveMode::NonRecursive)?;

    info!(
        "Config file watcher started for '{}'",
        config_path
    );

    // Track last reload time for debouncing
    let mut last_reload: Option<Instant> = None;

    // Process events
    loop {
        // Block waiting for events with a timeout
        match rx.recv_timeout(Duration::from_secs(60)) {
            Ok(()) => {
                // Check debounce
                let should_reload = match last_reload {
                    Some(last) => last.elapsed() >= debounce_duration,
                    None => true,
                };

                if should_reload {
                    // Wait for debounce period to catch rapid successive changes
                    tokio::time::sleep(debounce_duration).await;

                    // Drain any additional events that came in
                    while rx.try_recv().is_ok() {}

                    debug!("Config file change detected, reloading...");
                    last_reload = Some(Instant::now());

                    // Perform the reload
                    match config.reload() {
                        Ok(changes) => {
                            if changes.is_empty() {
                                debug!("Config file changed but no configuration changes detected");
                            } else {
                                info!(
                                    "Config file watcher triggered configuration reload with {} change(s):",
                                    changes.len()
                                );
                                for change in &changes {
                                    info!(
                                        "  - {}: '{}' -> '{}'",
                                        change.key, change.old_value, change.new_value
                                    );
                                }
                            }
                        }
                        Err(e) => {
                            error!("Failed to reload configuration from file change: {}", e);
                        }
                    }
                } else {
                    debug!(
                        "Debouncing config file change (last reload {}ms ago)",
                        last_reload.map(|l| l.elapsed().as_millis()).unwrap_or(0)
                    );
                }
            }
            Err(mpsc::RecvTimeoutError::Timeout) => {
                // No events, continue watching
                continue;
            }
            Err(mpsc::RecvTimeoutError::Disconnected) => {
                warn!("Config file watcher channel disconnected");
                break;
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_watcher_config_default() {
        let config = ConfigWatcherConfig::default();
        assert!(config.config_file_path.is_empty());
        assert_eq!(config.debounce_duration, Duration::from_secs(5));
        assert!(config.enabled);
    }

    #[test]
    fn test_config_from_environment_no_file() {
        std::env::remove_var("BROKKR_CONFIG_FILE");
        let config = ConfigWatcherConfig::from_environment();
        assert!(config.is_none());
    }

    #[test]
    fn test_config_from_environment_disabled() {
        std::env::set_var("BROKKR_CONFIG_FILE", "/tmp/test-config.toml");
        std::env::set_var("BROKKR_CONFIG_WATCHER_ENABLED", "false");
        let config = ConfigWatcherConfig::from_environment();
        assert!(config.is_none());
        std::env::remove_var("BROKKR_CONFIG_FILE");
        std::env::remove_var("BROKKR_CONFIG_WATCHER_ENABLED");
    }
}
