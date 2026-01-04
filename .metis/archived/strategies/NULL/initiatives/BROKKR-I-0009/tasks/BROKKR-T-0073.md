---
id: add-reloadableconfig-wrapper-and
level: task
title: "Add ReloadableConfig wrapper and dynamic config structure"
short_code: "BROKKR-T-0073"
created_at: 2025-12-29T19:32:33.341321+00:00
updated_at: 2025-12-29T19:43:22.595529+00:00
parent: BROKKR-I-0009
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: BROKKR-I-0009
---

# Add ReloadableConfig wrapper and dynamic config structure

## Parent Initiative

[[BROKKR-I-0009]]

## Objective

Create a `DynamicConfig` struct that holds hot-reloadable settings and wrap it in `Arc<RwLock<>>` for thread-safe runtime updates. Separate static config (database, encryption keys) from dynamic config (log level, webhook settings, CORS, diagnostic intervals).

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `DynamicConfig` struct created with hot-reloadable fields
- [ ] `ReloadableConfig` struct combining static `Settings` with dynamic `Arc<RwLock<DynamicConfig>>`
- [ ] Helper methods to read dynamic config values
- [ ] `reload()` method to update dynamic config from environment/file
- [ ] Unit tests for config reload behavior
- [ ] Existing broker code updated to use `ReloadableConfig`

## Implementation Notes

### Files to Modify
- `crates/brokkr-utils/src/config.rs` - Add DynamicConfig and ReloadableConfig
- `crates/brokkr-broker/src/bin.rs` - Use ReloadableConfig instead of Settings

### Dynamic (Hot-Reloadable) Settings
- `log.level` - Already uses atomic in logging module
- `broker.webhook_delivery_interval_seconds`
- `broker.webhook_delivery_batch_size`
- `broker.webhook_cleanup_retention_days`
- `broker.diagnostic_cleanup_interval_seconds`
- `broker.diagnostic_max_age_hours`
- `cors.allowed_origins`
- `cors.max_age_seconds`

### Static (Restart Required) Settings
- `database.url` - Connection pool already established
- `database.schema` - Schema set at startup
- `broker.webhook_encryption_key` - Encryption context established
- `telemetry.*` - OpenTelemetry provider initialized at startup
- `pak.*` - PAK controller initialized at startup

### Technical Approach
```rust
pub struct DynamicConfig {
    pub log_level: String,
    pub webhook_delivery_interval_seconds: u64,
    pub webhook_delivery_batch_size: i64,
    pub diagnostic_cleanup_interval_seconds: u64,
    pub cors_allowed_origins: Vec<String>,
}

pub struct ReloadableConfig {
    pub static_config: Settings,
    pub dynamic: Arc<RwLock<DynamicConfig>>,
}

impl ReloadableConfig {
    pub fn reload(&self) -> Result<Vec<String>, ConfigError>;
    pub fn log_level(&self) -> String;
    pub fn webhook_interval(&self) -> u64;
}
```

## Status Updates

*To be added during implementation*