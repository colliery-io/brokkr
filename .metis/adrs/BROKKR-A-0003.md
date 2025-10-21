---
id: 003-opentelemetry-for-observability
level: adr
title: "OpenTelemetry for Vendor-Agnostic Observability"
number: 3
short_code: "BROKKR-A-0003"
created_at: 2025-10-21T17:30:00.000000+00:00
updated_at: 2025-10-21T17:30:00.000000+00:00
decision_date: 2025-10-21
decision_maker: Dylan Storey
parent:
archived: false

tags:
  - "#adr"
  - "#phase/draft"


exit_criteria_met: false
strategy_id: NULL
initiative_id: BROKKR-I-0003
---

# ADR-3: OpenTelemetry for Vendor-Agnostic Observability

## Context **[REQUIRED]**

Brokkr needs observability instrumentation to provide metrics, distributed tracing, and logging capabilities for both broker and agent components. The choice of observability framework affects:

**Deployment topology challenges:**
- Broker is centrally deployed and accessible
- Agents are deployed in multiple Kubernetes clusters, potentially across:
  - Air-gapped environments
  - Highly restricted network topologies
  - Clusters behind firewalls
  - Multi-cloud or hybrid cloud deployments

**User flexibility requirements:**
- Users may have existing observability infrastructure:
  - Prometheus + Grafana
  - Datadog
  - New Relic
  - Honeycomb
  - Elastic APM
  - Custom solutions
- Forcing a specific monitoring backend is unacceptable
- "Bring your own observability tooling" is a core requirement

**Distributed system complexity:**
- Request flows span multiple components:
  ```
  User API → Broker → Database → Broker → Agent → Kubernetes API → Resources
  ```
- Understanding end-to-end latency requires distributed tracing
- Debugging failures across component boundaries is critical
- Metrics alone cannot capture causality chains

**Current state:**
- Broker has stub `/metrics` endpoint (returns "Metrics" string)
- Agent has no observability endpoints
- No instrumentation framework in place
- No dependency on observability libraries

## Decision **[REQUIRED]**

Implement observability using **OpenTelemetry (OTel)** with multiple export options, prioritizing vendor neutrality and flexibility over implementation simplicity.

**Architecture:**
1. **Instrumentation**: Use OpenTelemetry SDK for Rust
2. **Metrics export**: Support both Prometheus pull (via `/metrics` endpoint) and OTLP push
3. **Tracing export**: OTLP to collector or direct to backends
4. **Configuration**: Runtime-configurable exporters (users choose what to enable)
5. **Default mode**: Prometheus-compatible `/metrics` endpoint (most common expectation)

**Implementation approach:**
- **Phase 1**: Metrics with Prometheus export (BROKKR-T-0019)
  - Add `opentelemetry`, `opentelemetry-prometheus`, `opentelemetry_sdk` crates
  - Implement `/metrics` endpoint using Prometheus exporter
  - Instrument key operations (HTTP requests, DB queries, K8s operations)
  - Add ServiceMonitor CRD templates to Helm charts

- **Phase 2**: OTLP export support (future task)
  - Add `opentelemetry-otlp` crate
  - Configuration for OTLP endpoint
  - Support direct export to backends (Datadog, Honeycomb, etc.)

- **Phase 3**: Distributed tracing (future task)
  - Add trace spans to request flows
  - Propagate trace context broker → agent → K8s
  - Enable end-to-end request tracing

## Alternatives Analysis **[CONDITIONAL: Complex Decision]**

| Option | Pros | Cons | Vendor Lock-in | Network Requirements | Tracing Support |
|--------|------|------|----------------|---------------------|-----------------|
| **OpenTelemetry** (chosen) | Vendor-neutral; multiple backends; tracing + metrics; industry standard; future-proof | More complex implementation; larger dependency footprint | None | Flexible (push or pull) | Native |
| **Prometheus only** | Simple implementation; Kubernetes standard; pull-based familiar | Locked to Prometheus; no tracing; poor for restricted networks; metrics only | High | Pull requires access to all agents | None |
| **Prometheus + Jaeger** | Metrics + tracing; both CNCF projects | Two separate frameworks; duplicated instrumentation; no unified export | Medium | Pull (metrics) + Push (traces) | Via Jaeger |
| **Custom metrics API** | Full control; minimal dependencies | Reinventing the wheel; no ecosystem; users must build adapters | Extreme | Any | None |

