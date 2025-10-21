---
id: add-monitoring-and-observability
level: task
title: "Add monitoring and observability configuration"
short_code: "BROKKR-T-0019"
created_at: 2025-10-21T12:37:06.292444+00:00
updated_at: 2025-10-21T12:37:06.292444+00:00
parent: BROKKR-I-0003
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
strategy_id: NULL
initiative_id: BROKKR-I-0003
---

# Add monitoring and observability configuration

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[BROKKR-I-0003]]

## Objective **[REQUIRED]**

Implement OpenTelemetry-based observability with Prometheus-compatible metrics endpoints for both broker and agent components. Enable vendor-agnostic monitoring that supports multiple observability backends while maintaining backward compatibility with Prometheus tooling.

**See ADR-3** for the architectural decision to use OpenTelemetry instead of Prometheus-only metrics.



## Acceptance Criteria **[REQUIRED]**

**Core Implementation:**
- [ ] OpenTelemetry SDK integrated into broker and agent
- [ ] Prometheus-compatible `/metrics` endpoint in broker (replace stub at port 3000)
- [ ] Prometheus-compatible `/metrics` endpoint in agent (port 8080)
- [ ] Metrics export configurable (can be disabled via configuration)

**Instrumentation:**
- [ ] Broker metrics: HTTP requests (count, duration), database queries (count, duration), active agents, heartbeat age
- [ ] Agent metrics: Poll requests (count, duration), Kubernetes operations (count, duration)
- [ ] Metrics follow Prometheus naming conventions (snake_case, unit suffixes)
- [ ] Cardinality kept low (no agent IDs or request IDs in labels)

**Kubernetes Integration:**
- [ ] ServiceMonitor CRD templates added to Helm charts (optional via values)
- [ ] Helm values support enabling/disabling metrics export
- [ ] ServiceMonitor scrape interval configurable

**Documentation:**
- [ ] Complete metrics catalog in `docs/content/reference/monitoring.md`
- [ ] Example Prometheus scrape configurations (for non-operator users)
- [ ] ServiceMonitor configuration examples
- [ ] Documented metric types, labels, and meanings
- [ ] Note on vendor-agnostic design (users can export to other backends in future)



## Implementation Notes **[CONDITIONAL: Technical Task]**

### Technical Approach

**Architecture Decision:**
This task implements **Phase 1 of ADR-3**: OpenTelemetry with Prometheus export. This provides:
- Prometheus-compatible `/metrics` endpoint (maintains backward compatibility)
- Foundation for future OTLP export (Phase 2)
- Foundation for distributed tracing (Phase 3)

**Code Changes:**

1. **Add OpenTelemetry Dependencies:**
   Add to `Cargo.toml` workspace dependencies:
   ```toml
   opentelemetry = "0.22"
   opentelemetry_sdk = { version = "0.22", features = ["rt-tokio"] }
   opentelemetry-prometheus = "0.15"
   prometheus = "0.13"
   ```

2. **Replace Broker `/metrics` Stub:**
   Current stub at `broker/src/api/mod.rs:214` returns "Metrics" string.
   Replace with OpenTelemetry Prometheus exporter that returns actual metrics.

3. **Add Agent `/metrics` Endpoint:**
   Agent currently has no HTTP server. Add minimal HTTP server for health + metrics on port 8080.

4. **Broker Metrics to Implement:**
   Using OpenTelemetry instruments that export to Prometheus format:
   - `brokkr_http_requests_total{method, endpoint, status}` (counter)
   - `brokkr_http_request_duration_seconds{method, endpoint}` (histogram)
   - `brokkr_database_queries_total{operation}` (counter) - operation: select, insert, update, delete
   - `brokkr_database_query_duration_seconds{operation}` (histogram)
   - `brokkr_active_agents` (gauge) - Count of agents with recent heartbeats
   - `brokkr_stacks_total` (gauge)
   - `brokkr_deployment_objects_total` (gauge)

   **Note:** Avoid high-cardinality labels like agent IDs or request IDs.

5. **Agent Metrics to Implement:**
   - `brokkr_agent_poll_requests_total{result}` (counter) - result: success, error, empty
   - `brokkr_agent_poll_duration_seconds` (histogram)
   - `brokkr_agent_kubernetes_operations_total{operation, result}` (counter) - operation: apply, delete, get, etc.
   - `brokkr_agent_kubernetes_operation_duration_seconds{operation}` (histogram)
   - `brokkr_agent_heartbeat_sent_total` (counter)
   - `brokkr_agent_last_successful_poll_timestamp` (gauge) - Unix timestamp

**Helm Chart Changes:**

Add optional ServiceMonitor template to both charts:

```yaml
# charts/brokkr-broker/templates/servicemonitor.yaml
{{- if .Values.metrics.serviceMonitor.enabled }}
apiVersion: monitoring.coreos.com/v1
kind: ServiceMonitor
metadata:
  name: {{ include "brokkr-broker.fullname" . }}
  labels:
    {{- include "brokkr-broker.labels" . | nindent 4 }}
spec:
  selector:
    matchLabels:
      {{- include "brokkr-broker.selectorLabels" . | nindent 6 }}
  endpoints:
  - port: http
    path: /metrics
    interval: {{ .Values.metrics.serviceMonitor.interval }}
{{- end }}
```

