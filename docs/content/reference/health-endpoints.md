---
title: Health Check Endpoints
weight: 20
---

# Health Check Endpoints

Brokkr provides comprehensive health check endpoints for both the broker and agent components. These endpoints follow a three-tier pattern designed for different use cases: simple liveness checks, readiness validation, and detailed health diagnostics.

## Three-Tier Health Check Pattern

Brokkr implements a three-tier health check system:

1. **`/healthz`** - Liveness probe: Simple check that the process is alive
2. **`/readyz`** - Readiness probe: Validates that the service is ready to accept traffic
3. **`/health`** - Detailed diagnostics: Comprehensive JSON status for monitoring and debugging

This pattern aligns with Kubernetes best practices and provides appropriate checks for different operational needs.

## Broker Health Endpoints

The broker exposes health check endpoints on port 3000.

### `/healthz` - Liveness Probe

**Purpose:** Verify that the broker process is alive and responding to requests.

**Details:**
- **URL:** `http://<broker-host>:3000/healthz`
- **Method:** `GET`
- **Response:** `200 OK` with plain text body `"OK"`
- **Checks:** None (process must be alive to respond)
- **Use case:** Kubernetes livenessProbe to restart failed containers

**Example Request:**
```bash
curl http://brokkr-broker:3000/healthz
```

**Example Response:**
```
OK
```

**Failure Scenarios:**
- Process crashed or hung: No response (Kubernetes will restart container)

### `/readyz` - Readiness Probe

**Purpose:** Verify that the broker is ready to accept API requests.

**Details:**
- **URL:** `http://<broker-host>:3000/readyz`
- **Method:** `GET`
- **Response:** `200 OK` if ready, returns plain text `"Ready"`
- **Checks:** Basic broker readiness (currently lightweight check)
- **Use case:** Kubernetes readinessProbe to control traffic routing

**Example Request:**
```bash
curl http://brokkr-broker:3000/readyz
```

**Example Response (Healthy):**
```
Ready
```

**Failure Scenarios:**
- Broker not ready: Returns appropriate error status
- Database connectivity issues would be detected by application errors

### `/health` - Detailed Status

The broker currently provides basic health information. For detailed metrics about database connectivity, active agents, and system state, use the `/metrics` endpoint or the monitoring integration (see [Monitoring & Observability](./monitoring.md)).

## Agent Health Endpoints

The agent exposes health check endpoints on port 8080 with comprehensive dependency checking.

### `/healthz` - Liveness Probe

**Purpose:** Verify that the agent process is alive and responding to requests.

**Details:**
- **URL:** `http://<agent-host>:8080/healthz`
- **Method:** `GET`
- **Response:** `200 OK` with plain text body `"OK"`
- **Checks:** None (process must be alive to respond)
- **Use case:** Kubernetes livenessProbe to restart failed containers

**Example Request:**
```bash
curl http://brokkr-agent:8080/healthz
```

**Example Response:**
```
OK
```

**Failure Scenarios:**
- Process crashed or hung: No response (Kubernetes will restart container)

### `/readyz` - Readiness Probe

**Purpose:** Verify that the agent can perform its core functions.

**Details:**
- **URL:** `http://<agent-host>:8080/readyz`
- **Method:** `GET`
- **Response:** `200 OK` if ready, `503 Service Unavailable` if not
- **Checks:** Kubernetes API connectivity
- **Use case:** Kubernetes readinessProbe to control agent availability

**Example Request:**
```bash
curl http://brokkr-agent:8080/readyz
```

**Example Response (Healthy):**
```
Ready
```

**Example Response (Unhealthy):**
```
Kubernetes API unavailable
```
*HTTP Status: 503 Service Unavailable*

**Failure Scenarios:**
- Kubernetes API unreachable: Returns `503 Service Unavailable`
- Invalid kubeconfig or expired credentials: Returns `503 Service Unavailable`

### `/health` - Detailed Status

**Purpose:** Provide comprehensive JSON status for monitoring systems and debugging.

**Details:**
- **URL:** `http://<agent-host>:8080/health`
- **Method:** `GET`
- **Response:** `200 OK` if healthy, `503 Service Unavailable` if any check fails
- **Checks:**
  - Kubernetes API connectivity
  - Broker connection status
  - Service uptime
  - Application version
- **Use case:** Monitoring systems, operational dashboards, debugging

**Example Request:**
```bash
curl http://brokkr-agent:8080/health
```

**Example Response (Healthy):**
```json
{
  "status": "healthy",
  "kubernetes": {
    "connected": true
  },
  "broker": {
    "connected": true,
    "last_heartbeat": "2024-01-15T10:29:55Z"
  },
  "uptime_seconds": 3600,
  "version": "0.1.0",
  "timestamp": "2024-01-15T10:30:00Z"
}
```
*HTTP Status: 200 OK*

**Example Response (Unhealthy - K8s Issue):**
```json
{
  "status": "unhealthy",
  "kubernetes": {
    "connected": false,
    "error": "connection refused: Unable to connect to the server"
  },
  "broker": {
    "connected": true,
    "last_heartbeat": "2024-01-15T10:29:55Z"
  },
  "uptime_seconds": 3600,
  "version": "0.1.0",
  "timestamp": "2024-01-15T10:30:00Z"
}
```
*HTTP Status: 503 Service Unavailable*

**Example Response (Unhealthy - Broker Issue):**
```json
{
  "status": "unhealthy",
  "kubernetes": {
    "connected": true
  },
  "broker": {
    "connected": false
  },
  "uptime_seconds": 3600,
  "version": "0.1.0",
  "timestamp": "2024-01-15T10:30:00Z"
}
```
*HTTP Status: 503 Service Unavailable*