## Rationale **[REQUIRED]**

OpenTelemetry was chosen because:

### 1. Vendor Neutrality (Critical Requirement)

Users can export to ANY observability backend:
```rust
// Configuration allows users to enable what they need
metrics_exporters = ["prometheus", "otlp"]  // Or just one
otlp_endpoint = "https://api.honeycomb.io"  // Or Datadog, or New Relic, etc.
```

This prevents vendor lock-in and respects existing user infrastructure investments.

### 2. Network Topology Flexibility

**Pull-based (Prometheus):**
```
Prometheus --> scrape --> Broker /metrics
          |-> scrape --> Agent /metrics (if accessible)
```
Works when Prometheus can reach all components.

**Push-based (OTLP):**
```
Broker --> OTLP push --> OTel Collector --> Backend
Agent  --> OTLP push --> OTel Collector --> Backend
```
Works when agents are in restricted networks - they push outbound.

**Hybrid:**
Users can mix approaches - broker pull, agents push.

### 3. Distributed Tracing Value

Request flow example:
```
POST /stacks/123/deployment-objects
  ├─ Broker: Parse request (2ms)
  ├─ Broker: Database write (15ms)
  ├─ Agent: Poll broker (1s polling interval)
  ├─ Agent: Apply to K8s (50ms)
  └─ K8s: Resource creation (200ms)
```

With OTel tracing:
- See the entire 1.267s end-to-end flow
- Identify that 1s is polling delay (expected)
- Identify if K8s API is slow
- Debug cross-component failures with correlated trace IDs

Prometheus metrics alone can't capture these causality chains.

### 4. Future-Proof Architecture

OpenTelemetry is the CNCF standard for observability:
- Backed by major vendors (Google, Microsoft, AWS, Datadog, etc.)
- Active development and community
- Converged from OpenTracing + OpenCensus
- Will outlive proprietary solutions

Choosing OTel now prevents future migration pain.

### 5. Incremental Implementation

We can implement in phases:
- **Now**: Prometheus-compatible metrics (meets immediate need)
- **Later**: Add OTLP export (enables more backends)
- **Future**: Add distributed tracing (advanced debugging)

This allows delivering value quickly while building toward comprehensive observability.

## Consequences **[REQUIRED]**

### Positive

**User Experience:**
- Users can use their existing monitoring tools
- No forced vendor choices
- Flexibility to change backends without code changes
- Standard exporters work out of the box

**Operational:**
- Agents in restricted networks can push metrics outbound
- No requirement for Prometheus to access every agent
- Centralized collection via OTel collector possible
- Multi-cluster deployments supported

**Debugging:**
- Distributed tracing enables end-to-end request visibility
- Correlated logs, metrics, and traces (future)
- Better understanding of performance bottlenecks
- Faster incident resolution

**Development:**
- Single instrumentation framework for metrics + traces + logs
- Industry standard patterns and best practices
- Good Rust ecosystem support (`opentelemetry-rust`)
- Extensive documentation and examples

### Negative

**Implementation Complexity:**
- More complex than Prometheus-only approach
- Larger dependency footprint (multiple OTel crates)
- Learning curve for team (OTel concepts vs just Prometheus)
- Configuration complexity (multiple exporters to support)

**Runtime Overhead:**
- Additional memory for metrics + traces (estimated 50-100MB per process)
- CPU overhead for exporting (estimated 1-2% with OTLP)
- Network traffic for push-based export
- Mitigation: All exporters are optional, can be disabled

**Testing Requirements:**
- Must test with multiple backends (Prometheus, OTLP, etc.)
- Integration testing more complex
- Documentation must cover multiple configurations
- Mitigation: Provide tested example configs for popular backends

### Migration Path

**Existing stub metrics endpoint:**
Current broker `/metrics` endpoint returns "Metrics" string. Plan:
1. Replace stub implementation with OTel Prometheus exporter
2. Maintain `/metrics` endpoint path (backward compatible)
3. Return proper Prometheus format output

**No breaking changes** - users will simply start getting real metrics.

## Implementation Plan **[CONDITIONAL: Complex Decision]**

