# Brokkr Agent Values Files

Pre-configured values files for common deployment scenarios. These files provide sensible defaults for different environments and RBAC configurations.

## Quick Start

Install the agent with a specific environment configuration:

```bash
# Production deployment with cluster-wide RBAC
helm install brokkr-agent . -f values/production.yaml \
  --set broker.agentName=prod-cluster-01 \
  --set broker.clusterName=production-us-east-1 \
  --set broker.pak=<your-pak>

# Development deployment with namespace-scoped RBAC
helm install brokkr-agent-dev . -f values/development.yaml \
  --set broker.agentName=dev-test \
  --set broker.clusterName=local-dev \
  --set broker.pak=<your-pak>

# Staging deployment
helm install brokkr-agent-staging . -f values/staging.yaml \
  --set broker.agentName=staging-cluster-01 \
  --set broker.clusterName=staging \
  --set broker.pak=<your-pak>
```

## Required Configuration

All deployments require these three values to be set:

1. **broker.agentName**: Unique identifier for this agent instance
   - Must match agent registered in broker
   - Examples: `prod-cluster-01`, `dev-laptop`, `staging-us-east-1`

2. **broker.clusterName**: Human-readable cluster identifier
   - Helps organize agents in broker UI
   - Examples: `production-us-east-1`, `local-dev`, `staging`

3. **broker.pak**: Pre-Authenticated Key
   - Obtain from broker when creating agent
   - SECURITY: Always use `--set` flag, never commit to files

## Available Scenarios

### Production (`values/production.yaml`)

**Purpose**: Production deployment with full cluster-wide visibility.

**Key Features**:
- Cluster-wide RBAC (ClusterRole)
- Production image tag (pinned version)
- Higher resource limits (256Mi-512Mi memory, 100m-500m CPU)
- Full security context with seccomp
- 30-second polling interval
- Info-level logging
- HTTPS broker connection

**Use Cases**:
- Production cluster inventory collection
- Multi-namespace monitoring
- Cluster-wide resource discovery
- Compliance and security scanning

**RBAC Permissions**:
- Read access to all namespaces
- Read access to cluster-scoped resources (nodes, PVs, etc.)
- Read access to workloads, configs, and network resources
- See [RBAC.md](../RBAC.md) for complete permission details

**Example**:
```bash
helm install brokkr-agent . -f values/production.yaml \
  --set broker.url=https://brokkr.example.com \
  --set broker.agentName=prod-eks-us-east-1 \
  --set broker.clusterName=production-us-east-1 \
  --set broker.pak=$BROKER_PAK
```

### Development (`values/development.yaml`)

**Purpose**: Local development and testing with namespace-scoped permissions.

**Key Features**:
- Namespace-scoped RBAC (Role, not ClusterRole)
- Latest image tag with always pull policy
- Minimal resources (64Mi-128Mi memory, 25m-100m CPU)
- 10-second polling interval (faster testing)
- Debug-level logging
- HTTP broker connection (typically in-cluster)

**Use Cases**:
- Local development and testing
- CI/CD pipeline testing
- Feature development
- RBAC permission testing (less privileged)

**RBAC Permissions**:
- Read access only to resources in agent's namespace
- No access to cluster-scoped resources (nodes, PVs, etc.)
- Limited scope for easier testing and cleanup

**Benefits**:
- Test with least privilege (catches permission issues)
- Fast iteration with latest images
- Minimal resource usage
- Easy cleanup (delete namespace)

**Example**:
```bash
# Deploy broker first
helm install brokkr-broker-dev ../brokkr-broker -f ../brokkr-broker/values/development.yaml

# Deploy agent in same namespace
helm install brokkr-agent-dev . -f values/development.yaml \
  --set broker.url=http://brokkr-broker-dev:3000 \
  --set broker.agentName=dev-test \
  --set broker.clusterName=local-minikube \
  --set broker.pak=$BROKER_PAK
```

### Staging (`values/staging.yaml`)

**Purpose**: Pre-production testing with production-like configuration.

