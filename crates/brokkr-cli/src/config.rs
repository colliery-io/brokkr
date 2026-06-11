/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Connection configuration: where the broker is and which PAK to present.
//!
//! Values are resolved with a fixed precedence — **command-line flag > environment
//! variable > config file** — so an operator can keep a default broker in
//! `~/.brokkr/config` and override per-invocation without editing it.

use serde::Deserialize;
use std::path::{Path, PathBuf};

/// One layer of partially-specified connection settings. The config file, the
/// environment, and the CLI flags each produce one of these; [`resolve`] folds
/// them together by precedence.
#[derive(Debug, Clone, Default, Deserialize, PartialEq, Eq)]
pub struct ConfigLayer {
    /// Base URL of the broker. May or may not include the `/api/v1` suffix;
    /// [`normalize_base_url`] adds it when missing.
    pub broker_url: Option<String>,
    /// Project Access Key presented as `Authorization: Bearer <pak>`.
    pub pak: Option<String>,
}

/// Fully-resolved connection settings, ready to build a client from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedConfig {
    pub broker_url: String,
    pub pak: String,
}

/// Fold the three layers in precedence order — `flag` wins over `env`, which
/// wins over `file` — and fail with an actionable message if a required value
/// is missing from all three.
pub fn resolve(
    flag: &ConfigLayer,
    env: &ConfigLayer,
    file: &ConfigLayer,
) -> Result<ResolvedConfig, String> {
    // Treat a present-but-blank value as unset *per layer*, so an empty env var
    // can't shadow a real value in the config file.
    let nonblank = |v: &Option<String>| v.clone().filter(|s| !s.trim().is_empty());
    let pick = |get: fn(&ConfigLayer) -> &Option<String>| {
        nonblank(get(flag))
            .or_else(|| nonblank(get(env)))
            .or_else(|| nonblank(get(file)))
    };

    let broker_url = pick(|l| &l.broker_url).ok_or_else(|| {
        "no broker URL: pass --broker-url, set BROKKR_BROKER_URL, or add `broker_url` to \
         ~/.brokkr/config"
            .to_string()
    })?;
    let pak = pick(|l| &l.pak).ok_or_else(|| {
        "no PAK: pass --pak, set BROKKR_PAK, or add `pak` to ~/.brokkr/config".to_string()
    })?;

    Ok(ResolvedConfig {
        broker_url: normalize_base_url(&broker_url),
        pak,
    })
}

/// Ensure the base URL carries the `/api/v1` prefix the SDK expects. Accepts a
/// bare broker URL (`https://broker.example.com`) or one that already includes
/// the prefix, with or without a trailing slash, and never doubles it.
pub fn normalize_base_url(url: &str) -> String {
    let trimmed = url.trim().trim_end_matches('/');
    if trimmed.ends_with("/api/v1") {
        trimmed.to_string()
    } else {
        format!("{trimmed}/api/v1")
    }
}

/// Default config-file location, `~/.brokkr/config`. Returns `None` when the
/// home directory cannot be determined.
pub fn default_config_path() -> Option<PathBuf> {
    dirs::home_dir().map(|home| home.join(".brokkr").join("config"))
}

/// Read a TOML config layer from `path`. A missing file is **not** an error —
/// it yields an empty layer, since the values may come from flags or the
/// environment instead. A present-but-malformed file is an error.
pub fn load_file(path: &Path) -> Result<ConfigLayer, String> {
    match std::fs::read_to_string(path) {
        Ok(contents) => parse_file(&contents)
            .map_err(|e| format!("failed to parse config file {}: {e}", path.display())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(ConfigLayer::default()),
        Err(e) => Err(format!("failed to read config file {}: {e}", path.display())),
    }
}

/// Parse a TOML config layer from a string (separated out for testing).
pub fn parse_file(contents: &str) -> Result<ConfigLayer, toml::de::Error> {
    toml::from_str(contents)
}

/// Build the environment layer from `BROKKR_BROKER_URL` / `BROKKR_PAK`.
pub fn env_layer() -> ConfigLayer {
    ConfigLayer {
        broker_url: std::env::var("BROKKR_BROKER_URL").ok(),
        pak: std::env::var("BROKKR_PAK").ok(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn layer(url: Option<&str>, pak: Option<&str>) -> ConfigLayer {
        ConfigLayer {
            broker_url: url.map(String::from),
            pak: pak.map(String::from),
        }
    }

    #[test]
    fn flag_beats_env_beats_file() {
        let flag = layer(Some("https://flag"), None);
        let env = layer(Some("https://env"), Some("env-pak"));
        let file = layer(Some("https://file"), Some("file-pak"));
        let got = resolve(&flag, &env, &file).unwrap();
        // broker_url comes from the flag; pak falls through to env (flag empty).
        assert_eq!(got.broker_url, "https://flag/api/v1");
        assert_eq!(got.pak, "env-pak");
    }

    #[test]
    fn file_used_when_nothing_else_set() {
        let empty = ConfigLayer::default();
        let file = layer(Some("https://file/api/v1"), Some("file-pak"));
        let got = resolve(&empty, &empty, &file).unwrap();
        assert_eq!(got.broker_url, "https://file/api/v1");
        assert_eq!(got.pak, "file-pak");
    }

    #[test]
    fn missing_broker_url_is_an_error() {
        let only_pak = layer(None, Some("p"));
        let err = resolve(&only_pak, &ConfigLayer::default(), &ConfigLayer::default()).unwrap_err();
        assert!(err.contains("broker URL"), "{err}");
    }

    #[test]
    fn missing_pak_is_an_error() {
        let only_url = layer(Some("https://b"), None);
        let err = resolve(&only_url, &ConfigLayer::default(), &ConfigLayer::default()).unwrap_err();
        assert!(err.contains("PAK"), "{err}");
    }

    #[test]
    fn blank_values_are_treated_as_unset() {
        // An empty env var should not shadow a real file value.
        let env = layer(Some("   "), Some(""));
        let file = layer(Some("https://file"), Some("file-pak"));
        let got = resolve(&ConfigLayer::default(), &env, &file).unwrap();
        assert_eq!(got.broker_url, "https://file/api/v1");
        assert_eq!(got.pak, "file-pak");
    }

    #[test]
    fn normalize_adds_prefix_once() {
        assert_eq!(normalize_base_url("https://b"), "https://b/api/v1");
        assert_eq!(normalize_base_url("https://b/"), "https://b/api/v1");
        assert_eq!(normalize_base_url("https://b/api/v1"), "https://b/api/v1");
        assert_eq!(normalize_base_url("https://b/api/v1/"), "https://b/api/v1");
        assert_eq!(normalize_base_url("  https://b  "), "https://b/api/v1");
    }

    #[test]
    fn parse_file_reads_both_keys() {
        let got = parse_file("broker_url = \"https://b\"\npak = \"secret\"\n").unwrap();
        assert_eq!(got, layer(Some("https://b"), Some("secret")));
    }

    #[test]
    fn parse_file_tolerates_partial_and_empty() {
        assert_eq!(parse_file("").unwrap(), ConfigLayer::default());
        assert_eq!(
            parse_file("pak = \"only-pak\"\n").unwrap(),
            layer(None, Some("only-pak"))
        );
    }

    #[test]
    fn load_file_missing_is_empty_not_error() {
        let path = Path::new("/definitely/not/a/real/brokkr/config/path");
        assert_eq!(load_file(path).unwrap(), ConfigLayer::default());
    }
}
