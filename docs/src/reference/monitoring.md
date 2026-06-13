# Monitoring and Observability

Brokkr provides comprehensive Prometheus metrics for monitoring both the broker and agent components. This reference catalogs the available metrics and the shipped Grafana dashboards. For scrape configuration, alerting rules, dashboard import, and integration with external monitoring systems, see [Setting Up Monitoring](../how-to/monitoring-setup.md).

## Metrics Endpoints

Both broker and agent expose Prometheus metrics in standard text exposition format.

### Broker Metrics

**Endpoint:** `http://<broker-host>:3000/metrics`

The broker exposes metrics about HTTP requests, database operations, and system state.

### Agent Metrics

**Endpoint:** `http://<agent-host>:8080/metrics`

The agent exposes metrics about broker polling, Kubernetes operations, and agent health.

## Broker Metrics Catalog

### HTTP Request Metrics

#### `brokkr_http_requests_total`
- **Type:** Counter
- **Description:** Total number of HTTP requests by endpoint and status
- **Labels:**
  - `endpoint` - API endpoint path
  - `method` - HTTP method (GET, POST, PUT, DELETE)
  - `status` - HTTP status code (200, 404, 500, etc.)

**Example PromQL:**
```promql
# Request rate by endpoint
rate(brokkr_http_requests_total[5m])

# Error rate (4xx and 5xx)
sum(rate(brokkr_http_requests_total{status=~"[45].."}[5m])) by (endpoint)

# Success rate percentage
100 * sum(rate(brokkr_http_requests_total{status=~"2.."}[5m])) by (endpoint)
  / sum(rate(brokkr_http_requests_total[5m])) by (endpoint)
```

#### `brokkr_http_request_duration_seconds`
- **Type:** Histogram
- **Description:** HTTP request latency distribution in seconds
- **Labels:**
  - `endpoint` - API endpoint path
  - `method` - HTTP method
- **Buckets:** 0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0 seconds

**Example PromQL:**
```promql
# 99th percentile latency
histogram_quantile(0.99,
  sum(rate(brokkr_http_request_duration_seconds_bucket[5m])) by (le, endpoint)
)

# Average latency
rate(brokkr_http_request_duration_seconds_sum[5m])
  / rate(brokkr_http_request_duration_seconds_count[5m])
```

### System State Metrics

#### `brokkr_active_agents`
- **Type:** Gauge
- **Description:** Number of currently active agents
- **Labels:** None

**Example PromQL:**
```promql
# Current active agents
brokkr_active_agents

# Alert if no agents connected
brokkr_active_agents == 0
```

#### `brokkr_agent_heartbeat_age_seconds`
- **Type:** Gauge
- **Description:** Time since last heartbeat per agent in seconds
- **Labels:**
  - `agent_id` - Agent UUID
  - `agent_name` - Human-readable agent name
- **Freshness:** A broker background task refreshes this gauge (and
  `brokkr_active_agents`) from the database every ~30 seconds, so the values
  stay correct without depending on `GET /agents` traffic.

**Example PromQL:**
```promql
# Agents with stale heartbeats (>5 minutes)
brokkr_agent_heartbeat_age_seconds > 300

# Maximum heartbeat age across all agents
max(brokkr_agent_heartbeat_age_seconds)
```

#### `brokkr_stacks_total`
- **Type:** Gauge
- **Description:** Total number of stacks
- **Labels:** None

#### `brokkr_deployment_objects_total`
- **Type:** Gauge
- **Description:** Total number of deployment objects
- **Labels:** None

### WebSocket Channel Metrics

These metrics cover the internal broker↔agent WebSocket channel and the live telemetry fan-out (see [Internal Broker↔Agent WS Channel](../explanation/internal-ws-channel.md)). Labels are deliberately low-cardinality; per-connection detail is available from `GET /api/v1/admin/ws/connections` instead.

#### `brokkr_ws_connected_agents`
- **Type:** Gauge
- **Description:** Number of agents currently connected on `/internal/ws/agent`
- **Labels:** None

#### `brokkr_ws_messages_total`
- **Type:** Counter
- **Description:** WebSocket messages processed by the broker
- **Labels:**
  - `direction` - `in` (from agents) or `out` (to agents)
  - `type` - Wire message variant (e.g. `pod_log_line`, `k8s_event`, `heartbeat`, `work_order`)

**Example PromQL:**
```promql
# Telemetry ingest rate by message type
sum(rate(brokkr_ws_messages_total{direction="in"}[5m])) by (type)
```

#### `brokkr_ws_live_subscribers`
- **Type:** Gauge
- **Description:** Total live-tail subscribers across all stacks (`GET /api/v1/stacks/{id}/live`)
- **Labels:** None

#### `brokkr_ws_log_eviction_runs_total`
- **Type:** Counter
- **Description:** Telemetry eviction passes executed by the retention worker
- **Labels:** None

#### `brokkr_ws_telemetry_evicted_total`
- **Type:** Counter
- **Description:** Telemetry rows deleted by the retention worker (6-hour hard ceiling)
- **Labels:**
  - `table` - `agent_k8s_events` or `agent_pod_logs`