**Key Features**:
- Cluster-wide RBAC (test production permissions)
- Release candidate image tags
- Moderate resources (128Mi-256Mi memory, 50m-200m CPU)
- Full security context (test production security)
- 30-second polling interval
- Info-level logging
- HTTPS broker connection

**Use Cases**:
- Integration testing
- Pre-production validation
- Security testing
- Performance testing

**Example**:
```bash
helm install brokkr-agent-staging . -f values/staging.yaml \
  --set broker.url=https://brokkr-staging.internal \
  --set broker.agentName=staging-cluster-01 \
  --set broker.clusterName=staging-us-east-1 \
  --set broker.pak=$BROKER_PAK
```

## Decision Guide

### Choose production.yaml if:
- Deploying to customer-facing environment
- Need full cluster visibility
- Monitoring multiple namespaces
- Security and compliance scanning required
- High availability is critical

### Choose development.yaml if:
- Working on local machine (Minikube, kind, Docker Desktop)
- Testing agent features
- CI/CD pipeline deployment
- Only need single-namespace visibility
- Want to test with minimal RBAC permissions

### Choose staging.yaml if:
- Testing before production deployment
- Validating cluster-wide RBAC behavior
- Integration testing with staging broker
- Performance testing with realistic settings

## RBAC Comparison

| Feature | Production | Development | Staging |
|---------|-----------|-------------|---------|
| RBAC Scope | Cluster-wide | Namespace-scoped | Cluster-wide |
| Role Type | ClusterRole | Role | ClusterRole |
| Namespace Access | All namespaces | Single namespace | All namespaces |
| Node Access | Yes | No | Yes |
| PV Access | Yes | No | Yes |
| Security | Full hardening | Relaxed | Full hardening |

## Customization

### Use Custom Broker URL

```bash
helm install brokkr-agent . -f values/production.yaml \
  --set broker.url=https://my-broker.example.com \
  --set broker.agentName=my-agent \
  --set broker.clusterName=my-cluster \
  --set broker.pak=$PAK
```

### Adjust Polling Interval

Create custom values file:
```yaml
# my-values.yaml
agent:
  pollingInterval: 60  # Poll every 60 seconds
```

Install with custom values:
```bash
helm install brokkr-agent . \
  -f values/production.yaml \
  -f my-values.yaml \
  --set broker.agentName=my-agent \
  --set broker.clusterName=my-cluster \
  --set broker.pak=$PAK
```

### Add Custom RBAC Permissions

For custom CRDs or additional resources:
```yaml
# custom-rbac.yaml
rbac:
  additionalRules:
    - apiGroups: ["custom.io"]
      resources: ["customresources"]
      verbs: ["get", "list", "watch"]
```

Install with additional permissions:
```bash
helm install brokkr-agent . \
  -f values/production.yaml \
  -f custom-rbac.yaml \
  --set broker.agentName=my-agent \
  --set broker.clusterName=my-cluster \
  --set broker.pak=$PAK
```

### Increase Resources for Large Clusters

For clusters with 1000+ pods or many namespaces:
```yaml
# large-cluster.yaml
resources:
  requests:
    memory: "512Mi"
    cpu: "200m"
  limits:
    memory: "1Gi"
    cpu: "1000m"
```

## Getting Pre-Authenticated Key (PAK)

The PAK is required for agent authentication. Get it from the broker:

```bash
# Method 1: Broker API (if available)
curl -X POST https://brokkr.example.com/api/agents \
  -H "Authorization: Bearer $ADMIN_TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "prod-cluster-01",
    "cluster_name": "production-us-east-1"
  }'

# Response includes PAK
{
  "id": "123",
  "name": "prod-cluster-01",
  "pak": "generated-pre-authenticated-key"
}

# Method 2: Broker UI
# Navigate to broker UI and create new agent
# Copy PAK from creation dialog

# Use PAK in Helm install
export BROKER_PAK="generated-pre-authenticated-key"
helm install brokkr-agent . -f values/production.yaml \
  --set broker.agentName=prod-cluster-01 \
  --set broker.clusterName=production-us-east-1 \
  --set broker.pak=$BROKER_PAK
```

