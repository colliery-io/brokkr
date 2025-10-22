---
title: Monitoring & Observability
weight: 30
---

# Monitoring and Observability

Brokkr provides comprehensive Prometheus metrics for monitoring both the broker and agent components. This guide covers available metrics, configuration options, and example dashboards.

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

### Database Metrics

#### `brokkr_database_queries_total`
- **Type:** Counter
- **Description:** Total number of database queries by type
- **Labels:**
  - `query_type` - Type of query (select, insert, update, delete)

**Example PromQL:**
```promql
# Query rate by type
rate(brokkr_database_queries_total[5m])

# Total queries per second
sum(rate(brokkr_database_queries_total[5m]))
```

#### `brokkr_database_query_duration_seconds`
- **Type:** Histogram
- **Description:** Database query latency distribution in seconds
- **Labels:**
  - `query_type` - Type of query
- **Buckets:** 0.001, 0.005, 0.01, 0.025, 0.05, 0.1, 0.25, 0.5, 1.0, 2.5, 5.0 seconds

**Example PromQL:**
```promql
# 95th percentile query latency
histogram_quantile(0.95,
  sum(rate(brokkr_database_query_duration_seconds_bucket[5m])) by (le, query_type)
)
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

## Prometheus Configuration

### Manual Scrape Configuration

If you're not using the Prometheus Operator, add these scrape configs to your `prometheus.yml`:

```yaml
scrape_configs:
  # Broker metrics
  - job_name: 'brokkr-broker'
    kubernetes_sd_configs:
      - role: pod
    relabel_configs:
      - source_labels: [__meta_kubernetes_pod_label_app_kubernetes_io_name]
        action: keep
        regex: brokkr-broker
      - source_labels: [__meta_kubernetes_pod_container_port_number]
        action: keep
        regex: "3000"
      - source_labels: [__meta_kubernetes_namespace]
        target_label: kubernetes_namespace
      - source_labels: [__meta_kubernetes_pod_name]
        target_label: kubernetes_pod_name
    metrics_path: /metrics
    scrape_interval: 30s

  # Agent metrics
  - job_name: 'brokkr-agent'
    kubernetes_sd_configs:
      - role: pod
    relabel_configs:
      - source_labels: [__meta_kubernetes_pod_label_app_kubernetes_io_name]
        action: keep
        regex: brokkr-agent
      - source_labels: [__meta_kubernetes_pod_container_port_number]
        action: keep
        regex: "8080"
      - source_labels: [__meta_kubernetes_namespace]
        target_label: kubernetes_namespace
      - source_labels: [__meta_kubernetes_pod_name]
        target_label: kubernetes_pod_name
    metrics_path: /metrics
    scrape_interval: 30s
```

### ServiceMonitor Configuration (Prometheus Operator)

Both broker and agent Helm charts include optional ServiceMonitor CRDs for automatic discovery.

**Enable in values.yaml:**

```yaml
# For broker
metrics:
  enabled: true
  serviceMonitor:
    enabled: true
    interval: 30s
    additionalLabels:
      prometheus: kube-prometheus

# For agent
metrics:
  enabled: true
  serviceMonitor:
    enabled: true
    interval: 30s
    additionalLabels:
      prometheus: kube-prometheus
```

**Installation:**

```bash
# Broker with ServiceMonitor
helm install brokkr-broker oci://ghcr.io/colliery-io/charts/brokkr-broker \
  --set postgresql.enabled=true \
  --set metrics.serviceMonitor.enabled=true

# Agent with ServiceMonitor
helm install brokkr-agent oci://ghcr.io/colliery-io/charts/brokkr-agent \
  --set broker.url=http://brokkr-broker:3000 \
  --set broker.pak="<PAK>" \
  --set metrics.serviceMonitor.enabled=true
