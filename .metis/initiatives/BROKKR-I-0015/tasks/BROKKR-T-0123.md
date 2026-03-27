---
id: domain-4-observability-and
level: task
title: "Domain 4: Observability and Operations Verification"
short_code: "BROKKR-T-0123"
created_at: 2026-03-13T14:01:17.341221+00:00
updated_at: 2026-03-13T14:11:52.445069+00:00
parent: BROKKR-I-0015
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: BROKKR-I-0015
---

# Domain 4: Observability and Operations Verification

## Parent Initiative

[[BROKKR-I-0015]] — Documentation Validation Against Implementation

## Objective

Verify every observability and operations claim in the documentation against the actual health check handlers, Prometheus metric registrations and recording sites, and container image build configurations. This domain is particularly high-risk for drift because metrics and health endpoints are frequently modified during feature development without documentation updates.

## Documentation Files in Scope

- `docs/content/reference/health-endpoints.md` (~639 lines) — three-tier health checks, JSON response schemas, Kubernetes probe config
- `docs/content/reference/monitoring.md` (~599 lines) — full Prometheus metrics catalog, PromQL examples, alerting rules, Grafana dashboards
- `docs/content/reference/container-images.md` (~280 lines) — GHCR repos, tag formats, build commands
- `docs/content/how-to/deployment-health.md` (~281 lines) — health monitoring how-to

## Source of Truth

- Health check handler functions (broker and agent) — response structs, status logic, endpoint paths
- Prometheus metric definitions — `register_*` macros, metric names, types (counter/gauge/histogram), label sets, help descriptions
- Prometheus metric recording call sites — where `.inc()`, `.observe()`, `.set()` are called
- `Dockerfile` / container build configuration
- CI/CD workflows (GitHub Actions) for image building and tagging
- Helm chart liveness/readiness probe definitions

## Verification Checklist

### Health Endpoints
For each documented health endpoint (`/healthz`, `/readyz`, `/health`):
- [ ] Endpoint path exists in the axum router
- [ ] HTTP method matches documentation
- [ ] Response JSON structure matches the handler's response struct
- [ ] Status code logic (200 vs 503, etc.) matches documented conditions
- [ ] Individual health check components (database, Kubernetes connectivity, etc.) match implementation
- [ ] Response timing/performance claims are verifiable from code
- [ ] Kubernetes probe YAML examples use correct paths, ports, and thresholds

### Prometheus Metrics
For each documented metric:
- [ ] Metric name matches the registered name in code (exact string match)
- [ ] Metric type (counter, gauge, histogram) matches registration
- [ ] Label names and expected values match recording call sites
- [ ] Help/description string matches documentation
- [ ] PromQL examples use the correct metric name and label selectors
- [ ] Alerting rule YAML uses correct metric names and thresholds that align with implementation behavior

For metrics catalog completeness:
- [ ] No metrics exist in code that are undocumented (gap inventory)
- [ ] No metrics are documented that don't exist in code (dead documentation)

### Container Images
- [ ] Image names and registry paths match actual build configs
- [ ] Tag format/naming conventions match CI/CD workflow
- [ ] Multi-arch build claims match actual platform targets
- [ ] Build commands documented actually work

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] Every health endpoint documented has been traced to its handler function
- [ ] Every health check response field has been verified against the response struct
- [ ] Every Prometheus metric has been verified: name, type, labels, help text
- [ ] A complete inventory of undocumented metrics (if any) is recorded
- [ ] Container image names, registries, and tag formats verified against build configs
- [ ] All PromQL examples verified for correctness
- [ ] All Kubernetes probe YAML verified for correctness
- [ ] All findings recorded using verdict taxonomy
- [ ] All non-CORRECT findings fixed in documentation

## Findings Report

### health-endpoints.md

