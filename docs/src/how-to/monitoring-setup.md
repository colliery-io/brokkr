# Setting Up Monitoring

This guide walks through wiring Brokkr's metrics and health endpoints into Prometheus, Grafana, Kubernetes probes, and external monitoring systems. For the full list of exported metrics, see the [Monitoring & Observability Reference](../reference/monitoring.md); for the endpoint and response catalogs, see [Health Check Endpoints](../reference/health-endpoints.md).

## Configure Prometheus scraping

### Manual scrape configuration

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

### ServiceMonitor configuration (Prometheus Operator)

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

## Set up alerting rules

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

## Import Grafana dashboards

Brokkr ships pre-built Grafana dashboards for the broker, agent, and WebSocket channel (see the [dashboard inventory](../reference/monitoring.md#grafana-dashboards)).

**1. Download dashboard JSONs:**

```bash
# Broker dashboard
curl -O https://raw.githubusercontent.com/colliery-io/brokkr/main/docs/grafana/brokkr-broker-dashboard.json

# Agent dashboard
curl -O https://raw.githubusercontent.com/colliery-io/brokkr/main/docs/grafana/brokkr-agent-dashboard.json

# WebSocket channel dashboard
curl -O https://raw.githubusercontent.com/colliery-io/brokkr/main/docs/grafana/brokkr-ws-channel-dashboard.json
```

Prometheus alert rules for the WebSocket channel ship alongside the dashboards as `docs/grafana/brokkr-ws-channel.rules.yml`.

**2. Import into Grafana:**

- Navigate to Grafana UI
- Go to Dashboards → Import
- Upload the JSON file or paste the JSON content
- Select your Prometheus datasource
- Click Import

## Configure Kubernetes probes

Use `/healthz` for liveness probes and `/readyz` for readiness probes. The agent's `/health` endpoint performs multiple dependency checks and is intended for monitoring systems rather than Kubernetes probes.

### Broker deployment

The broker Helm chart ships with these probe defaults:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: brokkr-broker
spec:
  template:
    spec:
      containers:
      - name: broker
        image: ghcr.io/colliery-io/brokkr-broker:latest
        ports:
        - name: http
          containerPort: 3000
          protocol: TCP
        livenessProbe:
          httpGet:
            path: /healthz
            port: http
          initialDelaySeconds: 30
          periodSeconds: 10
          timeoutSeconds: 5
          failureThreshold: 3
        readinessProbe:
          httpGet:
            path: /readyz
            port: http
          initialDelaySeconds: 10
          periodSeconds: 5
          timeoutSeconds: 3
          failureThreshold: 3
```

### Agent deployment

The agent Helm chart ships with these probe defaults:

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: brokkr-agent
spec:
  template:
    spec:
      containers:
      - name: agent
        image: ghcr.io/colliery-io/brokkr-agent:latest
        ports:
        - name: http
          containerPort: 8080
          protocol: TCP
        livenessProbe:
          httpGet:
            path: /healthz
            port: 8080
          initialDelaySeconds: 30
          periodSeconds: 10
          timeoutSeconds: 5
          failureThreshold: 3
        readinessProbe:
          httpGet:
            path: /readyz
            port: 8080
          initialDelaySeconds: 10
          periodSeconds: 5
          timeoutSeconds: 3
          failureThreshold: 3
```

Tighter intervals detect failures faster at the cost of more probe traffic; longer `initialDelaySeconds` accommodates slower startup (for example, database connection on broker startup).

## Monitor health endpoints externally

### Prometheus Blackbox Exporter

Health endpoints can also be monitored via the Blackbox Exporter:

```yaml
# Prometheus scrape config for blackbox exporter
scrape_configs:
  - job_name: 'brokkr-health-checks'
    metrics_path: /probe
    params:
      module: [http_2xx]
    static_configs:
      - targets:
          - http://brokkr-broker:3000/healthz
          - http://brokkr-broker:3000/readyz
          - http://brokkr-agent:8080/healthz
          - http://brokkr-agent:8080/readyz
          - http://brokkr-agent:8080/health
    relabel_configs:
      - source_labels: [__address__]
        target_label: __param_target
      - source_labels: [__param_target]
        target_label: instance
      - target_label: __address__
        replacement: blackbox-exporter:9115
```

### Custom health check script

A monitoring script can poll the health endpoints directly:

```bash
#!/bin/bash
# check-brokkr-health.sh - Monitor Brokkr component health

BROKER_URL="http://brokkr-broker:3000"
AGENT_URL="http://brokkr-agent:8080"

# Check broker readiness
if ! curl -sf "$BROKER_URL/readyz" > /dev/null; then
  echo "ALERT: Broker not ready"
  # Send alert to monitoring system
fi

# Check agent detailed health
AGENT_HEALTH=$(curl -sf "$AGENT_URL/health")
if [ $? -ne 0 ]; then
  echo "ALERT: Agent health check failed"
  # Send alert
else
  STATUS=$(echo "$AGENT_HEALTH" | jq -r '.status')
  if [ "$STATUS" != "healthy" ]; then
    echo "ALERT: Agent unhealthy - $AGENT_HEALTH"
    # Send alert with details
  fi
fi
```

## Integrate with other monitoring systems

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

Monitor the health endpoints using Datadog's HTTP check:

```yaml
# datadog-checks.yaml
init_config:

instances:
  # Broker health checks
  - name: brokkr-broker-liveness
    url: http://brokkr-broker:3000/healthz
    timeout: 3
    method: GET

  - name: brokkr-broker-readiness
    url: http://brokkr-broker:3000/readyz
    timeout: 3
    method: GET

  # Agent health checks
  - name: brokkr-agent-liveness
    url: http://brokkr-agent:8080/healthz
    timeout: 3
    method: GET

  - name: brokkr-agent-readiness
    url: http://brokkr-agent:8080/readyz
    timeout: 3
    method: GET

  - name: brokkr-agent-detailed
    url: http://brokkr-agent:8080/health
    timeout: 5
    method: GET
    content_match: '"status":"healthy"'
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

## Troubleshooting

### Metrics not appearing

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

### Missing labels

ServiceMonitor labels must match the Prometheus ServiceMonitor selector. Check your Prometheus Operator configuration:

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

### Health check failures

**Symptom:** Broker `/readyz` returning errors or timeouts

**Possible Causes:**
- Database connectivity issues
- Broker process overloaded
- Network policy blocking health probe

**Resolution:**
```bash
# Check broker logs
kubectl logs -l app.kubernetes.io/name=brokkr-broker

# Test database connectivity
kubectl exec -it <broker-pod> -- env | grep DATABASE

# Test health endpoint manually
kubectl port-forward svc/brokkr-broker 3000:3000
curl -v http://localhost:3000/readyz
```

**Symptom:** Agent `/readyz` failing with "Kubernetes API unavailable"

**Possible Causes:**
- Invalid or expired service account credentials
- RBAC permissions insufficient
- Kubernetes API server unreachable
- Network policy blocking API access

**Resolution:**
```bash
# Check agent logs for detailed error
kubectl logs -l app.kubernetes.io/name=brokkr-agent

# Verify service account exists
kubectl get serviceaccount brokkr-agent

# Test K8s API access from agent pod
kubectl exec -it <agent-pod> -- sh
# Inside pod:
curl -k https://kubernetes.default.svc/api/v1/namespaces/default
```

**Symptom:** Agent `/health` showing `"broker.connected": false`

**Possible Causes:**
- Broker service unavailable
- Invalid broker URL configuration
- Network policy blocking broker access
- Authentication issues (invalid PAK)

**Resolution:**
```bash
# Check broker service
kubectl get svc brokkr-broker

# Test connectivity from agent to broker
kubectl exec -it <agent-pod> -- sh
# Inside pod:
curl http://brokkr-broker:3000/healthz

# Check agent configuration
kubectl get configmap <agent-configmap> -o yaml | grep BROKER

# Check agent logs for authentication errors
kubectl logs -l app.kubernetes.io/name=brokkr-agent | grep -i "auth\|broker"
```

### Probe configuration issues

**Symptom:** Container restarting frequently due to failed liveness probes

**Possible Causes:**
- `initialDelaySeconds` too low for startup time
- `timeoutSeconds` too low for slow responses
- `failureThreshold` too low (not enough retry tolerance)

**Resolution:**
```bash
# Check recent pod events
kubectl describe pod <pod-name>

# Look for "Liveness probe failed" messages
# Adjust probe configuration based on actual startup time

# For slow-starting containers, increase initialDelaySeconds:
kubectl edit deployment brokkr-broker
# Set initialDelaySeconds: 60 for livenessProbe
```

**Symptom:** Pod marked not ready immediately after deployment

**Possible Causes:**
- Dependencies not available at startup
- `initialDelaySeconds` on readiness probe too aggressive

**Resolution:**
```bash
# Check readiness probe configuration
kubectl get deployment brokkr-agent -o yaml | grep -A10 readinessProbe

# Test readiness endpoint manually during startup
kubectl port-forward <pod-name> 8080:8080
# In another terminal:
watch -n 1 'curl -i http://localhost:8080/readyz'
```

## Related Documentation

- [Monitoring & Observability Reference](../reference/monitoring.md) - Metrics catalogs and dashboard inventory
- [Health Check Endpoints](../reference/health-endpoints.md) - Endpoint and response catalogs
- [Monitoring Your Agent Fleet](./fleet-monitoring.md) - Application/fleet observability via the fleet API (agent connectivity, backpressure, and health signals), as distinct from this page's Prometheus/infra metrics
- [Installation Guide](../getting-started/installation.md) - Helm chart installation
