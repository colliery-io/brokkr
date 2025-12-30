/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! # Brokkr Config Module
//! This module provides a common configuration framework for our crates.
//!
//! For detailed configuration documentation, including environment variables,
//! configuration file format, and best practices, see the [Configuration Guide](https://brokkr.io/getting-started/configuration).
//!
//! # Variable Naming Convention
//!
//! Variables in this configuration framework follow these naming conventions:
//! - Struct fields use snake_case (e.g., `database`, `log_level`)
//! - Environment variables use SCREAMING_SNAKE_CASE and are prefixed with "BROKKR__" (e.g., `BROKKR__DATABASE__URL`)
//! - Configuration file keys use snake_case (e.g., `database.url`, `log.level`)
//!
//! # Configuration Overriding
//!
//! The configuration values are loaded and overridden in the following order (later sources take precedence):
//!
//! 1. Default values from the embedded `default.toml` file
//! 2. Values from an optional external configuration file (if provided)
//! 3. Environment variables
//!
//! To override a configuration value:
//! - In a configuration file: Use the appropriate key (e.g., `database.url = "new_value"`)
//! - Using environment variables: Set the variable with the "BROKKR__" prefix and "__" as separators
//!   (e.g., `BROKKR__DATABASE__URL=new_value`)
//!
//! # Available Environment Variables
//!
//! The following environment variables can be used to configure Brokkr:
//!
//! - `BROKKR__DATABASE__URL`: Sets the database connection URL
//!   Default: "postgres://brokkr:brokkr@localhost:5432/brokkr"
//!
//! - `BROKKR__DATABASE__SCHEMA`: Sets the PostgreSQL schema for multi-tenant isolation
//!   Default: None (uses public schema)
//!   Example: "tenant_acme"
//!
//! - `BROKKR__LOG__LEVEL`: Sets the log level for the application
//!   Default: "debug"
//!   Possible values: "trace", "debug", "info", "warn", "error"
//!
//! - `BROKKR__PAK__PREFIX`: Sets the prefix for PAKs (Pre-Authentication Keys)
//!   Default: "brokkr"
//!
//! - `BROKKR__PAK__RNG`: Sets the random number generator type for PAK generation
//!   Default: "osrng"
//!
//! - `BROKKR__PAK__DIGEST`: Sets the digest algorithm for PAK generation
//!   Default: 8
//!
//! - `BROKKR__PAK__SHORT_TOKEN_LENGTH`: Sets the length of short PAK tokens
//!   Default: 8
//!
//! - `BROKKR__PAK__LONG_TOKEN_LENGTH`: Sets the length of long PAK tokens
//!   Default: 24
//!
//! - `BROKKR__PAK__SHORT_TOKEN_PREFIX`: Sets the prefix for short PAK tokens
//!   Default: "BR"

use config::{Config, ConfigError, Environment, File};
use serde::Deserialize;
use std::sync::{Arc, RwLock};

// Include the default settings file as a string constant
const DEFAULT_SETTINGS: &str = include_str!("../default.toml");

/// Represents the main settings structure for the application
#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Settings {
    /// Database configuration
    pub database: Database,
    /// Logging configuration
    pub log: Log,
    /// PAK configuration
    pub pak: PAK,
    /// Agent configuration
    pub agent: Agent,
    /// Broker configuration
    pub broker: Broker,
    /// CORS configuration
    pub cors: Cors,
    /// Telemetry configuration
    pub telemetry: Telemetry,
}

/// Represents the CORS configuration
#[derive(Debug, Deserialize, Clone)]
pub struct Cors {
    /// Allowed origins for CORS requests
    /// Use "*" to allow all origins (not recommended for production)
    pub allowed_origins: Vec<String>,
    /// Allowed HTTP methods
    pub allowed_methods: Vec<String>,
    /// Allowed HTTP headers
    pub allowed_headers: Vec<String>,
    /// Max age for preflight cache in seconds
    pub max_age_seconds: u64,
}

