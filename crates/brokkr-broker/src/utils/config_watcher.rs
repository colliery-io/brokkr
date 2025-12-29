/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Kubernetes ConfigMap watcher for hot-reload support.
//!
//! This module provides functionality to watch for changes to the broker's ConfigMap
//! in Kubernetes and trigger configuration reloads automatically.

use brokkr_utils::config::ReloadableConfig;
use futures::StreamExt;
use k8s_openapi::api::core::v1::ConfigMap;
use kube::{
    api::Api,
    runtime::watcher::{self, Event},
    Client,
};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

/// Configuration for the ConfigMap watcher.
#[derive(Debug, Clone)]
pub struct ConfigWatcherConfig {
    /// Namespace where the ConfigMap is located.
    pub namespace: String,
    /// Name of the ConfigMap to watch.
    pub configmap_name: String,
    /// Debounce duration to prevent rapid successive reloads.
    pub debounce_duration: Duration,
    /// Whether the watcher is enabled.
    pub enabled: bool,
}

impl Default for ConfigWatcherConfig {
    fn default() -> Self {
        Self {
            namespace: String::new(),
            configmap_name: String::new(),
            debounce_duration: Duration::from_secs(5),
            enabled: true,
        }
    }
}

impl ConfigWatcherConfig {
    /// Creates a new ConfigWatcherConfig from environment variables.
    ///
    /// Detects if running in Kubernetes by checking for the service account namespace file.
    pub fn from_environment() -> Option<Self> {
        // Check if we're running in Kubernetes
        if !is_running_in_kubernetes() {
            info!("Not running in Kubernetes, ConfigMap watcher disabled");
            return None;
        }

        // Read namespace from service account
        let namespace = match std::fs::read_to_string(
            "/var/run/secrets/kubernetes.io/serviceaccount/namespace",
        ) {
            Ok(ns) => ns.trim().to_string(),
            Err(e) => {
                warn!("Failed to read namespace from service account: {}", e);
                return None;
            }
        };

        // Get ConfigMap name from environment or use default
        let configmap_name = std::env::var("BROKKR_CONFIGMAP_NAME")
            .unwrap_or_else(|_| "brokkr-broker".to_string());

        // Check if watcher is explicitly disabled
        let enabled = std::env::var("BROKKR_CONFIG_WATCHER_ENABLED")
            .map(|v| v.to_lowercase() != "false" && v != "0")
            .unwrap_or(true);

        if !enabled {
            info!("ConfigMap watcher explicitly disabled via environment variable");
            return None;
        }

        // Get debounce duration from environment (in seconds)
        let debounce_secs = std::env::var("BROKKR_CONFIG_WATCHER_DEBOUNCE_SECONDS")
            .ok()
            .and_then(|v| v.parse().ok())
            .unwrap_or(5);

        Some(Self {
            namespace,
            configmap_name,
            debounce_duration: Duration::from_secs(debounce_secs),
            enabled: true,
        })
    }
}

/// Checks if the application is running inside a Kubernetes cluster.
fn is_running_in_kubernetes() -> bool {
    // Check for Kubernetes service host environment variable
    std::env::var("KUBERNETES_SERVICE_HOST").is_ok()
}

/// Starts the ConfigMap watcher as a background task.
///
/// This function spawns a tokio task that watches for changes to the specified
/// ConfigMap and triggers configuration reloads with debouncing.
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
        info!("ConfigMap watcher is disabled");
        return None;
    }

    info!(
        "Starting ConfigMap watcher for {}/{} with {}s debounce",
        watcher_config.namespace,
        watcher_config.configmap_name,
        watcher_config.debounce_duration.as_secs()
    );

    let handle = tokio::spawn(async move {
        if let Err(e) = run_config_watcher(config, watcher_config).await {
            error!("ConfigMap watcher error: {}", e);
        }
    });

    Some(handle)
}