## Validation

Test your values file before deploying:

```bash
# Render templates without installing
helm template brokkr-agent . -f values/production.yaml \
  --set broker.agentName=test \
  --set broker.clusterName=test \
  --set broker.pak=test

# Dry run to check for issues
helm install brokkr-agent . -f values/production.yaml \
  --set broker.agentName=test \
  --set broker.clusterName=test \
  --set broker.pak=test \
  --dry-run

# Install with debug output
helm install brokkr-agent . -f values/production.yaml \
  --set broker.agentName=test \
  --set broker.clusterName=test \
  --set broker.pak=$BROKER_PAK \
  --debug
```

## Troubleshooting

### Agent Not Connecting to Broker

**Symptom**: Agent logs show connection refused or timeout errors

**Solutions**:
1. Verify broker URL is correct:
   ```bash
   kubectl logs -l app=brokkr-agent | grep "broker"
   ```
2. Check broker is running:
   ```bash
   kubectl get pods -l app=brokkr-broker
   ```
3. Test connectivity from agent pod:
   ```bash
   kubectl exec -it deploy/brokkr-agent -- curl -v http://brokkr-broker:3000/health
   ```
4. Verify PAK is correct (check broker logs for authentication failures)

### RBAC Permission Denied

**Symptom**: Agent logs show "forbidden" errors when accessing resources

**Solutions**:
1. Check RBAC mode in values file:
   ```yaml
   rbac:
     clusterWide: true  # or false for namespace-scoped
   ```
2. Verify ClusterRole/Role was created:
   ```bash
   # For cluster-wide
   kubectl get clusterrole brokkr-agent

   # For namespace-scoped
   kubectl get role brokkr-agent
   ```
3. Check RoleBinding/ClusterRoleBinding:
   ```bash
   # For cluster-wide
   kubectl get clusterrolebinding brokkr-agent

   # For namespace-scoped
   kubectl get rolebinding brokkr-agent
   ```
4. Review [RBAC.md](../RBAC.md) for permission details

### Agent Using Too Many Resources

**Symptom**: Agent pod is OOMKilled or CPU throttled

**Solutions**:
1. Check current resource usage:
   ```bash
   kubectl top pod -l app=brokkr-agent
   ```
2. Increase resource limits:
   ```yaml
   resources:
     limits:
       memory: "512Mi"  # Increase from 256Mi
       cpu: "500m"      # Increase from 200m
   ```
3. For very large clusters, consider adjusting polling interval:
   ```yaml
   agent:
     pollingInterval: 60  # Increase from 30
   ```

### Agent Not Polling

**Symptom**: Agent connected but not sending inventory updates

**Solutions**:
1. Check agent logs:
   ```bash
   kubectl logs -l app=brokkr-agent --tail=100
   ```
2. Verify polling interval:
   ```bash
   kubectl get deploy brokkr-agent -o yaml | grep -A2 POLLING_INTERVAL
   ```
3. Check agent status in broker
4. Verify RBAC permissions (agent may be silently failing API calls)

### Wrong RBAC Scope

**Symptom**: Agent not seeing expected resources

**Solutions**:
1. Check if using correct values file:
   ```bash
   # Production should use cluster-wide
   helm get values brokkr-agent
   ```
2. Verify Role type:
   ```bash
   # Should be ClusterRole for production
   kubectl get clusterrole brokkr-agent

   # Should be Role for development
   kubectl get role brokkr-agent
   ```
3. Re-deploy with correct values file if needed:
   ```bash
   helm upgrade brokkr-agent . -f values/production.yaml \
     --set broker.agentName=$NAME \
     --set broker.clusterName=$CLUSTER \
     --set broker.pak=$PAK
   ```

## Additional Resources

- [RBAC Documentation](../RBAC.md) - Detailed RBAC permission documentation
- [Brokkr Agent Documentation](https://docs.brokkr.io/agent) - Full agent documentation
- [Helm Documentation](https://helm.sh/docs/) - Helm usage guide