#[derive(Debug, Deserialize, Clone)]
pub struct Broker {
    /// PAK Hash
    pub pak_hash: Option<String>,
    /// Interval for diagnostic cleanup task in seconds (default: 900 = 15 minutes)
    pub diagnostic_cleanup_interval_seconds: Option<u64>,
    /// Maximum age for completed/expired diagnostics before deletion in hours (default: 1)
    pub diagnostic_max_age_hours: Option<i64>,
    /// Webhook encryption key (hex-encoded, 32 bytes for AES-256)
    /// If not provided, a random key will be generated on startup (not recommended for production)
    pub webhook_encryption_key: Option<String>,
    /// Webhook delivery worker interval in seconds (default: 5)
    pub webhook_delivery_interval_seconds: Option<u64>,
    /// Webhook delivery batch size (default: 50)
    pub webhook_delivery_batch_size: Option<i64>,
    /// Webhook delivery cleanup retention in days (default: 7)
    pub webhook_cleanup_retention_days: Option<i64>,
    /// Audit log retention in days (default: 90)
    pub audit_log_retention_days: Option<i64>,
}

/// Represents the agent configuration
#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Agent {
    /// Broker URL
    pub broker_url: String,
    /// Polling interval in seconds
    pub polling_interval: u64,
    /// Kubeconfig path
    pub kubeconfig_path: Option<String>,
    /// Max number of retries
    pub max_retries: u32,
    /// PAK
    pub pak: String,
    /// Agent name
    pub agent_name: String,
    /// Cluster name
    pub cluster_name: String,
    /// Max number of retries for event messages
    pub max_event_message_retries: usize,
    /// Delay between event message retries in seconds
    pub event_message_retry_delay: u64,
    /// Health check HTTP server port
    pub health_port: Option<u16>,
    /// Whether deployment health checking is enabled
    pub deployment_health_enabled: Option<bool>,
    /// Interval for deployment health checks in seconds
    pub deployment_health_interval: Option<u64>,
}

/// Represents the database configuration
#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Database {
    /// Database connection URL
    pub url: String,
    /// Optional schema name for multi-tenant isolation
    pub schema: Option<String>,
}

/// Represents the logging configuration
#[derive(Debug, Deserialize, Clone)]
pub struct Log {
    /// Log level (e.g., "info", "debug", "warn", "error")
    pub level: String,
    /// Log format: "text" for human-readable, "json" for structured JSON
    #[serde(default = "default_log_format")]
    pub format: String,
}

fn default_log_format() -> String {
    "text".to_string()
}

/// Represents the telemetry (OpenTelemetry) configuration with hierarchical overrides
#[derive(Debug, Deserialize, Clone)]
pub struct Telemetry {
    /// Whether telemetry is enabled (base default)
    #[serde(default)]
    pub enabled: bool,
    /// OTLP endpoint for trace export (gRPC)
    #[serde(default = "default_otlp_endpoint")]
    pub otlp_endpoint: String,
    /// Service name for traces
    #[serde(default = "default_service_name")]
    pub service_name: String,
    /// Sampling rate (0.0 to 1.0)
    #[serde(default = "default_sampling_rate")]
    pub sampling_rate: f64,
    /// Broker-specific overrides
    #[serde(default)]
    pub broker: TelemetryOverride,
    /// Agent-specific overrides
    #[serde(default)]
    pub agent: TelemetryOverride,
}

/// Component-specific telemetry overrides (all fields optional)
#[derive(Debug, Deserialize, Clone, Default)]
pub struct TelemetryOverride {
    /// Override enabled flag
    pub enabled: Option<bool>,
    /// Override OTLP endpoint
    pub otlp_endpoint: Option<String>,
    /// Override service name
    pub service_name: Option<String>,
    /// Override sampling rate
    pub sampling_rate: Option<f64>,
}

/// Resolved telemetry configuration after merging base with overrides
#[derive(Debug, Clone)]
pub struct ResolvedTelemetry {
    pub enabled: bool,
    pub otlp_endpoint: String,
    pub service_name: String,
    pub sampling_rate: f64,
}

impl Telemetry {
    /// Get resolved telemetry config for broker (base merged with broker overrides)
    pub fn for_broker(&self) -> ResolvedTelemetry {
        ResolvedTelemetry {
            enabled: self.broker.enabled.unwrap_or(self.enabled),
            otlp_endpoint: self
                .broker
                .otlp_endpoint
                .clone()
                .unwrap_or_else(|| self.otlp_endpoint.clone()),
            service_name: self
                .broker
                .service_name
                .clone()
                .unwrap_or_else(|| self.service_name.clone()),
            sampling_rate: self.broker.sampling_rate.unwrap_or(self.sampling_rate),
        }
    }