**Response Fields:**
- `status`: Overall health status (`"healthy"` or `"unhealthy"`)
- `kubernetes.connected`: Boolean indicating K8s API connectivity
- `kubernetes.error`: Optional error message if connection failed
- `broker.connected`: Boolean indicating broker connectivity
- `broker.last_heartbeat`: ISO 8601 timestamp of last successful heartbeat
- `uptime_seconds`: Service uptime in seconds
- `version`: Application version from Cargo.toml
- `timestamp`: Current timestamp in RFC3339 format

## Kubernetes Probe Configuration

### Broker Deployment

The broker Helm chart includes these recommended probe configurations:

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

**Configuration Rationale:**
- **Liveness:**
  - `initialDelaySeconds: 30` - Allow broker startup and database connection
  - `periodSeconds: 10` - Check every 10 seconds
  - `failureThreshold: 3` - Restart after 30 seconds of failures
- **Readiness:**
  - `initialDelaySeconds: 10` - Quick readiness check after startup
  - `periodSeconds: 5` - Check frequently to minimize downtime
  - `failureThreshold: 3` - Remove from service after 15 seconds

### Agent Deployment

The agent Helm chart includes these recommended probe configurations:

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

**Configuration Rationale:**
- **Liveness:**
  - `initialDelaySeconds: 30` - Allow agent startup and K8s/broker connection
  - `periodSeconds: 10` - Check every 10 seconds
  - `failureThreshold: 3` - Restart after 30 seconds of failures
- **Readiness:**
  - `initialDelaySeconds: 10` - Quick readiness check after startup
  - `periodSeconds: 5` - Check frequently for K8s API issues
  - `failureThreshold: 3` - Remove from service after 15 seconds

## Monitoring Integration

### Prometheus Health Check Monitoring

While health endpoints are primarily for Kubernetes probes, you can also monitor them with Prometheus using the Blackbox Exporter:

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

### Custom Health Check Script

You can create custom monitoring scripts to poll the health endpoints:

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

### Datadog Integration

Monitor health endpoints using Datadog's HTTP check:

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

## Troubleshooting

### Health Check Failures

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

### Probe Configuration Issues

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

## Performance Considerations

### Endpoint Latency

Health check endpoints are designed to be lightweight:

**Broker Endpoints:**
- `/healthz`: <1ms (no checks, immediate response)
- `/readyz`: <5ms (lightweight readiness validation)

**Agent Endpoints:**
- `/healthz`: <1ms (no checks, immediate response)
- `/readyz`: 5-50ms (depends on Kubernetes API latency)
- `/health`: 10-100ms (multiple checks including K8s API call)

### Probe Frequency Impact

With default probe configurations:
- **Liveness probes:** Every 10 seconds = 6 requests/minute per pod
- **Readiness probes:** Every 5 seconds = 12 requests/minute per pod
- **Total per pod:** ~18 health check requests/minute

This generates minimal load:
- **CPU:** <0.1% per probe
- **Memory:** Negligible
- **Network:** <1KB per probe

### Recommended Probe Intervals

**Production Environments:**
```yaml
livenessProbe:
  initialDelaySeconds: 30
  periodSeconds: 10
  timeoutSeconds: 5
  failureThreshold: 3

readinessProbe:
  initialDelaySeconds: 10
  periodSeconds: 5
  timeoutSeconds: 3
  failureThreshold: 3
```

**High-Availability Environments (faster failure detection):**
```yaml
livenessProbe:
  initialDelaySeconds: 30
  periodSeconds: 5
  timeoutSeconds: 3
  failureThreshold: 2

readinessProbe:
  initialDelaySeconds: 10
  periodSeconds: 3
  timeoutSeconds: 2
  failureThreshold: 2
```

**Development/Testing (more forgiving):**
```yaml
livenessProbe:
  initialDelaySeconds: 60
  periodSeconds: 30
  timeoutSeconds: 10
  failureThreshold: 5

readinessProbe:
  initialDelaySeconds: 30
  periodSeconds: 10
  timeoutSeconds: 5
  failureThreshold: 5
```

## Best Practices

1. **Use all three endpoint types appropriately:**
   - `/healthz` for liveness probes only
   - `/readyz` for readiness probes only
   - `/health` for monitoring and debugging (not for probes)

2. **Set appropriate timeouts:**
   - Account for slow network conditions
   - Consider cold start performance
   - Test probe timing in staging before production

3. **Monitor probe failures:**
   - Alert on excessive readiness probe failures
   - Track liveness probe failure rate
   - Use Prometheus to monitor probe success rate

4. **Tune for your environment:**
   - Adjust `initialDelaySeconds` based on actual startup time
   - Increase `periodSeconds` if probes cause excessive load
   - Increase `failureThreshold` in high-latency environments

5. **Test probe configurations:**
   - Simulate failures in staging
   - Verify restarts work as expected
   - Ensure startup timing is adequate

6. **Use `/health` endpoint for operational visibility:**
   - Monitor detailed status in dashboards
   - Parse JSON response for alerting
   - Track component dependencies (K8s API, broker)

7. **Avoid common mistakes:**
   - Don't use `/health` for Kubernetes probes (too detailed, may cause false positives)
   - Don't set timeouts shorter than actual endpoint latency
   - Don't set `initialDelaySeconds` too low for startup dependencies

## Related Documentation

- [Monitoring & Observability](./monitoring.md) - Prometheus metrics and dashboards
- [Installation Guide](../getting-started/installation.md) - Helm chart installation with probe configuration
- [Configuration Reference](../getting-started/configuration.md) - Environment variables and advanced configuration