## Agent Metrics Catalog

### Broker Polling Metrics

#### `brokkr_agent_poll_requests_total`
- **Type:** Counter
- **Description:** Total number of broker poll requests
- **Labels:**
  - `status` - Request status (success, error)

**Example PromQL:**
```promql
# Poll request rate
rate(brokkr_agent_poll_requests_total[5m])

# Error rate
rate(brokkr_agent_poll_requests_total{status="error"}[5m])

# Success rate percentage
100 * sum(rate(brokkr_agent_poll_requests_total{status="success"}[5m]))
  / sum(rate(brokkr_agent_poll_requests_total[5m]))
```

#### `brokkr_agent_poll_duration_seconds`
- **Type:** Histogram
- **Description:** Broker poll request latency distribution in seconds
- **Labels:** None
- **Buckets:** 0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0, 30.0 seconds

**Example PromQL:**
```promql
# 99th percentile poll latency
histogram_quantile(0.99,
  sum(rate(brokkr_agent_poll_duration_seconds_bucket[5m])) by (le)
)
```

### Kubernetes Operation Metrics

#### `brokkr_agent_kubernetes_operations_total`
- **Type:** Counter
- **Description:** Total number of Kubernetes API operations by type
- **Labels:**
  - `operation` - Operation type (apply, delete, get, list)

**Example PromQL:**
```promql
# K8s operation rate by type
rate(brokkr_agent_kubernetes_operations_total[5m])

# Total K8s operations per second
sum(rate(brokkr_agent_kubernetes_operations_total[5m]))
```

#### `brokkr_agent_kubernetes_operation_duration_seconds`
- **Type:** Histogram
- **Description:** Kubernetes operation latency distribution in seconds
- **Labels:**
  - `operation` - Operation type
- **Buckets:** 0.01, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0, 10.0 seconds

**Example PromQL:**
```promql
# 95th percentile K8s operation latency
histogram_quantile(0.95,
  sum(rate(brokkr_agent_kubernetes_operation_duration_seconds_bucket[5m])) by (le, operation)
)
```

### Agent Health Metrics

#### `brokkr_agent_heartbeat_sent_total`
- **Type:** Counter
- **Description:** Total number of heartbeats sent to broker
- **Labels:** None

**Example PromQL:**
```promql
# Heartbeat send rate
rate(brokkr_agent_heartbeat_sent_total[5m])
```

#### `brokkr_agent_last_successful_poll_timestamp`
- **Type:** Gauge
- **Description:** Unix timestamp of last successful broker poll
- **Labels:** None

**Example PromQL:**
```promql
# Time since last successful poll
time() - brokkr_agent_last_successful_poll_timestamp

# Alert if no successful poll in 5 minutes
time() - brokkr_agent_last_successful_poll_timestamp > 300
```

## Grafana Dashboards

Brokkr ships pre-built Grafana dashboards in the repository under `docs/grafana/`:

| Dashboard | File |
|-----------|------|
| Broker | `brokkr-broker-dashboard.json` |
| Agent | `brokkr-agent-dashboard.json` |
| WebSocket channel | `brokkr-ws-channel-dashboard.json` |

Prometheus alert rules for the WebSocket channel ship alongside the dashboards as `docs/grafana/brokkr-ws-channel.rules.yml`. For download and import steps, see [Setting Up Monitoring](../how-to/monitoring-setup.md#import-grafana-dashboards).

### Broker Dashboard Features

The broker dashboard includes:

- **Active Agents** - Current count of connected agents
- **Total Stacks** - Number of managed stacks
- **Deployment Objects** - Total deployment objects
- **HTTP Request Rate** - Requests per second by endpoint
- **HTTP Request Latency** - p50, p95, p99 latencies by endpoint
- **Agent Heartbeat Age** - Time since last heartbeat per agent

### Agent Dashboard Features

The agent dashboard includes:

- **Broker Poll Request Rate** - Success/error poll rates
- **Broker Poll Latency** - p50, p95, p99 poll latencies
- **Kubernetes Operations Rate** - Operations per second by type
- **Kubernetes Operation Latency** - p50, p95, p99 operation latencies
- **Heartbeat Send Rate** - Heartbeats sent per second
- **Time Since Last Successful Poll** - Gauge showing polling health

## Performance Impact

Metrics collection has minimal performance overhead:

- **CPU:** <1% per component
- **Memory:** ~10MB for metrics registry
- **Network:** ~5KB per scrape (30s intervals = ~170KB/min)

Metrics are collected lazily and only computed when scraped by Prometheus.

## Related Documentation

- [Setting Up Monitoring](../how-to/monitoring-setup.md) - Prometheus scrape configuration, alerting rules, dashboard import, external integrations
- [Installation Guide](../getting-started/installation.md) - Helm chart installation
- [Health Check Endpoints](./health-endpoints.md) - Liveness and readiness probes
- [Configuration Guide](../getting-started/configuration.md) - Configuration options