    /// Get resolved telemetry config for agent (base merged with agent overrides)
    pub fn for_agent(&self) -> ResolvedTelemetry {
        ResolvedTelemetry {
            enabled: self.agent.enabled.unwrap_or(self.enabled),
            otlp_endpoint: self
                .agent
                .otlp_endpoint
                .clone()
                .unwrap_or_else(|| self.otlp_endpoint.clone()),
            service_name: self
                .agent
                .service_name
                .clone()
                .unwrap_or_else(|| self.service_name.clone()),
            sampling_rate: self.agent.sampling_rate.unwrap_or(self.sampling_rate),
        }
    }
}

fn default_otlp_endpoint() -> String {
    "http://localhost:4317".to_string()
}

fn default_service_name() -> String {
    "brokkr".to_string()
}

fn default_sampling_rate() -> f64 {
    0.1
}

/// Represents the PAK configuration
#[derive(Debug, Deserialize, Clone)]
pub struct PAK {
    /// PAK prefix
    pub prefix: Option<String>,
    /// Digest algorithm for PAK
    pub digest: Option<String>,
    /// RNG type for PAK
    pub rng: Option<String>,
    /// Short token length for PAK
    pub short_token_length: Option<usize>,
    /// Short token length as a string
    pub short_token_length_str: Option<String>,
    /// Prefix for short tokens
    pub short_token_prefix: Option<String>,
    /// Long token length for PAK
    pub long_token_length: Option<usize>,
    /// Long token length as a string
    pub long_token_length_str: Option<String>,
}

impl PAK {
    /// Convert short token length to string
    pub fn short_length_as_str(&mut self) {
        self.short_token_length_str = self.short_token_length.map(|v| v.to_string());
    }

    /// Convert long token length to string
    pub fn long_length_as_str(&mut self) {
        self.long_token_length_str = self.long_token_length.map(|v| v.to_string());
    }
}

impl Settings {
    /// Creates a new `Settings` instance
    ///
    /// # Arguments
    ///
    /// * `file` - An optional path to a configuration file
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the `Settings` instance or a `ConfigError`
    pub fn new(file: Option<String>) -> Result<Self, ConfigError> {
        // Start with default settings from the embedded TOML file
        let mut s = Config::builder()
            .add_source(File::from_str(DEFAULT_SETTINGS, config::FileFormat::Toml));

        // If a configuration file is provided, add it as a source
        s = match file {
            Some(x) => s.add_source(File::with_name(x.as_str())),
            None => s,
        };

        // Add environment variables as a source, prefixed with "BROKKR" and using "__" as a separator
        s = s.add_source(Environment::with_prefix("BROKKR").separator("__"));

        // Build the configuration
        let settings = s.build().unwrap();

        // Deserialize the configuration into a Settings instance
        settings.try_deserialize()
    }
}

/// Dynamic configuration values that can be hot-reloaded at runtime.
///
/// These settings can be updated without restarting the application.
/// Changes are applied atomically via the RwLock in ReloadableConfig.
#[derive(Debug, Clone)]
pub struct DynamicConfig {
    /// Log level (e.g., "info", "debug", "warn", "error")
    pub log_level: String,
    /// Interval for diagnostic cleanup task in seconds
    pub diagnostic_cleanup_interval_seconds: u64,
    /// Maximum age for completed/expired diagnostics before deletion in hours
    pub diagnostic_max_age_hours: i64,
    /// Webhook delivery worker interval in seconds
    pub webhook_delivery_interval_seconds: u64,
    /// Webhook delivery batch size
    pub webhook_delivery_batch_size: i64,
    /// Webhook delivery cleanup retention in days
    pub webhook_cleanup_retention_days: i64,
    /// Allowed origins for CORS requests
    pub cors_allowed_origins: Vec<String>,
    /// Max age for CORS preflight cache in seconds
    pub cors_max_age_seconds: u64,
}

