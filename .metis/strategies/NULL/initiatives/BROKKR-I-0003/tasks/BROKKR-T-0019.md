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

Implement Prometheus metrics endpoints, ServiceMonitor CRDs, and provide example Grafana dashboards and monitoring configurations for both broker and agent components. Enable production-ready observability for Brokkr deployments.



## Acceptance Criteria **[REQUIRED]**

- [ ] Prometheus `/metrics` endpoint implemented in broker (port 3000)
- [ ] Prometheus `/metrics` endpoint implemented in agent (port 8080)
- [ ] ServiceMonitor CRDs created for Prometheus Operator integration
- [ ] ServiceMonitor templates added to Helm charts (optional/configurable)
- [ ] Example Grafana dashboard JSON for broker metrics
- [ ] Example Grafana dashboard JSON for agent metrics
- [ ] Documentation of key metrics and what they measure
- [ ] Example monitoring configurations (Prometheus scrape configs, alerting rules)
- [ ] Metrics include: request count, latency, error rates, queue depth, resource usage
- [ ] Documentation in `docs/content/reference/monitoring.md`



## Implementation Notes **[CONDITIONAL: Technical Task]**

### Technical Approach

**Code Changes:**

1. **Add Prometheus Dependency:**
   - Use `prometheus` crate in Rust for metrics
   - Add `/metrics` endpoint to both broker and agent HTTP servers

2. **Broker Metrics to Implement:**
   - `brokkr_http_requests_total` (counter) - Total HTTP requests by endpoint and status
   - `brokkr_http_request_duration_seconds` (histogram) - Request latency distribution
   - `brokkr_database_queries_total` (counter) - Database query count
   - `brokkr_database_query_duration_seconds` (histogram) - DB query latency
   - `brokkr_active_agents` (gauge) - Number of connected agents
   - `brokkr_agent_heartbeat_age_seconds` (gauge) - Time since last heartbeat per agent
   - `brokkr_stacks_total` (gauge) - Total number of stacks
   - `brokkr_deployment_objects_total` (gauge) - Total deployment objects

3. **Agent Metrics to Implement:**
   - `brokkr_agent_poll_requests_total` (counter) - Total broker poll requests
   - `brokkr_agent_poll_duration_seconds` (histogram) - Poll request latency
   - `brokkr_agent_kubernetes_operations_total` (counter) - K8s API operations by type
   - `brokkr_agent_kubernetes_operation_duration_seconds` (histogram) - K8s operation latency
   - `brokkr_agent_heartbeat_sent_total` (counter) - Total heartbeats sent
   - `brokkr_agent_last_successful_poll_timestamp` (gauge) - Unix timestamp of last successful poll

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

**Grafana Dashboards:**

Create example dashboards:
- `docs/grafana/brokkr-broker-dashboard.json`
- `docs/grafana/brokkr-agent-dashboard.json`

Dashboard panels to include:
- Request rate and error rate
- Request latency (p50, p95, p99)
- Database query performance
- Active agents and heartbeat status
- Resource usage (from kube-state-metrics)

**Documentation:**

Create `docs/content/reference/monitoring.md` with:
- Complete metrics catalog (name, type, description, labels)
- Prometheus scrape configuration examples
- ServiceMonitor configuration
- Example alerting rules (high error rate, agent disconnected, etc.)
- Grafana dashboard installation instructions
- Integration with other monitoring systems (Datadog, New Relic)

### Dependencies

- Requires adding `prometheus` crate to Cargo.toml
- Helm charts already exist (can add ServiceMonitor templates)
- No blocking dependencies - can implement incrementally

### Risk Considerations

**Risk: Metrics implementation adds overhead**
- Mitigation: Use Prometheus efficient counters/histograms
- Make metrics optional via feature flag if needed
- Document performance impact

**Risk: Too many metrics causing cardinality issues**
- Mitigation: Limit high-cardinality labels (avoid agent IDs in labels)
- Use standard Prometheus practices
- Document recommended metric retention

**Risk: ServiceMonitor requires Prometheus Operator**
- Mitigation: Make ServiceMonitor optional in Helm values
- Provide manual Prometheus scrape configs as alternative
- Document both installation methods

**Risk: Grafana dashboards may not work for all Prometheus setups**
- Mitigation: Use standard PromQL queries
- Test dashboards against vanilla Prometheus
- Provide templated dashboards that work with variables

**Risk: Breaking changes to metric names/labels**
- Mitigation: Version metrics documentation
- Follow Prometheus naming conventions from start
- Note that metrics are subject to change in pre-1.0 releases

## Status Updates **[REQUIRED]**

*To be added during implementation*