| Line/Section | Claim | Verdict | Source File | Notes |
|---|---|---|---|---|
| L24-47 | Broker `/healthz` returns 200 OK with "OK" | CORRECT | `crates/brokkr-broker/src/api/mod.rs:238-239` | Exact match |
| L48-71 | Broker `/readyz` returns 200 OK with "Ready" | CORRECT | `crates/brokkr-broker/src/api/mod.rs:250-251` | Exact match |
| L73-75 | Broker has no detailed `/health` endpoint | CORRECT | `crates/brokkr-broker/src/api/mod.rs:200-204` | Only `/healthz`, `/readyz`, `/metrics` registered |
| L80 | Agent health endpoints on port 8080 | CORRECT | `crates/brokkr-agent/src/health.rs`, `docker/Dockerfile.agent:98` | EXPOSE 8080 confirmed |
| L82-103 | Agent `/healthz` returns 200 OK with "OK" | CORRECT | `crates/brokkr-agent/src/health.rs:93-94` | Exact match |
| L105-134 | Agent `/readyz` returns 200/503, checks K8s API | CORRECT | `crates/brokkr-agent/src/health.rs:101-112` | Uses `apiserver_version()` check |
| L136-218 | Agent `/health` JSON structure, fields, status logic | CORRECT | `crates/brokkr-agent/src/health.rs:54-61,125-182` | All fields match: status, kubernetes, broker, uptime_seconds, version, timestamp |
| L156-170 | Healthy response JSON example | CORRECT | `crates/brokkr-agent/src/health.rs:156-164` | Status logic: healthy when k8s_connected AND broker_connected |
| L174-191 | Unhealthy K8s response includes error field | CORRECT | `crates/brokkr-agent/src/health.rs:65-69` | `kubernetes.error` uses `skip_serializing_if = "Option::is_none"` |
| L195-208 | Unhealthy broker response omits last_heartbeat when None | CORRECT | `crates/brokkr-agent/src/health.rs:73-76` | `broker.last_heartbeat` uses `skip_serializing_if = "Option::is_none"` |
| L222-257 | Broker Helm probe config (paths, ports, thresholds) | CORRECT | `charts/brokkr-broker/templates/deployment.yaml:39-54` | All values match exactly |
| L269-304 | Agent Helm probe config (paths, ports, thresholds) | CORRECT | `charts/brokkr-agent/templates/deployment.yaml:41-55` | All values match exactly |

### monitoring.md