```

**Verify ServiceMonitor:**

```bash
kubectl get servicemonitor
kubectl describe servicemonitor brokkr-broker
kubectl describe servicemonitor brokkr-agent
```

## Example Alerting Rules

Create a PrometheusRule resource for automated alerting:

```yaml
apiVersion: monitoring.coreos.com/v1
kind: PrometheusRule
metadata:
  name: brokkr-alerts
  labels:
    prometheus: kube-prometheus
spec:
  groups:
    - name: brokkr-broker
      interval: 30s
      rules:
        # No active agents
        - alert: BrokerNoActiveAgents
          expr: brokkr_active_agents == 0
          for: 5m
          labels:
            severity: critical
          annotations:
            summary: "No active agents connected to broker"
            description: "Broker has no active agents for 5 minutes"

        # High error rate
        - alert: BrokerHighErrorRate
          expr: |
            sum(rate(brokkr_http_requests_total{status=~"[45].."}[5m]))
            / sum(rate(brokkr_http_requests_total[5m])) > 0.05
          for: 5m
          labels:
            severity: warning
          annotations:
            summary: "Broker error rate above 5%"
            description: "Broker HTTP error rate is {{ $value | humanizePercentage }}"

        # High request latency
        - alert: BrokerHighLatency
          expr: |
            histogram_quantile(0.95,
              sum(rate(brokkr_http_request_duration_seconds_bucket[5m])) by (le, endpoint)
            ) > 1.0
          for: 10m
          labels:
            severity: warning
          annotations:
            summary: "Broker p95 latency above 1s"
            description: "Endpoint {{ $labels.endpoint }} p95 latency is {{ $value }}s"

        # Stale agent heartbeat
        - alert: BrokerAgentHeartbeatStale
          expr: brokkr_agent_heartbeat_age_seconds > 300
          for: 5m
          labels:
            severity: warning
          annotations:
            summary: "Agent heartbeat is stale"
            description: "Agent {{ $labels.agent_name }} last heartbeat {{ $value }}s ago"

    - name: brokkr-agent
      interval: 30s
      rules:
        # Poll failures
        - alert: AgentPollFailures
          expr: |
            rate(brokkr_agent_poll_requests_total{status="error"}[5m])
            / rate(brokkr_agent_poll_requests_total[5m]) > 0.1
          for: 5m
          labels:
            severity: warning
          annotations:
            summary: "Agent poll failure rate above 10%"
            description: "Agent poll failure rate is {{ $value | humanizePercentage }}"

        # No successful polls
        - alert: AgentNoSuccessfulPolls
          expr: time() - brokkr_agent_last_successful_poll_timestamp > 300
          for: 5m
          labels:
            severity: critical
          annotations:
            summary: "Agent has not successfully polled broker"
            description: "Agent last successful poll was {{ $value }}s ago"

        # High K8s operation latency
        - alert: AgentHighK8sLatency
          expr: |
            histogram_quantile(0.95,
              sum(rate(brokkr_agent_kubernetes_operation_duration_seconds_bucket[5m])) by (le, operation)
            ) > 5.0
          for: 10m
          labels:
            severity: warning
          annotations:
            summary: "Agent K8s operation latency above 5s"
            description: "Operation {{ $labels.operation }} p95 latency is {{ $value }}s"
```

## Grafana Dashboards

Brokkr includes pre-built Grafana dashboards for both broker and agent components.

### Installing Dashboards

**1. Download dashboard JSONs:**

```bash
# Broker dashboard
curl -O https://raw.githubusercontent.com/colliery-io/brokkr/main/docs/grafana/brokkr-broker-dashboard.json