### Phase 1: Prometheus-Compatible Metrics (BROKKR-T-0019)

**Crate dependencies:**
```toml
[workspace.dependencies]
opentelemetry = "0.22"
opentelemetry_sdk = { version = "0.22", features = ["rt-tokio"] }
opentelemetry-prometheus = "0.15"
prometheus = "0.13"
```

**Broker metrics:**
- `brokkr_http_requests_total` - Counter by endpoint, method, status
- `brokkr_http_request_duration_seconds` - Histogram for latency
- `brokkr_database_queries_total` - Counter by query type
- `brokkr_database_query_duration_seconds` - Histogram
- `brokkr_active_agents` - Gauge
- `brokkr_agent_heartbeat_age_seconds` - Gauge per agent

**Agent metrics:**
- `brokkr_agent_poll_requests_total` - Counter by result
- `brokkr_agent_poll_duration_seconds` - Histogram
- `brokkr_agent_kubernetes_operations_total` - Counter by operation type
- `brokkr_agent_kubernetes_operation_duration_seconds` - Histogram

**Helm chart updates:**
- Add ServiceMonitor template (optional via `metrics.serviceMonitor.enabled`)
- Add `metrics.enabled` flag to disable metrics entirely

**Timeline:** 2-3 days implementation + 1 day documentation

### Phase 2: OTLP Export Support (Future)

**Additional dependencies:**
```toml
opentelemetry-otlp = { version = "0.15", features = ["grpc-tonic"] }
```

**Configuration:**
```yaml
# Broker/Agent config
metrics:
  exporters:
    - prometheus  # Enable /metrics endpoint
    - otlp        # Enable OTLP push
  otlp:
    endpoint: "http://otel-collector:4317"
    protocol: grpc
```

**Timeline:** 1-2 days implementation

### Phase 3: Distributed Tracing (Future)

**Trace spans:**
- HTTP request handling (broker)
- Database operations (broker)
- Broker → Agent communication (both)
- Kubernetes API calls (agent)

**Context propagation:**
- W3C Trace Context headers in HTTP
- Trace ID in logs for correlation

**Timeline:** 3-4 days implementation + testing

## Metrics **[CONDITIONAL: Include If Measurable]**

**Success criteria:**
- [ ] Users can use Prometheus to scrape `/metrics` endpoints
- [ ] Users can configure OTLP export to their backends
- [ ] Metrics cardinality stays below 10,000 per instance
- [ ] Metrics export overhead < 2% CPU under load
- [ ] At least 3 different observability backends tested (Prometheus, Datadog, Honeycomb)
- [ ] Documentation covers common backend configurations

**Monitoring the decision:**
- GitHub issues for observability integration problems
- Community feedback on flexibility and usability
- Performance benchmarks with metrics enabled/disabled

## References **[CONDITIONAL: External Dependencies]**

**OpenTelemetry:**
- [OpenTelemetry Specification](https://opentelemetry.io/docs/specs/otel/)
- [OpenTelemetry Rust SDK](https://github.com/open-telemetry/opentelemetry-rust)
- [OTLP Protocol](https://opentelemetry.io/docs/specs/otlp/)

**Prometheus:**
- [Prometheus Metrics Naming](https://prometheus.io/docs/practices/naming/)
- [ServiceMonitor CRD](https://github.com/prometheus-operator/prometheus-operator/blob/main/Documentation/api.md#servicemonitor)

**Integration Examples:**
- [OTel with Datadog](https://docs.datadoghq.com/opentelemetry/)
- [OTel with Honeycomb](https://docs.honeycomb.io/getting-data-in/opentelemetry/)
- [OTel with New Relic](https://docs.newrelic.com/docs/more-integrations/open-source-telemetry-integrations/opentelemetry/opentelemetry-introduction/)

## Status Updates **[REQUIRED]**

### 2025-10-21: Draft Created

ADR drafted to document the decision to use OpenTelemetry for observability instead of Prometheus-only approach. Key decision drivers:
- Vendor neutrality requirement (users bring their own tools)
- Network topology flexibility (agents in restricted environments)
- Distributed tracing value for debugging
- Future-proof architecture

Next steps:
- Review and approve ADR
- Update BROKKR-T-0019 to reflect OTel implementation
- Begin Phase 1 implementation