| Line/Section | Claim | Verdict | Source File | Notes |
|---|---|---|---|---|
| L16 | Broker metrics at `:3000/metrics` | CORRECT | `crates/brokkr-broker/src/api/mod.rs:204` | Route registered |
| L22 | Agent metrics at `:8080/metrics` | CORRECT | `crates/brokkr-agent/src/health.rs:85` | Route registered |
| L30-49 | `brokkr_http_requests_total` Counter, labels: endpoint/method/status | CORRECT | `crates/brokkr-broker/src/metrics.rs:23-34` | Name, type, labels, help text all match |
| L51-69 | `brokkr_http_request_duration_seconds` Histogram, labels: endpoint/method, buckets | CORRECT | `crates/brokkr-broker/src/metrics.rs:38-52` | Buckets: 0.001,0.005,0.01,0.025,0.05,0.1,0.25,0.5,1.0,2.5,5.0,10.0 match |
| L73-86 | `brokkr_database_queries_total` Counter, labels: query_type | CORRECT | `crates/brokkr-broker/src/metrics.rs:56-67` | Exact match |
| L88-101 | `brokkr_database_query_duration_seconds` Histogram, labels: query_type, buckets | CORRECT | `crates/brokkr-broker/src/metrics.rs:71-85` | Buckets: 0.001,0.005,0.01,0.025,0.05,0.1,0.25,0.5,1.0,2.5,5.0 match |
| L105-117 | `brokkr_active_agents` Gauge, no labels | CORRECT | `crates/brokkr-broker/src/metrics.rs:88-95` | IntGauge, no labels |
| L119-133 | `brokkr_agent_heartbeat_age_seconds` Gauge, labels: agent_id/agent_name | CORRECT | `crates/brokkr-broker/src/metrics.rs:99-110` | GaugeVec with matching labels |
| L135-138 | `brokkr_stacks_total` Gauge, no labels | CORRECT | `crates/brokkr-broker/src/metrics.rs:113-120` | IntGauge |
| L140-143 | `brokkr_deployment_objects_total` Gauge, no labels | CORRECT | `crates/brokkr-broker/src/metrics.rs:123-133` | IntGauge |
| L149-166 | `brokkr_agent_poll_requests_total` Counter, labels: status | CORRECT | `crates/brokkr-agent/src/metrics.rs:27-41` | CounterVec with label "status" |
| L168-180 | `brokkr_agent_poll_duration_seconds` Histogram, no labels, buckets | CORRECT | `crates/brokkr-agent/src/metrics.rs:44-59` | HistogramVec with empty labels `&[]`, buckets match |
| L184-197 | `brokkr_agent_kubernetes_operations_total` Counter, labels: operation | CORRECT | `crates/brokkr-agent/src/metrics.rs:63-77` | CounterVec with label "operation" |
| L199-212 | `brokkr_agent_kubernetes_operation_duration_seconds` Histogram, labels: operation, buckets | CORRECT | `crates/brokkr-agent/src/metrics.rs:81-96` | Buckets: 0.01,0.05,0.1,0.25,0.5,1.0,2.5,5.0,10.0 match |
| L216-225 | `brokkr_agent_heartbeat_sent_total` Counter, no labels | CORRECT | `crates/brokkr-agent/src/metrics.rs:99-112` | IntCounter, no labels |
| L227-239 | `brokkr_agent_last_successful_poll_timestamp` Gauge, no labels | CORRECT | `crates/brokkr-agent/src/metrics.rs:115-128` | Gauge, no labels |
| L339-433 | Alerting rules use correct metric names and labels | CORRECT | All metric names verified above | All PromQL selectors reference valid metrics/labels |
| N/A | Undocumented metrics inventory | CORRECT | Checked all metrics in broker and agent | No undocumented metrics found - complete catalog |
| N/A | Dead documentation (metrics that don't exist in code) | CORRECT | Cross-referenced all doc entries against code | No phantom metrics |

### container-images.md

| Line/Section | Claim | Verdict | Source File | Notes |
|---|---|---|---|---|
| L16-21 | Image repos: broker, agent, UI at ghcr.io/colliery-io | CORRECT | `charts/*/values.yaml`, `.github/workflows/release.yml` | Repos match |
| L24-26 | Multi-arch: linux/amd64 and linux/arm64 | CORRECT | `.github/workflows/release.yml:28`, `build-and-test.yml:44` | Matrix builds both platforms |
| L34-47 | Semver tags use `v` prefix: `v1.2.3`, `v1.2`, `v1` | **INCORRECT** | `.github/workflows/release.yml:57-59` | CI uses `type=semver,pattern={{version}}` which strips `v` prefix. Actual tags: `1.2.3`, `1.2`, `1` (no `v`). Must remove `v` prefix from docs. |
| L41-47 | Example tags show `v1.2.3` etc. | **INCORRECT** | `.github/workflows/release.yml:57-59` | Same as above. Example should show `1.2.3`, `1.2`, `1`, `latest` |
| L53-60 | SHA tags: `sha-{short-sha}` | **INCORRECT** | `.github/workflows/build-and-test.yml:83` | Build-and-test creates `{branch}-sha-{short-sha}` tags. Release workflow does not create SHA tags. Docs should clarify branch-prefixed format. |
| L66-74 | Branch tags: `{branch-name}` | CORRECT | `.github/workflows/build-and-test.yml:70-74` | Branch name used as tag for non-PR builds |
| L80-87 | PR tags: `pr-{number}` | CORRECT | `.github/workflows/build-and-test.yml:71` | PR builds use `pr-{number}` |
| L106-147 | Build commands: `angreal build multi-arch` with params | CORRECT | `.angreal/task_build.py:74-81` | All parameters match: component, --tag, --registry, --platforms, --push |
| L221-227 | Broker/Agent: 4-stage build (planner, cacher, builder, final Debian slim) | CORRECT | `docker/Dockerfile.broker`, `docker/Dockerfile.agent` | Both use 4 stages with debian:bookworm-slim final |
| L229-231 | UI: "Builder stage: npm build, Final stage: Node.js runtime" (multi-stage) | **INCORRECT** | `docker/Dockerfile.ui` | Single-stage build using `node:18-alpine`. No multi-stage. Docs should say: "Single stage: Node.js Alpine with npm install and start" |

### deployment-health.md

| Line/Section | Claim | Verdict | Source File | Notes |
|---|---|---|---|---|
| L18-25 | Health status values: healthy, degraded, failing, unknown | CORRECT | `crates/brokkr-models/src/models/deployment_health.rs:39-48` | VALID_HEALTH_STATUSES exact match |
| L31-45 | Detected conditions list | CORRECT | `crates/brokkr-agent/src/deployment_health.rs:22-46` | DEGRADED_CONDITIONS + TERMINATED_ISSUES + PodFailed all verified |
| L52-66 | Config via env vars and Helm values | CORRECT | `charts/brokkr-agent/values.yaml:73-78` | `agent.deploymentHealth.enabled` and `intervalSeconds` match |
| L107-125 | API response for `/deployment-objects/{id}/health` | **INCORRECT** | `crates/brokkr-broker/src/api/v1/health.rs:58-66` | Documented response shows flat object with `deployment_object_id`, `agent_id`, `status`, `summary`, `checked_at`. Actual `DeploymentHealthResponse` returns `deployment_object_id`, `health_records` (array of DeploymentHealth), and `overall_status`. The response is a wrapper with multiple agent records, not a single flat record. |
| L130-153 | Health summary fields: pods_ready, pods_total, conditions, resources | CORRECT | `crates/brokkr-models/src/models/deployment_health.rs:185-195` | HealthSummary struct matches |
| L219-221 | Multi-agent query with `?all_agents=true` param | **UNVERIFIABLE** | `crates/brokkr-broker/src/api/v1/health.rs:201-243` | The `get_deployment_health` handler does not accept query parameters. It always returns all health records (all agents). The `?all_agents=true` parameter does not exist in code. The docs should remove this claim since the endpoint always returns all agent records. |

## Summary of Errors Found

### Must Fix (5 errors):

1. **container-images.md L34-47**: Semver tag format incorrectly shows `v` prefix. CI produces `1.2.3`, not `v1.2.3`. Fix: remove `v` from tag format column, examples column, and example block.

2. **container-images.md L53-60**: SHA tag format documented as `sha-{short-sha}` but CI creates `{branch}-sha-{short-sha}`. Fix: update format and example.

3. **container-images.md L229-231**: UI image described as multi-stage build. Actual `Dockerfile.ui` is single-stage `node:18-alpine`. Fix: describe as single-stage Node.js Alpine.

4. **deployment-health.md L107-125**: API response structure is wrong. Documented as flat object with `agent_id`/`status`/`summary`/`checked_at`. Actual response is `DeploymentHealthResponse` with `deployment_object_id`, `health_records` (array), `overall_status`. Fix: update example response.

5. **deployment-health.md L219-221**: `?all_agents=true` query parameter does not exist. The endpoint always returns all agent health records. Fix: remove the query parameter from the example URL or note that all agents are always returned.

## Status Updates

- 2026-03-13: Task activated. Full verification completed across all 4 doc files.
- 2026-03-13: Found 5 errors: 3 in container-images.md, 2 in deployment-health.md. health-endpoints.md and monitoring.md fully correct.
- 2026-03-13: All health endpoints verified against handler code (12 checks, 12 CORRECT).
- 2026-03-13: All 14 Prometheus metrics verified character-by-character (name, type, labels, help, buckets). Complete inventory: 0 undocumented, 0 phantom.
- 2026-03-13: All alerting PromQL verified against metric names and label selectors.
- 2026-03-13: Helm probe configs verified against actual chart templates (exact match).
- 2026-03-13: Attempting doc fixes (Edit/Bash tools denied - fixes documented but must be applied manually).
