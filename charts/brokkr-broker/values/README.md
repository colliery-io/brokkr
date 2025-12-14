# Brokkr Broker Values Files

Pre-configured values files for common deployment scenarios. These files provide sensible defaults for different environments, eliminating the need to manually configure dozens of settings.

## Quick Start

Install the broker with a specific environment configuration:

```bash
# Production deployment
helm install brokkr-broker . -f values/production.yaml

# Development deployment
helm install brokkr-broker-dev . -f values/development.yaml

# Staging deployment
helm install brokkr-broker-staging . -f values/staging.yaml
```

## Available Scenarios

### Production (`values/production.yaml`)

**Purpose**: Production-ready deployment with high availability, security, and reliability.

**Key Features**:
- 3 replicas for high availability
- External managed database (AWS RDS, GCP Cloud SQL, etc.)
- TLS enabled with automatic certificate management
- Ingress configured for external access
- Production resource limits (512Mi-1Gi memory, 500m-1000m CPU)
- Full security hardening (seccomp, non-root, etc.)
- Info-level logging

**Before Deploying**:
1. Set up external PostgreSQL database
2. Create database credentials secret:
   ```bash
   kubectl create secret generic brokkr-db-prod \
     --from-literal=database-url='postgresql://user:pass@host:5432/brokkr_prod'
   ```
3. Configure cert-manager ClusterIssuer for TLS
4. Update `ingress.hosts` with your domain
5. Review and adjust resource limits based on your workload

**Post-Deployment**:
- Access via configured ingress domain (e.g., https://brokkr.example.com)
- Monitor resource usage and adjust limits as needed
- Set up monitoring and alerting

### Development (`values/development.yaml`)

**Purpose**: Local development and testing with minimal dependencies and resource usage.

**Key Features**:
- Single replica
- Bundled PostgreSQL (ephemeral storage)
- No TLS (simplified local access)
- No ingress (use port-forward)
- Minimal resources (128Mi-256Mi memory, 50m-200m CPU)
- Debug-level logging
- Latest image tag with always pull policy

**Usage**:
```bash
# Install
helm install brokkr-broker-dev . -f values/development.yaml

# Access locally
kubectl port-forward svc/brokkr-broker-dev 3000:3000

# Access at http://localhost:3000
```

**Benefits**:
- Quick setup with no external dependencies
- Fast iteration (ephemeral database, always pull latest)
- Minimal resource usage (can run on laptop)
- Verbose logging for debugging

**Limitations**:
- Data is lost on pod restart (ephemeral database)
- No high availability
- Not suitable for load testing

### Staging (`values/staging.yaml`)

**Purpose**: Pre-production testing environment that mirrors production configuration.

**Key Features**:
- 2 replicas (some redundancy)
- External database (dedicated staging instance)
- TLS enabled (self-signed or internal CA)
- Ingress configured
- Moderate resources (256Mi-512Mi memory, 250m-500m CPU)
- Full security contexts (test production security)
- Info-level logging

**Before Deploying**:
1. Set up staging database
2. Create database credentials secret:
   ```bash
   kubectl create secret generic brokkr-db-staging \
     --from-literal=database-url='postgresql://user:pass@staging-host:5432/brokkr_staging'
   ```
3. Create TLS secret (self-signed is fine):
   ```bash
   openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
     -keyout tls.key -out tls.crt \
     -subj "/CN=brokkr-staging.internal.example.com"

   kubectl create secret tls brokkr-tls-staging \
     --cert=tls.crt --key=tls.key
   ```
4. Update `ingress.hosts` with staging domain

**Use Cases**:
- Integration testing
- Performance testing
- Security validation
- Pre-production smoke tests
- Training and demos

## Decision Guide

**Choose production.yaml if**:
- Deploying to customer-facing environment
- Need high availability and reliability
- Have managed database service available
- Security and compliance are critical

**Choose development.yaml if**:
- Working on local machine (Minikube, kind, Docker Desktop)
- Need fast iteration cycles
- Don't need persistent data
- Want minimal setup and dependencies

**Choose staging.yaml if**:
- Testing changes before production
- Need production-like environment
- Validating security configurations
- Running integration tests
- Performance testing

## Customization

These files are starting points. For custom configurations:

1. **Copy an existing file**:
   ```bash
   cp values/production.yaml my-custom-values.yaml
   ```

2. **Modify specific values**:
   Edit `my-custom-values.yaml` with your requirements

3. **Install with custom values**:
   ```bash
   helm install brokkr-broker . -f my-custom-values.yaml
   ```

4. **Combine multiple values files**:
   ```bash
   helm install brokkr-broker . \
     -f values/production.yaml \
     -f my-overrides.yaml
   ```
   Later files override earlier ones.

## Common Customizations

### Change Resource Limits

```yaml
resources:
  requests:
    memory: "1Gi"
    cpu: "1000m"
  limits:
    memory: "2Gi"
    cpu: "2000m"
```

### Add Custom Environment Variables

```yaml
extraEnv:
  - name: CUSTOM_CONFIG
    value: "custom-value"
  - name: FEATURE_FLAG
    value: "true"
```

### Use Different Image Registry

```yaml
image:
  repository: my-registry.example.com/brokkr-broker
  tag: "v1.0.0"
```

### Configure LoadBalancer Service

```yaml
service:
  type: LoadBalancer
  annotations:
    service.beta.kubernetes.io/aws-load-balancer-type: "nlb"
```

## Validation

Test your values file before deploying:

```bash
# Render templates without installing
helm template brokkr-broker . -f values/production.yaml

# Dry run to check for issues
helm install brokkr-broker . -f values/production.yaml --dry-run

# Install with debug output
helm install brokkr-broker . -f values/production.yaml --debug
```

## Troubleshooting

### Database Connection Issues

**Symptom**: Broker pods crash with database connection errors

**Solutions**:
1. Verify database credentials secret exists:
   ```bash
   kubectl get secret brokkr-db-prod
   ```
2. Check database URL format:
   ```
   postgresql://username:password@host:port/database
   ```
3. Verify network connectivity to database

### TLS Certificate Issues

**Symptom**: Ingress returns certificate errors

**Solutions**:
1. Check cert-manager issuer is ready:
   ```bash
   kubectl get clusterissuer letsencrypt-prod
   ```
2. View certificate status:
   ```bash
   kubectl get certificate
   kubectl describe certificate brokkr-tls-prod
   ```
3. Check cert-manager logs:
   ```bash
   kubectl logs -n cert-manager deploy/cert-manager
   ```

### Resource Limits Too Low

**Symptom**: Pods are OOMKilled or CPU throttled

**Solutions**:
1. Check current resource usage:
   ```bash
   kubectl top pod -l app=brokkr-broker
   ```
2. Increase limits in your values file
3. Upgrade release with new limits:
   ```bash
   helm upgrade brokkr-broker . -f values/production.yaml
   ```

### Pods Not Ready

**Symptom**: Pods stuck in Pending or CrashLoopBackOff

**Solutions**:
1. Check pod status and events:
   ```bash
   kubectl describe pod -l app=brokkr-broker
   ```
2. View pod logs:
   ```bash
   kubectl logs -l app=brokkr-broker
   ```
3. Common causes:
   - Insufficient cluster resources (increase node capacity)
   - Image pull errors (check image tag and pull secrets)
   - Database connection failures (verify external.host and secret)

## Additional Resources

- [Helm Values Documentation](../../README.md) - Complete values reference
- [Brokkr Documentation](https://docs.brokkr.io) - Full product documentation
- [Helm Documentation](https://helm.sh/docs/) - Helm usage guide