Add to values.yaml:
```yaml
metrics:
  enabled: true
  serviceMonitor:
    enabled: false
    interval: 30s
```

**Configuration:**

Add configuration options to enable/disable metrics:

Broker `config.toml`:
```toml
[metrics]
enabled = true
# Future: otlp_endpoint = "http://otel-collector:4317"
```

Agent configuration (environment variables or config file):
```toml
[metrics]
enabled = true
```

**Documentation:**

Create `docs/content/reference/monitoring.md` with:

1. **Introduction:**
   - OpenTelemetry-based observability (reference ADR-3)
   - Vendor-agnostic design
   - Current support: Prometheus-compatible metrics
   - Future roadmap: OTLP export, distributed tracing

2. **Metrics Catalog:**
   - Complete list of all metrics with:
     - Metric name
     - Type (counter, histogram, gauge)
     - Description
     - Labels and their values
     - Example PromQL queries

3. **Prometheus Integration:**
   - Manual scrape configuration (for non-operator users)
   - ServiceMonitor configuration (for Prometheus Operator users)
   - Recommended scrape intervals
   - Example alerting rules:
     - High HTTP error rate
     - Agent disconnected (no recent heartbeat)
     - Database query latency spike
     - Kubernetes operation failures

4. **Multi-Backend Support:**
   - Note that OpenTelemetry enables future support for:
     - Datadog (via OTLP export - future)
     - New Relic (via OTLP export - future)
     - Honeycomb (via OTLP export - future)
     - Any OpenTelemetry-compatible backend
   - Current Phase 1: Prometheus only
   - Users can request OTLP support if needed

### Dependencies

**Rust Crates:**
- `opentelemetry` - Core OpenTelemetry API
- `opentelemetry_sdk` - OpenTelemetry SDK with Tokio runtime
- `opentelemetry-prometheus` - Prometheus exporter for OpenTelemetry
- `prometheus` - Prometheus client library (used by opentelemetry-prometheus)

**Design Dependencies:**
- ADR-3: OpenTelemetry for Vendor-Agnostic Observability (approved)

**Existing Infrastructure:**
- Helm charts exist (can add ServiceMonitor templates)
- Broker has stub `/metrics` endpoint (will be replaced)
- Agent has health checks (will add HTTP server for metrics)

**No blocking dependencies** - can implement incrementally

### Risk Considerations

**Risk: OpenTelemetry adds complexity and dependencies**
- Mitigation: Phase 1 only adds Prometheus export (familiar to users)
- Future phases (OTLP, tracing) are optional additions
- Document that Phase 1 is production-ready without future phases
- Performance: OTel SDK overhead estimated at 1-2% CPU, 50-100MB memory

**Risk: Too many metrics causing cardinality issues**
- Mitigation: Limit high-cardinality labels (no agent IDs, request IDs, etc.)
- Use aggregated gauges instead of per-item metrics where possible
- Follow Prometheus best practices for label design
- Document cardinality considerations for users

**Risk: ServiceMonitor requires Prometheus Operator**
- Mitigation: Make ServiceMonitor optional in Helm values (disabled by default)
- Provide manual Prometheus scrape configs as alternative
- Document both installation methods clearly
- Most users either have operator or can use manual config

**Risk: Users may expect OTLP/tracing immediately**
- Mitigation: Clearly document this is Phase 1 (Prometheus only)
- Mention future roadmap in documentation
- Provide issue template for users to request OTLP support
- ADR-3 sets expectations for phased rollout

**Risk: Breaking changes to metric names/labels in future**
- Mitigation: Follow OpenTelemetry semantic conventions from start
- Use standard Prometheus naming conventions (snake_case, unit suffixes)
- Version metrics documentation
- Note that metrics are subject to change in pre-1.0 releases
- OpenTelemetry stability guarantees apply once 1.0 is reached

## Status Updates **[REQUIRED]**

### 2025-10-21: Task Revised for OpenTelemetry

Updated task to reflect ADR-3 decision to use OpenTelemetry instead of Prometheus-only metrics.

**Key Changes:**
- Objective updated: OpenTelemetry with Prometheus-compatible export
- Dependencies changed: Added OTel crates instead of just Prometheus
- Acceptance criteria refined: Focus on Phase 1 (Prometheus export)
- Removed Grafana dashboard requirement (not critical for Phase 1)
- Added configuration section for enabling/disabling metrics
- Updated documentation requirements to mention vendor-agnostic design
- Risk considerations updated for OTel approach

**Implementation Approach:**
This task now implements **Phase 1 of ADR-3**:
- OpenTelemetry SDK integration
- Prometheus-compatible `/metrics` endpoint
- Foundation for future OTLP export (Phase 2)
- Foundation for distributed tracing (Phase 3)

**Scope Clarification:**
- Grafana dashboards: Removed from this task (users can build their own)
- Alerting rules: Included as examples in documentation only
- Multi-backend support: Documented as future roadmap, not implemented yet

Ready for implementation once approved.
