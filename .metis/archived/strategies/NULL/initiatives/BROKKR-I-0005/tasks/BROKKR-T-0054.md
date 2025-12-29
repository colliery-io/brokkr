---
id: configure-cors-with-allowed
level: task
title: "Configure CORS with allowed origins from environment"
short_code: "BROKKR-T-0054"
created_at: 2025-12-29T14:27:13.172643+00:00
updated_at: 2025-12-29T14:59:54.555900+00:00
parent: BROKKR-I-0005
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: BROKKR-I-0005
---

# Configure CORS with allowed origins from environment

## Description

Replace the overly permissive CORS configuration (allow all) with configurable allowed origins read from environment/config.

## Files to Modify

- `crates/brokkr-utils/src/config.rs` - Add CORS config struct
- `crates/brokkr-utils/default.toml` - Add default CORS settings
- `crates/brokkr-broker/src/api/v1/mod.rs:37-41` - Use configured CORS

## Implementation

Config struct:
```rust
// In config.rs
#[derive(Debug, Deserialize, Clone)]
pub struct CorsConfig {
    pub allowed_origins: Vec<String>,
    pub allowed_methods: Vec<String>,
    pub allowed_headers: Vec<String>,
    pub max_age_seconds: u64,
}

impl Default for CorsConfig {
    fn default() -> Self {
        Self {
            allowed_origins: vec!["http://localhost:3001".to_string()],
            allowed_methods: vec!["GET", "POST", "PUT", "DELETE", "OPTIONS"]
                .into_iter().map(String::from).collect(),
            allowed_headers: vec!["Authorization", "Content-Type"]
                .into_iter().map(String::from).collect(),
            max_age_seconds: 3600,
        }
    }
}
```

Default config:
```toml
[cors]
allowed_origins = ["http://localhost:3001"]
allowed_methods = ["GET", "POST", "PUT", "DELETE", "OPTIONS"]
allowed_headers = ["Authorization", "Content-Type"]
max_age_seconds = 3600
```

Environment override:
```
BROKKR__CORS__ALLOWED_ORIGINS=https://ui.example.com,https://admin.example.com
```

Updated CORS layer:
```rust
fn build_cors_layer(config: &CorsConfig) -> CorsLayer {
    let origins: Vec<HeaderValue> = config.allowed_origins
        .iter()
        .filter_map(|o| o.parse().ok())
        .collect();
    
    let methods: Vec<Method> = config.allowed_methods
        .iter()
        .filter_map(|m| m.parse().ok())
        .collect();
    
    CorsLayer::new()
        .allow_origin(origins)
        .allow_methods(methods)
        .allow_headers([AUTHORIZATION, CONTENT_TYPE])
        .max_age(Duration::from_secs(config.max_age_seconds))
}
```

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] CORS rejects requests from non-allowed origins
- [ ] Allowed origins configurable via environment
- [ ] Default allows only localhost for development
- [ ] Helm chart values include CORS configuration
- [ ] Add integration test for CORS rejection

## Helm Chart Updates

Add to `charts/brokkr-broker/values.yaml`:
```yaml
cors:
  allowedOrigins:
    - "http://localhost:3001"
  allowedMethods:
    - "GET"
    - "POST"
    - "PUT"
    - "DELETE"
    - "OPTIONS"
  allowedHeaders:
    - "Authorization"
    - "Content-Type"
  maxAgeSeconds: 3600
```

Add to `charts/brokkr-broker/templates/configmap.yaml`:
```yaml
BROKKR__CORS__ALLOWED_ORIGINS: {{ .Values.cors.allowedOrigins | join "," | quote }}
```

## Docker Compose Updates

Update `.angreal/files/docker-compose.yaml` to include CORS config for local dev.

## Dependencies

- None (independent task)

## Breaking Change Warning

This is a **breaking change** for existing deployments:
- Previously: Any origin allowed
- After: Only configured origins allowed

**Migration path:**
1. Before upgrading, identify all origins that access the API
2. Configure `BROKKR__CORS__ALLOWED_ORIGINS` with those origins
3. Deploy update

## Notes

- Consider adding `*` as a special value to allow all origins (for backwards compat)
- Log rejected CORS requests at WARN level for debugging