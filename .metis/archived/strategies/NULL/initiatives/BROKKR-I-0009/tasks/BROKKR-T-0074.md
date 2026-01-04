---
id: implement-configuration-refresh
level: task
title: "Implement configuration refresh API endpoint"
short_code: "BROKKR-T-0074"
created_at: 2025-12-29T19:32:33.454596+00:00
updated_at: 2025-12-29T19:53:56.040570+00:00
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

# Implement configuration refresh API endpoint

## Parent Initiative

[[BROKKR-I-0009]]

## Objective

Add an admin-only API endpoint `POST /api/v1/admin/config/reload` that triggers a configuration reload and returns which settings were changed.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] `POST /api/v1/admin/config/reload` endpoint implemented
- [ ] Requires admin PAK authentication
- [ ] Returns JSON with list of changed settings
- [ ] Returns 200 on success with changes, 304 if no changes
- [ ] Returns 401/403 for unauthorized requests
- [ ] Integration tests for endpoint

## Implementation Notes

### Files to Create/Modify
- `crates/brokkr-broker/src/api/v1/admin.rs` - New admin routes module
- `crates/brokkr-broker/src/api/v1/mod.rs` - Register admin routes

### API Design

**Request:**
```
POST /api/v1/admin/config/reload
Authorization: Bearer <admin_pak>
```

**Response (200 OK):**
```json
{
  "reloaded_at": "2025-01-01T12:00:00Z",
  "changes": [
    {"key": "log.level", "old": "info", "new": "debug"},
    {"key": "cors.allowed_origins", "old": ["*"], "new": ["https://app.example.com"]}
  ]
}
```

**Response (304 Not Modified):**
```json
{
  "reloaded_at": "2025-01-01T12:00:00Z",
  "changes": []
}
```

### Dependencies
- Depends on T-0073 (ReloadableConfig wrapper)

## Status Updates

*To be added during implementation*