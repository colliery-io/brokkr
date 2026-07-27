# Health Check Endpoints

Brokkr exposes health check endpoints on both the broker and the agent. Each endpoint answers a different question, and the endpoints deliberately do not answer each other's.

## Liveness vs. Readiness

The two probe endpoints are not interchangeable, and the difference is intentional in both components.

**`/healthz` (liveness)** answers only "is this process running and serving HTTP?" It performs no dependency checks on either component. A failing liveness probe causes Kubernetes to restart the container, so making it depend on an external system would convert an outage of that system into a fleet-wide restart loop. This is why the broker's `/healthz` does not check the database: a database blip must not restart every broker pod.

**`/readyz` (readiness)** answers "can this instance serve traffic right now?" Readiness gates Service endpoints, so it checks the one dependency without which the component cannot do its job — the database for the broker, the Kubernetes API for the agent. A failing readiness probe removes the pod from the Service endpoints without restarting it, so a dependency outage sheds traffic and recovers on its own once the dependency returns.

**`/health` (detailed status)** is a JSON diagnostic surface for monitoring systems. Only the agent implements it; the broker does not.

Consequently:

| Signal | Tells you | Does *not* tell you |
|--------|-----------|---------------------|
| Broker `/healthz` = 200 | The broker process is up and its HTTP stack is serving | Whether the database is reachable, whether requests are succeeding, whether migrations ran |
| Broker `/readyz` = 200 | A database connection was checked out and answered a trivial query within the last two seconds | Whether the schema is at the expected migration version, whether any particular API route works, whether the database is fast |
| Broker `/readyz` = 503 | The broker could not obtain a working database connection | Which of pool exhaustion, network failure, or database failure caused it (see broker logs) |
| Agent `/healthz` = 200 | The agent process is up | Whether it can reach the Kubernetes API or the broker |
| Agent `/readyz` = 200 | The Kubernetes API answered a version request | Whether the agent can reach the broker, or whether reconciliation is succeeding |
| Agent `/health` = 200 | Both the Kubernetes API and the broker connection are healthy | Whether individual deployment objects applied successfully (see [deployment health](./deployment-health.md)) |

## Broker Health Endpoints

The broker exposes `/healthz`, `/readyz`, and `/metrics` on port 3000. All three sit outside the API authentication middleware and require no credentials.

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
- Process crashed or hung: No response (Kubernetes will restart the container)

`/healthz` returns `200 OK` even when the database is unreachable. That is deliberate — see [Liveness vs. Readiness](#liveness-vs-readiness). Use `/readyz` to detect database problems.

### `/readyz` - Readiness Probe

**Purpose:** Verify that the broker can serve API traffic, which requires a reachable database.

**Details:**
- **URL:** `http://<broker-host>:3000/readyz`
- **Method:** `GET`
- **Response:** `200 OK` with plain text `"Ready"`, or `503 Service Unavailable` with plain text `"database unavailable"`
- **Checks:** Checks out a connection from the database pool (bounded to 750 ms) and runs `SELECT 1`
- **Caching:** The verdict — success or failure — is cached for 2 seconds. Probes arriving inside that window are answered from the cache without touching the database.
- **Use case:** Kubernetes readinessProbe to control traffic routing

**Example Request:**
```bash
curl -i http://brokkr-broker:3000/readyz
```

**Example Response (Ready):**
```
Ready
```
*HTTP Status: 200 OK*

**Example Response (Not Ready):**
```
database unavailable
```
*HTTP Status: 503 Service Unavailable*

**Failure Scenarios:**
- Database unreachable, refusing connections, or down: `503 Service Unavailable`
- Connection pool exhausted for longer than the 750 ms checkout budget: `503 Service Unavailable`
- Database reachable but rejecting queries (for example, credentials revoked): `503 Service Unavailable`

The specific cause is not encoded in the response body; the broker logs the underlying error at each cache-miss check.

**What `/readyz` does not check:**
- **Migrations.** Migrations run once at broker startup and abort the process on failure, so a running broker has already applied them. The probe does not re-verify schema version.
- **Individual API routes.** A `200 Ready` means the database answered, not that any given endpoint will succeed.
- **Database latency.** The check passes as long as `SELECT 1` completes within the checkout budget; it is not a performance signal. Use the [Prometheus metrics](./monitoring.md) for latency.

Because the endpoint is unauthenticated, the 2-second cache also bounds how much database load an external caller can generate by polling it.

### Detailed Status

The broker does not implement a `/health` endpoint. For database connectivity, active agent counts, and request-level state, scrape `/metrics` (see [Monitoring & Observability](./monitoring.md)) or use the fleet API (see [Monitoring Your Agent Fleet](../how-to/fleet-monitoring.md)).

The `/api/v1/.../health` routes are a different feature entirely: they report the health of *deployed workloads*, not of the broker process. See [Deployment Health](./deployment-health.md).

## Agent Health Endpoints

The agent exposes `/healthz`, `/readyz`, `/health`, and `/metrics` on port 8080. The agent's readiness check differs from the broker's: it validates Kubernetes API connectivity rather than database connectivity, and its result is not cached.

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
- Process crashed or hung: No response (Kubernetes will restart the container)

Like the broker's, the agent's `/healthz` performs no dependency checks and returns `200 OK` even when the Kubernetes API or the broker is unreachable.

### `/readyz` - Readiness Probe

**Purpose:** Verify that the agent can reach the Kubernetes API it reconciles against.

**Details:**
- **URL:** `http://<agent-host>:8080/readyz`
- **Method:** `GET`
- **Response:** `200 OK` with plain text `"Ready"`, or `503 Service Unavailable` with plain text `"Kubernetes API unavailable"`
- **Checks:** Requests the Kubernetes API server version
- **Caching:** None — every probe is a live request to the API server
- **Use case:** Kubernetes readinessProbe to control agent availability

Broker connectivity is **not** part of the agent's readiness check. An agent that cannot reach the broker still reports `Ready`; use `/health` or the `brokkr_agent_last_successful_poll_timestamp` metric to detect that condition.

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

These values are baked into the chart templates and are not exposed as `values.yaml` settings; change them by editing the rendered Deployment.

The broker's readiness defaults accommodate its database check: the worst case on a cache miss is the 750 ms checkout budget plus the query, well inside the 3 s probe timeout, and an unreachable database returns a fast `503` rather than hanging until the timeout. With `periodSeconds: 5` and a 2-second cache TTL, every kubelet probe is a real database round-trip; the cache exists to bound load from other callers, not from the kubelet.

Probe manifests, tuning, and troubleshooting are covered in [Setting Up Monitoring](../how-to/monitoring-setup.md#configure-kubernetes-probes).

## Related Documentation

- [Setting Up Monitoring](../how-to/monitoring-setup.md) - Probe configuration, external health monitoring, troubleshooting
- [Monitoring & Observability](./monitoring.md) - Prometheus metrics and dashboards
- [Installation Guide](../getting-started/installation.md) - Helm chart installation with probe configuration
- [Configuration Reference](../getting-started/configuration.md) - Environment variables and advanced configuration
