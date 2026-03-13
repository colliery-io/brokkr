---
id: domain-5-configuration-and
level: task
title: "Domain 5: Configuration and Deployment Verification"
short_code: "BROKKR-T-0124"
created_at: 2026-03-13T14:01:17.906844+00:00
updated_at: 2026-03-13T14:52:02.348928+00:00
parent: BROKKR-I-0015
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0015
---

# Domain 5: Configuration and Deployment Verification

## Parent Initiative

[[BROKKR-I-0015]] — Documentation Validation Against Implementation

## Objective

Verify every configuration and deployment claim across getting-started docs, Helm chart READMEs, and RBAC documentation against the actual environment variable parsing code, Helm `values.yaml` files, Helm templates, and RBAC ClusterRole templates. Configuration documentation is the most user-facing and error-sensitive domain — a wrong env var name or a missing Helm value sends users on a wild goose chase.

## Documentation Files in Scope

- `docs/content/getting-started/configuration.md` (if exists)
- `docs/content/getting-started/installation.md` (if exists)
- `docs/content/getting-started/quick-start.md` (if exists)
- `charts/brokkr-broker/README.md`
- `charts/brokkr-agent/README.md`
- `charts/brokkr-agent/RBAC.md`
- `README.md` (root project README — configuration and quick-start sections)

## Source of Truth

- Environment variable parsing code in broker and agent (look for `std::env::var`, `env::var`, config structs)
- Helm `values.yaml` for broker and agent charts
- Helm templates (`templates/*.yaml`) — what values are actually referenced
- RBAC templates (`templates/clusterrole.yaml`, `templates/clusterrolebinding.yaml`)
- CLI argument parsing (if any — clap or structopt usage)
- Docker Compose files (if referenced in getting-started)

## Verification Checklist

### Environment Variables
For each documented environment variable:
- [ ] Variable name exactly matches the string in `env::var("...")` or equivalent
- [ ] Default value documented matches the fallback in code
- [ ] Description of behavior matches what the code does with the value
- [ ] Required vs optional status matches (does code panic/exit without it, or has a default?)
- [ ] Any documented validation rules match actual parsing (e.g., must be a number, valid URL)

For completeness:
- [ ] No env vars exist in code that are undocumented (gap inventory)
- [ ] No env vars are documented that don't exist in code (dead documentation)

### Helm Charts
For each documented Helm value:
- [ ] Value path exists in `values.yaml`
- [ ] Default value in `values.yaml` matches documentation
- [ ] Value is actually used in at least one template (not dead configuration)
- [ ] Type (string, int, bool, object) matches documentation
- [ ] Description matches actual behavior when the value is applied

For Helm value completeness:
- [ ] No values in `values.yaml` are undocumented
- [ ] No documented values are absent from `values.yaml`

### RBAC
For each documented RBAC rule:
- [ ] API group matches the ClusterRole template
- [ ] Resources listed match the template
- [ ] Verbs (get, list, watch, create, update, patch, delete) match the template
- [ ] Any documented rationale for why each permission is needed is accurate

### Getting Started Content
- [ ] Any curl commands or API calls use correct URLs, ports, and payloads
- [ ] Docker/docker-compose commands work with current image names and tags
- [ ] Prerequisites listed are accurate and complete

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Every documented env var traced to parsing code and verified
- [ ] Every Helm value traced to `values.yaml` and at least one template usage
- [ ] RBAC ClusterRole template verified against documentation
- [ ] Complete gap inventory of undocumented env vars and Helm values
- [ ] All findings recorded using verdict taxonomy
- [ ] All non-CORRECT findings fixed in documentation

## Findings Report

*To be populated during verification. Use this format:*

```
### [filename.md]
| Line/Section | Claim | Verdict | Source File | Notes |
|---|---|---|---|---|
| ... | ... | ... | ... | ... |
```

## Status Updates

### Fixes Applied (Session 2)

**configuration.md** (7 fixes):
- CORS allowed_origins default: `["*"]` → `["http://localhost:3001"]`
- CORS allowed_methods: added missing `OPTIONS`
- polling_interval default: 30 → 10
- max_retries default: 3 → 60
- max_event_message_retries default: 3 → 2
- TOML example polling_interval/max_retries updated
- ConfigMap watcher → filesystem watcher

**RBAC.md** (2 structural fixes):
- Overview: rewrote from "read-only observation" to "read+write deployment management"
- "Future Write Permissions (Phase 3+)": replaced with "Deployment Write Permissions" documenting actual wildcard `["*"]` permissions on core/apps/batch/networking/RBAC API groups

**Agent README.md** (3 fixes):
- PAK env var: `BROKKR__BROKER__PAK` → `BROKKR__AGENT__PAK`
- RBAC section: rewrote from "read-only" to document observation + deployment wildcard permissions
- Development phases: Phase 2 marked complete, Phase 3 removed (already implemented)