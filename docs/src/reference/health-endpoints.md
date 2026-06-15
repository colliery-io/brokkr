# Health Check Endpoints

Brokkr provides comprehensive health check endpoints for both the broker and agent components. These endpoints follow a three-tier pattern designed for different use cases: simple liveness checks, readiness validation, and detailed health diagnostics.

## Three-Tier Health Check Pattern

Brokkr implements a three-tier health check system:

1. **`/healthz`** - Liveness probe: Simple check that the process is alive
2. **`/readyz`** - Readiness probe: Validates that the service is ready to accept traffic
3. **`/health`** - Detailed diagnostics: Comprehensive JSON status for monitoring and debugging

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
  "version": "0.8.0",
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
  "version": "0.8.0",
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
  "version": "0.8.0",
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

## Default Probe Configuration

The Helm charts ship with these probe defaults for both broker and agent:

| Setting | Liveness (`/healthz`) | Readiness (`/readyz`) |
|---------|----------------------|----------------------|
| `initialDelaySeconds` | 30 | 10 |
| `periodSeconds` | 10 | 5 |
| `timeoutSeconds` | 5 | 3 |
| `failureThreshold` | 3 | 3 |

Probe manifests, tuning, and troubleshooting are covered in [Setting Up Monitoring](../how-to/monitoring-setup.md#configure-kubernetes-probes).

## Related Documentation

- [Setting Up Monitoring](../how-to/monitoring-setup.md) - Probe configuration, external health monitoring, troubleshooting
- [Monitoring & Observability](./monitoring.md) - Prometheus metrics and dashboards
- [Installation Guide](../getting-started/installation.md) - Helm chart installation with probe configuration
- [Configuration Reference](../getting-started/configuration.md) - Environment variables and advanced configuration
