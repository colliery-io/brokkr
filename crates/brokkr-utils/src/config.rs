//! # Brokkr Config Module
//! This module provides a common configuration framework for our crates.
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
}

/// Represents the database configuration
#[derive(Debug, Deserialize, Clone)]
#[allow(unused)]
pub struct Database {
    /// Database connection URL
    pub url: String,
}

/// Represents the logging configuration
#[derive(Debug, Deserialize, Clone)]
pub struct Log {
    /// Log level (e.g., "info", "debug", "warn", "error")
    pub level: String,
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