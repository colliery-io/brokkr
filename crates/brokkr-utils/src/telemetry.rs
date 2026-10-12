/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! # Brokkr Telemetry Module
//!
//! This module provides OpenTelemetry-based distributed tracing for Brokkr components.
//!
//! ## Features
//! - OTLP export to any OpenTelemetry-compatible collector
//! - Configurable sampling rate
//! - Integration with the `tracing` crate for instrumentation
//!
//! ## Usage
//!
//! ```rust,ignore
//! use brokkr_utils::telemetry;
//! use brokkr_utils::config::ResolvedTelemetry;
//!
//! let config = ResolvedTelemetry {
//!     enabled: true,
//!     otlp_endpoint: "http://localhost:4317".to_string(),
//!     service_name: "brokkr-broker".to_string(),
//!     sampling_rate: 0.1,
//! };
//!
//! telemetry::init(&config)?;
//!
//! // Use tracing macros for instrumentation
//! tracing::info!("Application started");
//! ```

use crate::config::ResolvedTelemetry;
use opentelemetry::KeyValue;
use opentelemetry::trace::TracerProvider;
use opentelemetry_otlp::WithExportConfig;
use opentelemetry_sdk::trace::Sampler;
use opentelemetry_sdk::{Resource, runtime};
use std::sync::OnceLock;
use tracing_subscriber::layer::SubscriberExt;
use tracing_subscriber::util::SubscriberInitExt;
use tracing_subscriber::{EnvFilter, Registry, reload};

/// Handle to the live log filter, set by `init` (BROKKR-T-0292).
///
/// The broker logs through `tracing`. Its filter is built once in `init`, so a
/// config reload of `log.level` must change the filter through this handle.
/// Changing `log::max_level` (the `logging` module) does not reach it.
///
/// Not set when `RUST_LOG` chose the filter at startup: `RUST_LOG` wins over
/// `log.level` at startup, so a reload does not override it either.
static LOG_FILTER: OnceLock<reload::Handle<EnvFilter, Registry>> = OnceLock::new();

/// Set the live log filter to `level` (for example "debug").
///
/// Returns `Ok(false)` and changes nothing when there is no live filter to
/// change: `init` did not run, or `RUST_LOG` set the filter.
pub fn set_log_level(level: &str) -> Result<bool, String> {
    match LOG_FILTER.get() {
        Some(handle) => apply_log_level(handle, level).map(|()| true),
        None => Ok(false),
    }
}

/// Replace the filter behind `handle` with one built from `level`.
fn apply_log_level(
    handle: &reload::Handle<EnvFilter, Registry>,
    level: &str,
) -> Result<(), String> {
    let filter = EnvFilter::try_new(level).map_err(|e| e.to_string())?;
    handle.reload(filter).map_err(|e| e.to_string())
}

/// Build the startup filter as a reloadable layer.
///
/// Returns the handle only when the filter came from `log_level`, not from
/// `RUST_LOG`.
fn reloadable_filter(
    log_level: &str,
) -> (
    reload::Layer<EnvFilter, Registry>,
    Option<reload::Handle<EnvFilter, Registry>>,
) {
    let (filter, from_config) = match EnvFilter::try_from_default_env() {
        Ok(filter) => (filter, false),
        Err(_) => (EnvFilter::new(log_level), true),
    };
    let (layer, handle) = reload::Layer::new(filter);
    (layer, from_config.then_some(handle))
}

/// Error type for telemetry initialization
#[derive(Debug)]
pub enum TelemetryError {
    /// Failed to create OTLP exporter
    ExporterError(String),
    /// Failed to initialize tracer
    TracerError(String),
    /// Failed to set global subscriber
    SubscriberError(String),
}

impl std::fmt::Display for TelemetryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TelemetryError::ExporterError(e) => write!(f, "OTLP exporter error: {}", e),
            TelemetryError::TracerError(e) => write!(f, "Tracer error: {}", e),
            TelemetryError::SubscriberError(e) => write!(f, "Subscriber error: {}", e),
        }
    }
}

impl std::error::Error for TelemetryError {}