# Agent dashboard
curl -O https://raw.githubusercontent.com/colliery-io/brokkr/main/docs/grafana/brokkr-agent-dashboard.json
```

**2. Import into Grafana:**

- Navigate to Grafana UI
- Go to Dashboards → Import
- Upload the JSON file or paste the JSON content
- Select your Prometheus datasource
- Click Import

### Broker Dashboard Features

The broker dashboard includes:

- **Active Agents** - Current count of connected agents
- **Total Stacks** - Number of managed stacks
- **Deployment Objects** - Total deployment objects
- **HTTP Request Rate** - Requests per second by endpoint
- **HTTP Request Latency** - p50, p95, p99 latencies by endpoint
- **Database Query Rate** - Queries per second by type
- **Database Query Latency** - p50, p95, p99 query latencies
- **Agent Heartbeat Age** - Time since last heartbeat per agent

### Agent Dashboard Features

The agent dashboard includes:

- **Broker Poll Request Rate** - Success/error poll rates
- **Broker Poll Latency** - p50, p95, p99 poll latencies
- **Kubernetes Operations Rate** - Operations per second by type
- **Kubernetes Operation Latency** - p50, p95, p99 operation latencies
- **Heartbeat Send Rate** - Heartbeats sent per second
- **Time Since Last Successful Poll** - Gauge showing polling health

## Integration with Other Monitoring Systems

### Datadog

Use the Datadog OpenMetrics integration to scrape Prometheus metrics:

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: datadog-checks
data:
  openmetrics.yaml: |
    init_config:
    instances:
      - prometheus_url: http://brokkr-broker:3000/metrics
        namespace: "brokkr.broker"
        metrics:
          - brokkr_*
      - prometheus_url: http://brokkr-agent:8080/metrics
        namespace: "brokkr.agent"
        metrics:
          - brokkr_agent_*
```

### New Relic

Use the New Relic Prometheus OpenMetrics integration:

```yaml
integrations:
  - name: nri-prometheus
    config:
      targets:
        - description: Brokkr Broker
          urls: ["http://brokkr-broker:3000/metrics"]
        - description: Brokkr Agent
          urls: ["http://brokkr-agent:8080/metrics"]
      transformations:
        - description: "Add cluster label"
          add_attributes:
            - metric_prefix: "brokkr_"
              attributes:
                cluster_name: "production"
```

## Performance Impact

Metrics collection has minimal performance overhead:

- **CPU:** <1% per component
- **Memory:** ~10MB for metrics registry
- **Network:** ~5KB per scrape (30s intervals = ~170KB/min)

Metrics are collected lazily and only computed when scraped by Prometheus.

## Troubleshooting

### Metrics Not Appearing

**Check endpoint accessibility:**

```bash
# Broker metrics
kubectl port-forward svc/brokkr-broker 3000:3000
curl http://localhost:3000/metrics

# Agent metrics
kubectl port-forward svc/brokkr-agent 8080:8080
curl http://localhost:8080/metrics
```

**Verify ServiceMonitor:**

```bash
# Check if ServiceMonitor is created
kubectl get servicemonitor brokkr-broker
kubectl get servicemonitor brokkr-agent

# Check Prometheus targets
kubectl port-forward svc/prometheus-operated 9090:9090
# Visit http://localhost:9090/targets
```

### Missing Labels

ServiceMonitor labels must match Prometheus ServiceMonitor selector. Check your Prometheus Operator configuration:

```bash
kubectl get prometheus -o yaml | grep serviceMonitorSelector
```

Update Helm values to include matching labels:

```yaml
metrics:
  serviceMonitor:
    enabled: true
    additionalLabels:
      prometheus: <your-prometheus-instance-label>
```

## Best Practices

1. **Use ServiceMonitors** when possible for automatic discovery
2. **Set appropriate scrape intervals** (30s is recommended)
3. **Configure alerting rules** for critical metrics
4. **Monitor resource usage** in high-traffic environments
5. **Use recording rules** for frequently queried expensive PromQL expressions
6. **Enable grafana dashboards** for operational visibility
7. **Test alerts** in staging before production deployment

## Related Documentation

- [Installation Guide](../getting-started/installation.md) - Helm chart installation
- [Health Check Endpoints](./health-endpoints.md) - Liveness and readiness probes
- [Configuration Reference](../how-to/configuration.md) - Advanced configuration options