/// Internal function that runs the ConfigMap watcher loop.
async fn run_config_watcher(
    config: ReloadableConfig,
    watcher_config: ConfigWatcherConfig,
) -> Result<(), kube::Error> {
    // Create Kubernetes client
    let client = Client::try_default().await?;
    let configmaps: Api<ConfigMap> = Api::namespaced(client, &watcher_config.namespace);

    // Create a field selector to watch only our specific ConfigMap
    let watch_config = watcher::Config::default()
        .fields(&format!("metadata.name={}", watcher_config.configmap_name));

    // Create the watcher stream
    let mut stream = watcher::watcher(configmaps, watch_config).boxed();

    // Track last reload time for debouncing
    let mut last_reload: Option<Instant> = None;

    // Channel for debounced reload requests
    let (reload_tx, mut reload_rx) = mpsc::channel::<()>(1);

    // Spawn debounce handler
    let debounce_duration = watcher_config.debounce_duration;
    let config_clone = config.clone();
    tokio::spawn(async move {
        while reload_rx.recv().await.is_some() {
            // Wait for debounce duration
            tokio::time::sleep(debounce_duration).await;

            // Drain any additional requests that came in during debounce
            while reload_rx.try_recv().is_ok() {}

            // Perform the reload
            match config_clone.reload() {
                Ok(changes) => {
                    if changes.is_empty() {
                        debug!("ConfigMap changed but no configuration changes detected");
                    } else {
                        info!(
                            "ConfigMap watcher triggered configuration reload with {} change(s):",
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
                    error!("Failed to reload configuration from ConfigMap change: {}", e);
                }
            }
        }
    });

    info!(
        "ConfigMap watcher started for {}/{}",
        watcher_config.namespace, watcher_config.configmap_name
    );

    // Process events from the watcher
    while let Some(event) = stream.next().await {
        match event {
            Ok(Event::Apply(cm)) | Ok(Event::InitApply(cm)) => {
                let name = cm.metadata.name.as_deref().unwrap_or("unknown");
                debug!("ConfigMap '{}' applied/updated", name);

                // Check debounce
                let should_reload = match last_reload {
                    Some(last) => last.elapsed() >= watcher_config.debounce_duration,
                    None => true,
                };

                if should_reload {
                    last_reload = Some(Instant::now());
                    // Send reload request (non-blocking)
                    let _ = reload_tx.try_send(());
                } else {
                    debug!(
                        "Debouncing ConfigMap change (last reload {}ms ago)",
                        last_reload.map(|l| l.elapsed().as_millis()).unwrap_or(0)
                    );
                }
            }
            Ok(Event::Delete(cm)) => {
                let name = cm.metadata.name.as_deref().unwrap_or("unknown");
                warn!("ConfigMap '{}' was deleted - configuration will not be updated", name);
            }
            Ok(Event::Init) => {
                debug!("ConfigMap watcher initialized");
            }
            Ok(Event::InitDone) => {
                debug!("ConfigMap watcher initial sync complete");
            }
            Err(e) => {
                error!("ConfigMap watcher error: {}", e);
                // Continue watching - the watcher will try to recover
            }
        }
    }

    warn!("ConfigMap watcher stream ended unexpectedly");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_watcher_config_default() {
        let config = ConfigWatcherConfig::default();
        assert!(config.namespace.is_empty());
        assert!(config.configmap_name.is_empty());
        assert_eq!(config.debounce_duration, Duration::from_secs(5));
        assert!(config.enabled);
    }

    #[test]
    fn test_is_running_in_kubernetes_false() {
        // When not in K8s, the env var shouldn't be set
        std::env::remove_var("KUBERNETES_SERVICE_HOST");
        assert!(!is_running_in_kubernetes());
    }

    #[test]
    fn test_is_running_in_kubernetes_true() {
        std::env::set_var("KUBERNETES_SERVICE_HOST", "10.0.0.1");
        assert!(is_running_in_kubernetes());
        std::env::remove_var("KUBERNETES_SERVICE_HOST");
    }

    #[test]
    fn test_config_from_environment_not_in_k8s() {
        std::env::remove_var("KUBERNETES_SERVICE_HOST");
        let config = ConfigWatcherConfig::from_environment();
        assert!(config.is_none());
    }
}