impl DynamicConfig {
    /// Create DynamicConfig from Settings
    pub fn from_settings(settings: &Settings) -> Self {
        Self {
            log_level: settings.log.level.clone(),
            diagnostic_cleanup_interval_seconds: settings
                .broker
                .diagnostic_cleanup_interval_seconds
                .unwrap_or(900),
            diagnostic_max_age_hours: settings.broker.diagnostic_max_age_hours.unwrap_or(1),
            webhook_delivery_interval_seconds: settings
                .broker
                .webhook_delivery_interval_seconds
                .unwrap_or(5),
            webhook_delivery_batch_size: settings.broker.webhook_delivery_batch_size.unwrap_or(50),
            webhook_cleanup_retention_days: settings
                .broker
                .webhook_cleanup_retention_days
                .unwrap_or(7),
            cors_allowed_origins: settings.cors.allowed_origins.clone(),
            cors_max_age_seconds: settings.cors.max_age_seconds,
        }
    }
}

/// Represents a configuration change detected during reload
#[derive(Debug, Clone)]
pub struct ConfigChange {
    /// The configuration key that changed
    pub key: String,
    /// The old value (as string for display)
    pub old_value: String,
    /// The new value (as string for display)
    pub new_value: String,
}

/// Configuration wrapper that separates static (restart-required) settings
/// from dynamic (hot-reloadable) settings.
///
/// Static settings are immutable after creation and require an application
/// restart to change. Dynamic settings can be updated at runtime via the
/// `reload()` method.
///
/// # Example
///
/// ```rust,ignore
/// use brokkr_utils::config::ReloadableConfig;
///
/// let config = ReloadableConfig::new(None)?;
///
/// // Read dynamic config (thread-safe)
/// let log_level = config.log_level();
///
/// // Reload config from sources
/// let changes = config.reload()?;
/// for change in changes {
///     println!("Changed {}: {} -> {}", change.key, change.old_value, change.new_value);
/// }
/// ```
#[derive(Clone)]
pub struct ReloadableConfig {
    /// Static configuration that requires restart to change
    static_config: Settings,
    /// Dynamic configuration that can be hot-reloaded
    dynamic: Arc<RwLock<DynamicConfig>>,
    /// Optional path to config file for reloading
    config_file: Option<String>,
}

impl ReloadableConfig {
    /// Creates a new ReloadableConfig instance
    ///
    /// # Arguments
    ///
    /// * `file` - An optional path to a configuration file
    ///
    /// # Returns
    ///
    /// Returns a `Result` containing the `ReloadableConfig` instance or a `ConfigError`
    pub fn new(file: Option<String>) -> Result<Self, ConfigError> {
        let settings = Settings::new(file.clone())?;
        let dynamic = DynamicConfig::from_settings(&settings);

        Ok(Self {
            static_config: settings,
            dynamic: Arc::new(RwLock::new(dynamic)),
            config_file: file,
        })
    }

    /// Creates a ReloadableConfig from an existing Settings instance
    ///
    /// # Arguments
    ///
    /// * `settings` - The Settings instance to wrap
    /// * `config_file` - An optional path to the config file for future reloads
    ///
    /// # Returns
    ///
    /// Returns a `ReloadableConfig` instance
    pub fn from_settings(settings: Settings, config_file: Option<String>) -> Self {
        let dynamic = DynamicConfig::from_settings(&settings);

        Self {
            static_config: settings,
            dynamic: Arc::new(RwLock::new(dynamic)),
            config_file,
        }
    }

    /// Get a reference to the static (immutable) settings
    ///
    /// These settings require an application restart to change.
    pub fn static_config(&self) -> &Settings {
        &self.static_config
    }

