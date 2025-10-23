---
id: document-health-check-endpoints
level: task
title: "Document health check endpoints"
short_code: "BROKKR-T-0018"
created_at: 2025-10-21T12:37:06.187390+00:00
updated_at: 2025-10-22T10:25:30.826183+00:00
parent: BROKKR-I-0003
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: BROKKR-I-0003
---

# Document health check endpoints

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[BROKKR-I-0003]]

## Objective **[REQUIRED]**

Create comprehensive documentation for all health check endpoints (/healthz, /readyz, /health) for both broker and agent components. Include API specifications, response formats, Kubernetes probe configurations, and monitoring integration examples.

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [x] Documentation created in `docs/content/reference/health-endpoints.md`
- [x] All three endpoints documented for both broker and agent (/healthz, /readyz, /health)
- [x] API specifications include: endpoint URL, HTTP method, response codes, response format
- [x] Example requests and responses provided for each endpoint
- [x] Kubernetes liveness and readiness probe configuration examples
- [x] Monitoring integration examples (Prometheus, Datadog, custom scripts)
- [x] Troubleshooting section for common health check issues
- [x] Performance impact notes (latency, frequency recommendations)



## Documentation Sections **[CONDITIONAL: Documentation Task]**

### Content Outline for docs/content/reference/health-endpoints.md

**Overview:**
- Purpose of health check endpoints
- Three-tier health check pattern
- When to use each endpoint

**Broker Endpoints:**

**`/healthz` - Liveness Probe**
- URL: `http://broker:3000/healthz`
- Method: GET
- Response: 200 OK (simple text "OK")
- Purpose: Process is alive, no dependency checks
- Use for: Kubernetes livenessProbe

**`/readyz` - Readiness Probe**
- URL: `http://broker:3000/readyz`
- Method: GET
- Response: 200 OK if ready, 503 Service Unavailable if not
- Checks: PostgreSQL database connectivity
- Purpose: Service ready to accept traffic
- Use for: Kubernetes readinessProbe

**`/health` - Detailed Status**
- URL: `http://broker:3000/health`
- Method: GET
- Response: JSON with detailed status
- Example response:
```json
{
  "status": "healthy",
  "timestamp": "2024-01-15T10:30:00Z",
  "version": "1.0.0",
  "uptime": 3600,
  "components": {
    "database": {
      "status": "healthy",
      "latency_ms": 5
    }
  }
}
```
- Purpose: Detailed diagnostics and monitoring
- Use for: Monitoring systems, debugging

**Agent Endpoints:**

**`/healthz` - Liveness Probe**
- URL: `http://agent:8080/healthz`
- Method: GET
- Response: 200 OK
- Purpose: Process is alive

**`/readyz` - Readiness Probe**
- URL: `http://agent:8080/readyz`
- Method: GET
- Response: 200 OK if ready, 503 if not
- Checks: Kubernetes API connectivity, broker connectivity
- Purpose: Agent ready to process work

**`/health` - Detailed Status**
- URL: `http://agent:8080/health`
- Method: GET
- Response: JSON with component status
- Example response:
```json
{
  "status": "healthy",
  "timestamp": "2024-01-15T10:30:00Z",
  "version": "1.0.0",
  "components": {
    "kubernetes": {
      "status": "healthy",
      "api_latency_ms": 10
    },
    "broker": {
      "status": "healthy",
      "last_heartbeat": "2024-01-15T10:29:55Z"
    }
  }
}
```

**Kubernetes Configuration Examples:**

```yaml
# Broker deployment with health probes
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
        livenessProbe:
          httpGet:
            path: /healthz
            port: 3000
          initialDelaySeconds: 10
          periodSeconds: 10
          timeoutSeconds: 2
          failureThreshold: 3
        readinessProbe:
          httpGet:
            path: /readyz
            port: 3000
          initialDelaySeconds: 5
          periodSeconds: 5
          timeoutSeconds: 2
          failureThreshold: 2
```

**Monitoring Integration Examples:**

**Prometheus:**
- Custom metrics endpoint (if implemented in T-0019)
- Health endpoint polling for status labels

**Custom Monitoring Script:**
```bash
#!/bin/bash
# Check broker health and alert on failure
HEALTH=$(curl -s http://broker:3000/health | jq -r '.status')
if [ "$HEALTH" != "healthy" ]; then
  echo "ALERT: Broker unhealthy"
  # Send alert to monitoring system
fi
```

**Troubleshooting:**
- Readyz failing: Check database/k8s connectivity
- Health endpoint slow: Check component latencies
- Probes causing restarts: Adjust initialDelaySeconds

**Performance Recommendations:**
- Liveness probe: 10s period, 2s timeout
- Readiness probe: 5s period, 2s timeout
- Health endpoint: Limit to monitoring systems only

## Implementation Notes **[CONDITIONAL: Technical Task]**

### Technical Approach

**File to Create:**
- `docs/content/reference/health-endpoints.md`

**Documentation Strategy:**
- Reference actual implementation from T-0002 (broker) and T-0003 (agent)
- Test all endpoints to verify actual behavior matches documentation
- Include real example responses (not made up)
- Align probe configurations with actual Helm chart templates

**Validation:**
- Deploy broker and agent locally
- Curl each endpoint and capture actual responses
- Test failure scenarios (DB down, K8s unreachable)
- Verify probe configurations work in Helm charts

### Dependencies

- Health endpoints implemented in T-0002 (broker) and T-0003 (agent)
- Helm charts exist with probe configurations from T-0006, T-0007
- Can reference actual implementation code for accuracy

### Risk Considerations

**Risk: Documentation may not match actual implementation**
- Mitigation: Test all endpoints before documenting
- Verify response formats match actual code

**Risk: Response format may change during development**
- Mitigation: Document current state, note if responses are subject to change
- Version the API documentation if needed

**Risk: Performance recommendations may not reflect real-world usage**
- Mitigation: Base recommendations on existing Kubernetes best practices
- Note that values may need tuning based on specific deployments

## Status Updates **[REQUIRED]**

### 2025-10-22 - Task Completed

Created comprehensive health check endpoint documentation at `docs/content/reference/health-endpoints.md` covering:

- All three health check endpoints (/healthz, /readyz, /health) for both broker and agent
- Complete API specifications with actual implementation details verified from source code
- Real example requests and responses for healthy and unhealthy states
- Kubernetes probe configurations from actual Helm chart templates
- Monitoring integration examples for Prometheus, Datadog, and custom scripts
- Comprehensive troubleshooting section for common issues
- Performance considerations and recommended probe intervals
- Best practices for health check configuration

Documentation is based on actual implementation from:
- Broker: `crates/brokkr-broker/src/api/mod.rs`
- Agent: `crates/brokkr-agent/src/health.rs`
- Helm charts: `charts/brokkr-broker/templates/deployment.yaml` and `charts/brokkr-agent/templates/deployment.yaml`

All acceptance criteria met.
