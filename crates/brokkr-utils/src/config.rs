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
}

#[derive(Debug, Deserialize, Clone)]
pub struct Broker {
    /// PAK Hash
    pub pak_hash: Option<String>,
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

#[cfg(test)]
mod tests {
    use super::Settings;

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
}