    /// Reload dynamic configuration from sources (file + environment)
    ///
    /// Returns a list of configuration changes that were applied.
    /// Thread-safe: blocks writers during reload.
    pub fn reload(&self) -> Result<Vec<ConfigChange>, ConfigError> {
        // Load fresh settings from sources
        let new_settings = Settings::new(self.config_file.clone())?;
        let new_dynamic = DynamicConfig::from_settings(&new_settings);

        // Acquire write lock and compute changes
        let mut dynamic = self
            .dynamic
            .write()
            .map_err(|e| ConfigError::Message(format!("Failed to acquire write lock: {}", e)))?;

        let mut changes = Vec::new();

        // Check each field for changes
        if dynamic.log_level != new_dynamic.log_level {
            changes.push(ConfigChange {
                key: "log.level".to_string(),
                old_value: dynamic.log_level.clone(),
                new_value: new_dynamic.log_level.clone(),
            });
        }
        if dynamic.diagnostic_cleanup_interval_seconds
            != new_dynamic.diagnostic_cleanup_interval_seconds
        {
            changes.push(ConfigChange {
                key: "broker.diagnostic_cleanup_interval_seconds".to_string(),
                old_value: dynamic.diagnostic_cleanup_interval_seconds.to_string(),
                new_value: new_dynamic.diagnostic_cleanup_interval_seconds.to_string(),
            });
        }
        if dynamic.diagnostic_max_age_hours != new_dynamic.diagnostic_max_age_hours {
            changes.push(ConfigChange {
                key: "broker.diagnostic_max_age_hours".to_string(),
                old_value: dynamic.diagnostic_max_age_hours.to_string(),
                new_value: new_dynamic.diagnostic_max_age_hours.to_string(),
            });
        }
        if dynamic.webhook_delivery_interval_seconds
            != new_dynamic.webhook_delivery_interval_seconds
        {
            changes.push(ConfigChange {
                key: "broker.webhook_delivery_interval_seconds".to_string(),
                old_value: dynamic.webhook_delivery_interval_seconds.to_string(),
                new_value: new_dynamic.webhook_delivery_interval_seconds.to_string(),
            });
        }
        if dynamic.webhook_delivery_batch_size != new_dynamic.webhook_delivery_batch_size {
            changes.push(ConfigChange {
                key: "broker.webhook_delivery_batch_size".to_string(),
                old_value: dynamic.webhook_delivery_batch_size.to_string(),
                new_value: new_dynamic.webhook_delivery_batch_size.to_string(),
            });
        }
        if dynamic.webhook_cleanup_retention_days != new_dynamic.webhook_cleanup_retention_days {
            changes.push(ConfigChange {
                key: "broker.webhook_cleanup_retention_days".to_string(),
                old_value: dynamic.webhook_cleanup_retention_days.to_string(),
                new_value: new_dynamic.webhook_cleanup_retention_days.to_string(),
            });
        }
        if dynamic.cors_allowed_origins != new_dynamic.cors_allowed_origins {
            changes.push(ConfigChange {
                key: "cors.allowed_origins".to_string(),
                old_value: format!("{:?}", dynamic.cors_allowed_origins),
                new_value: format!("{:?}", new_dynamic.cors_allowed_origins),
            });
        }
        if dynamic.cors_max_age_seconds != new_dynamic.cors_max_age_seconds {
            changes.push(ConfigChange {
                key: "cors.max_age_seconds".to_string(),
                old_value: dynamic.cors_max_age_seconds.to_string(),
                new_value: new_dynamic.cors_max_age_seconds.to_string(),
            });
        }

        // Apply the new configuration
        *dynamic = new_dynamic;

        Ok(changes)
    }

    // ============================================
    // Convenience accessors for dynamic config
    // ============================================

    /// Get current log level
    pub fn log_level(&self) -> String {
        self.dynamic
            .read()
            .map(|d| d.log_level.clone())
            .unwrap_or_else(|_| "info".to_string())
    }

    /// Get diagnostic cleanup interval in seconds
    pub fn diagnostic_cleanup_interval_seconds(&self) -> u64 {
        self.dynamic
            .read()
            .map(|d| d.diagnostic_cleanup_interval_seconds)
            .unwrap_or(900)
    }

    /// Get diagnostic max age in hours
    pub fn diagnostic_max_age_hours(&self) -> i64 {
        self.dynamic
            .read()
            .map(|d| d.diagnostic_max_age_hours)
            .unwrap_or(1)
    }

    /// Get webhook delivery interval in seconds
    pub fn webhook_delivery_interval_seconds(&self) -> u64 {
        self.dynamic
            .read()
            .map(|d| d.webhook_delivery_interval_seconds)
            .unwrap_or(5)
    }

    /// Get webhook delivery batch size
    pub fn webhook_delivery_batch_size(&self) -> i64 {
        self.dynamic
            .read()
            .map(|d| d.webhook_delivery_batch_size)
            .unwrap_or(50)
    }

    /// Get webhook cleanup retention in days
    pub fn webhook_cleanup_retention_days(&self) -> i64 {
        self.dynamic
            .read()
            .map(|d| d.webhook_cleanup_retention_days)
            .unwrap_or(7)
    }