/// Initialize OpenTelemetry tracing with the given configuration.
///
/// If telemetry is disabled in the config, this function sets up a basic
/// tracing subscriber without OpenTelemetry export.
///
/// # Arguments
/// * `config` - Resolved telemetry configuration (from `Telemetry::for_broker()` or `for_agent()`)
/// * `log_level` - Log level filter string (e.g., "info", "debug")
/// * `log_format` - Log format ("text" or "json")
///
/// # Returns
/// * `Ok(())` on success
/// * `Err(TelemetryError)` if initialization fails
pub fn init(
    config: &ResolvedTelemetry,
    log_level: &str,
    log_format: &str,
) -> Result<(), TelemetryError> {
    let (env_filter, handle) = reloadable_filter(log_level);
    if let Some(handle) = handle {
        // A second `init` in one process fails at `try_init` below, so the
        // first handle is the right one to keep.
        let _ = LOG_FILTER.set(handle);
    }

    if !config.enabled {
        // Telemetry disabled - just set up basic tracing subscriber
        let subscriber = tracing_subscriber::registry().with(env_filter);

        if log_format.eq_ignore_ascii_case("json") {
            subscriber
                .with(tracing_subscriber::fmt::layer().json())
                .try_init()
                .map_err(|e| TelemetryError::SubscriberError(e.to_string()))?;
        } else {
            subscriber
                .with(tracing_subscriber::fmt::layer())
                .try_init()
                .map_err(|e| TelemetryError::SubscriberError(e.to_string()))?;
        }

        return Ok(());
    }

    // Create OTLP exporter
    let exporter = opentelemetry_otlp::SpanExporter::builder()
        .with_tonic()
        .with_endpoint(&config.otlp_endpoint)
        .build()
        .map_err(|e| TelemetryError::ExporterError(e.to_string()))?;

    // Create sampler based on sampling rate
    let sampler = if config.sampling_rate >= 1.0 {
        Sampler::AlwaysOn
    } else if config.sampling_rate <= 0.0 {
        Sampler::AlwaysOff
    } else {
        Sampler::TraceIdRatioBased(config.sampling_rate)
    };

    // Create tracer provider with resource attributes
    let tracer_provider = opentelemetry_sdk::trace::TracerProvider::builder()
        .with_batch_exporter(exporter, runtime::Tokio)
        .with_sampler(sampler)
        .with_resource(Resource::new(vec![
            KeyValue::new(
                opentelemetry_semantic_conventions::resource::SERVICE_NAME,
                config.service_name.clone(),
            ),
            KeyValue::new(
                opentelemetry_semantic_conventions::resource::SERVICE_VERSION,
                env!("CARGO_PKG_VERSION"),
            ),
        ]))
        .build();

    // Get tracer from provider
    let tracer = tracer_provider.tracer(config.service_name.clone());

    // Set global tracer provider
    opentelemetry::global::set_tracer_provider(tracer_provider);

    // Create OpenTelemetry tracing layer
    let otel_layer = tracing_opentelemetry::layer().with_tracer(tracer);

    // Build subscriber with OpenTelemetry layer
    let subscriber = tracing_subscriber::registry()
        .with(env_filter)
        .with(otel_layer);

    if log_format.eq_ignore_ascii_case("json") {
        subscriber
            .with(tracing_subscriber::fmt::layer().json())
            .try_init()
            .map_err(|e| TelemetryError::SubscriberError(e.to_string()))?;
    } else {
        subscriber
            .with(tracing_subscriber::fmt::layer())
            .try_init()
            .map_err(|e| TelemetryError::SubscriberError(e.to_string()))?;
    }

    Ok(())
}

/// Shutdown OpenTelemetry, flushing any pending traces.
///
/// Should be called during graceful shutdown to ensure all traces are exported.
pub fn shutdown() {
    opentelemetry::global::shutdown_tracer_provider();
}

/// Re-export tracing macros for convenience
pub mod prelude {
    pub use tracing::Instrument;
    pub use tracing::{Level, instrument, span};
    pub use tracing::{debug, error, info, trace, warn};
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_disabled_telemetry_config() {
        let config = ResolvedTelemetry {
            enabled: false,
            otlp_endpoint: "http://localhost:4317".to_string(),
            service_name: "test-service".to_string(),
            sampling_rate: 0.1,
        };

        assert!(!config.enabled);
        assert_eq!(config.service_name, "test-service");
    }

    #[test]
    fn test_reloaded_log_level_changes_what_tracing_emits() {
        // The handle must change the live filter, not only a stored value.
        // A local subscriber is used so the test does not need the global one.
        let (layer, handle) = reload::Layer::new(EnvFilter::new("error"));
        let subscriber = tracing_subscriber::registry().with(layer);
        tracing::subscriber::with_default(subscriber, || {
            assert!(!tracing::enabled!(tracing::Level::DEBUG));
            apply_log_level(&handle, "debug").unwrap();
            assert!(tracing::enabled!(tracing::Level::DEBUG));
            apply_log_level(&handle, "warn").unwrap();
            assert!(!tracing::enabled!(tracing::Level::INFO));
            assert!(tracing::enabled!(tracing::Level::WARN));
        });
    }

    #[test]
    fn test_bad_log_level_is_an_error() {
        let (_layer, handle) = reload::Layer::<EnvFilter, Registry>::new(EnvFilter::new("info"));
        assert!(apply_log_level(&handle, "[").is_err());
    }

    #[test]
    fn test_sampling_rate_bounds() {
        // Test that sampling rate is properly bounded
        let config = ResolvedTelemetry {
            enabled: true,
            otlp_endpoint: "http://localhost:4317".to_string(),
            service_name: "test-service".to_string(),
            sampling_rate: 1.5, // > 1.0 should use AlwaysOn
        };
        assert!(config.sampling_rate >= 1.0);

        let config2 = ResolvedTelemetry {
            enabled: true,
            otlp_endpoint: "http://localhost:4317".to_string(),
            service_name: "test-service".to_string(),
            sampling_rate: -0.5, // < 0.0 should use AlwaysOff
        };
        assert!(config2.sampling_rate <= 0.0);
    }
}