    /// Get CORS allowed origins
    pub fn cors_allowed_origins(&self) -> Vec<String> {
        self.dynamic
            .read()
            .map(|d| d.cors_allowed_origins.clone())
            .unwrap_or_else(|_| vec!["*".to_string()])
    }

    /// Get CORS max age in seconds
    pub fn cors_max_age_seconds(&self) -> u64 {
        self.dynamic.read().map(|d| d.cors_max_age_seconds).unwrap_or(3600)
    }

    /// Get a snapshot of all dynamic config values
    pub fn dynamic_snapshot(&self) -> Option<DynamicConfig> {
        self.dynamic.read().ok().map(|d| d.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::{DynamicConfig, ReloadableConfig, Settings, Telemetry, TelemetryOverride};

    #[test]
    /// Test the creation of Settings with default values
    ///
    /// This test ensures that:
    /// 1. A Settings instance can be created successfully using the `new` method
    /// 2. When no custom configuration is provided (None), the default values are set correctly
    /// 3. Specifically, it checks that the default database URL is set to the expected value
    fn test_settings_default_values() {
        // Attempt to create settings with default values (no custom configuration)
        let settings = Settings::new(None).unwrap();

        // Assert that the default database URL is set to the expected value
        assert_eq!(
            settings.database.url,
            "postgres://brokkr:brokkr@localhost:5432/brokkr"
        );
    }

    #[test]
    fn test_telemetry_default_values() {
        let settings = Settings::new(None).unwrap();

        // Check base telemetry defaults
        assert!(!settings.telemetry.enabled);
        assert_eq!(settings.telemetry.otlp_endpoint, "http://localhost:4317");
        assert_eq!(settings.telemetry.service_name, "brokkr");
        assert!((settings.telemetry.sampling_rate - 0.1).abs() < f64::EPSILON);
    }

    #[test]
    fn test_telemetry_for_broker_no_overrides() {
        let telemetry = Telemetry {
            enabled: true,
            otlp_endpoint: "http://collector:4317".to_string(),
            service_name: "base-service".to_string(),
            sampling_rate: 0.5,
            broker: TelemetryOverride::default(),
            agent: TelemetryOverride::default(),
        };

        let resolved = telemetry.for_broker();

        // With no overrides, should use base values
        assert!(resolved.enabled);
        assert_eq!(resolved.otlp_endpoint, "http://collector:4317");
        assert_eq!(resolved.service_name, "base-service");
        assert!((resolved.sampling_rate - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_telemetry_for_broker_full_overrides() {
        let telemetry = Telemetry {
            enabled: false,
            otlp_endpoint: "http://base:4317".to_string(),
            service_name: "base-service".to_string(),
            sampling_rate: 0.1,
            broker: TelemetryOverride {
                enabled: Some(true),
                otlp_endpoint: Some("http://broker-collector:4317".to_string()),
                service_name: Some("brokkr-broker".to_string()),
                sampling_rate: Some(1.0),
            },
            agent: TelemetryOverride::default(),
        };

        let resolved = telemetry.for_broker();

        // All values should be from broker overrides
        assert!(resolved.enabled);
        assert_eq!(resolved.otlp_endpoint, "http://broker-collector:4317");
        assert_eq!(resolved.service_name, "brokkr-broker");
        assert!((resolved.sampling_rate - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn test_telemetry_for_broker_partial_overrides() {
        let telemetry = Telemetry {
            enabled: true,
            otlp_endpoint: "http://shared-collector:4317".to_string(),
            service_name: "base-service".to_string(),
            sampling_rate: 0.5,
            broker: TelemetryOverride {
                enabled: None,                                      // Use base
                otlp_endpoint: None,                                // Use base
                service_name: Some("brokkr-broker".to_string()),    // Override
                sampling_rate: Some(0.8),                           // Override
            },
            agent: TelemetryOverride::default(),
        };

        let resolved = telemetry.for_broker();

        // Mix of base and override values
        assert!(resolved.enabled);                                       // From base
        assert_eq!(resolved.otlp_endpoint, "http://shared-collector:4317"); // From base
        assert_eq!(resolved.service_name, "brokkr-broker");              // From override
        assert!((resolved.sampling_rate - 0.8).abs() < f64::EPSILON);    // From override
    }

    #[test]
    fn test_telemetry_for_agent_no_overrides() {
        let telemetry = Telemetry {
            enabled: true,
            otlp_endpoint: "http://collector:4317".to_string(),
            service_name: "base-service".to_string(),
            sampling_rate: 0.5,
            broker: TelemetryOverride::default(),
            agent: TelemetryOverride::default(),
        };

        let resolved = telemetry.for_agent();

        // With no overrides, should use base values
        assert!(resolved.enabled);
        assert_eq!(resolved.otlp_endpoint, "http://collector:4317");
        assert_eq!(resolved.service_name, "base-service");
        assert!((resolved.sampling_rate - 0.5).abs() < f64::EPSILON);
    }

    #[test]
    fn test_telemetry_for_agent_full_overrides() {
        let telemetry = Telemetry {
            enabled: false,
            otlp_endpoint: "http://base:4317".to_string(),
            service_name: "base-service".to_string(),
            sampling_rate: 0.1,
            broker: TelemetryOverride::default(),
            agent: TelemetryOverride {
                enabled: Some(true),
                otlp_endpoint: Some("http://agent-collector:4317".to_string()),
                service_name: Some("brokkr-agent".to_string()),
                sampling_rate: Some(0.2),
            },
        };

        let resolved = telemetry.for_agent();

        // All values should be from agent overrides
        assert!(resolved.enabled);
        assert_eq!(resolved.otlp_endpoint, "http://agent-collector:4317");
        assert_eq!(resolved.service_name, "brokkr-agent");
        assert!((resolved.sampling_rate - 0.2).abs() < f64::EPSILON);
    }

    #[test]
    fn test_telemetry_broker_and_agent_independent() {
        let telemetry = Telemetry {
            enabled: true,
            otlp_endpoint: "http://shared:4317".to_string(),
            service_name: "brokkr".to_string(),
            sampling_rate: 0.1,
            broker: TelemetryOverride {
                enabled: Some(true),
                otlp_endpoint: Some("http://broker-collector:4317".to_string()),
                service_name: Some("brokkr-broker".to_string()),
                sampling_rate: Some(0.5),
            },
            agent: TelemetryOverride {
                enabled: Some(false),
                otlp_endpoint: Some("http://agent-collector:4317".to_string()),
                service_name: Some("brokkr-agent".to_string()),
                sampling_rate: Some(0.9),
            },
        };

        let broker_resolved = telemetry.for_broker();
        let agent_resolved = telemetry.for_agent();

        // Broker values
        assert!(broker_resolved.enabled);
        assert_eq!(broker_resolved.otlp_endpoint, "http://broker-collector:4317");
        assert_eq!(broker_resolved.service_name, "brokkr-broker");
        assert!((broker_resolved.sampling_rate - 0.5).abs() < f64::EPSILON);

        // Agent values - completely independent from broker
        assert!(!agent_resolved.enabled);
        assert_eq!(agent_resolved.otlp_endpoint, "http://agent-collector:4317");
        assert_eq!(agent_resolved.service_name, "brokkr-agent");
        assert!((agent_resolved.sampling_rate - 0.9).abs() < f64::EPSILON);
    }

    #[test]
    fn test_telemetry_override_enabled_false_overrides_base_true() {
        let telemetry = Telemetry {
            enabled: true,  // Base is enabled
            otlp_endpoint: "http://collector:4317".to_string(),
            service_name: "brokkr".to_string(),
            sampling_rate: 0.1,
            broker: TelemetryOverride {
                enabled: Some(false),  // But broker override disables it
                otlp_endpoint: None,
                service_name: None,
                sampling_rate: None,
            },
            agent: TelemetryOverride::default(),
        };

        let broker_resolved = telemetry.for_broker();
        let agent_resolved = telemetry.for_agent();

        // Broker should be disabled (override), agent should be enabled (base)
        assert!(!broker_resolved.enabled);
        assert!(agent_resolved.enabled);
    }

    #[test]
    fn test_telemetry_sampling_rate_extremes() {
        // Test 0.0 sampling rate
        let telemetry_zero = Telemetry {
            enabled: true,
            otlp_endpoint: "http://collector:4317".to_string(),
            service_name: "test".to_string(),
            sampling_rate: 0.0,
            broker: TelemetryOverride::default(),
            agent: TelemetryOverride::default(),
        };
        assert!((telemetry_zero.for_broker().sampling_rate - 0.0).abs() < f64::EPSILON);

        // Test 1.0 sampling rate
        let telemetry_full = Telemetry {
            enabled: true,
            otlp_endpoint: "http://collector:4317".to_string(),
            service_name: "test".to_string(),
            sampling_rate: 1.0,
            broker: TelemetryOverride::default(),
            agent: TelemetryOverride::default(),
        };
        assert!((telemetry_full.for_broker().sampling_rate - 1.0).abs() < f64::EPSILON);
    }

    // ============================================
    // ReloadableConfig tests
    // ============================================

    #[test]
    fn test_reloadable_config_creation() {
        let config = ReloadableConfig::new(None).unwrap();

        // Verify static config is accessible
        assert_eq!(
            config.static_config().database.url,
            "postgres://brokkr:brokkr@localhost:5432/brokkr"
        );

        // Verify dynamic config accessors work
        assert!(!config.log_level().is_empty());
        assert!(config.diagnostic_cleanup_interval_seconds() > 0);
        assert!(config.webhook_delivery_interval_seconds() > 0);
    }

    #[test]
    fn test_dynamic_config_from_settings() {
        let settings = Settings::new(None).unwrap();
        let dynamic = DynamicConfig::from_settings(&settings);

        // Check defaults are applied
        assert_eq!(dynamic.log_level, settings.log.level);
        assert_eq!(dynamic.diagnostic_cleanup_interval_seconds, 900); // default
        assert_eq!(dynamic.diagnostic_max_age_hours, 1); // default
        assert_eq!(dynamic.webhook_delivery_interval_seconds, 5); // default
        assert_eq!(dynamic.webhook_delivery_batch_size, 50); // default
        assert_eq!(dynamic.webhook_cleanup_retention_days, 7); // default
    }

    #[test]
    fn test_reloadable_config_accessors_with_defaults() {
        let config = ReloadableConfig::new(None).unwrap();

        // These should match the defaults in DynamicConfig::from_settings
        assert_eq!(config.diagnostic_cleanup_interval_seconds(), 900);
        assert_eq!(config.diagnostic_max_age_hours(), 1);
        assert_eq!(config.webhook_delivery_interval_seconds(), 5);
        assert_eq!(config.webhook_delivery_batch_size(), 50);
        assert_eq!(config.webhook_cleanup_retention_days(), 7);
        assert_eq!(config.cors_max_age_seconds(), 3600); // 1 hour default from default.toml
    }

    #[test]
    fn test_reloadable_config_dynamic_snapshot() {
        let config = ReloadableConfig::new(None).unwrap();

        let snapshot = config.dynamic_snapshot();
        assert!(snapshot.is_some());

        let snapshot = snapshot.unwrap();
        assert_eq!(snapshot.log_level, config.log_level());
        assert_eq!(
            snapshot.webhook_delivery_interval_seconds,
            config.webhook_delivery_interval_seconds()
        );
    }

    #[test]
    fn test_reloadable_config_reload_no_changes() {
        let config = ReloadableConfig::new(None).unwrap();

        // Reload should detect no changes when nothing changed
        let changes = config.reload().unwrap();
        assert!(
            changes.is_empty(),
            "Expected no changes but got: {:?}",
            changes
        );
    }

    #[test]
    fn test_reloadable_config_is_clone() {
        let config = ReloadableConfig::new(None).unwrap();
        let config_clone = config.clone();

        // Both should see the same dynamic config (Arc is shared)
        assert_eq!(config.log_level(), config_clone.log_level());
    }

    #[test]
    fn test_reloadable_config_thread_safety() {
        use std::sync::Arc;
        use std::thread;

        let config = Arc::new(ReloadableConfig::new(None).unwrap());

        let handles: Vec<_> = (0..4)
            .map(|_| {
                let config = Arc::clone(&config);
                thread::spawn(move || {
                    // Multiple threads reading config simultaneously
                    for _ in 0..100 {
                        let _ = config.log_level();
                        let _ = config.webhook_delivery_interval_seconds();
                        let _ = config.cors_allowed_origins();
                    }
                })
            })
            .collect();

        for handle in handles {
            handle.join().unwrap();
        }
    }
}
