# Brokkr Broker Helm Chart

This Helm chart deploys the Brokkr control plane broker to a Kubernetes cluster.

## Prerequisites

- Kubernetes 1.19+
- Helm 3.0+
- PostgreSQL database (bundled by default, or external)
- (Optional) cert-manager for automatic TLS certificate management
- (Optional) Ingress controller for external access

## Installation

### Basic Installation

Deploy with default settings (bundled PostgreSQL, no TLS, ClusterIP service):

```bash
helm install my-broker charts/brokkr-broker
```

### Production Installation

For production, use external PostgreSQL and enable TLS:

```bash
helm install my-broker charts/brokkr-broker \
  --set postgresql.enabled=false \
  --set postgresql.external.host=prod-postgres.example.com \
  --set postgresql.external.password=secure-password \
  --set tls.enabled=true \
  --set tls.existingSecret=my-tls-secret
```

## Configuration

### Database Configuration

#### Bundled PostgreSQL (Development/Testing)

The chart includes a PostgreSQL subchart from Bitnami that's enabled by default:

```yaml
postgresql:
  enabled: true
  auth:
    username: brokkr
    password: brokkr  # Change in production!
    database: brokkr
  primary:
    persistence:
      enabled: true
      size: 8Gi
```

#### External PostgreSQL (Production)

For production workloads, use an external PostgreSQL instance:

```yaml
postgresql:
  enabled: false
  external:
    host: postgres.example.com
    port: 5432
    database: brokkr
    username: brokkr
    password: secure-password
```

Or use an existing secret:

```yaml
postgresql:
  enabled: false
  existingSecret: my-db-secret
  existingSecretKey: database-url
```

The secret should contain a key with the full PostgreSQL connection URL:
```
postgres://username:password@host:port/database
```

### TLS/SSL Configuration

The chart supports multiple methods for configuring TLS certificates.

#### Method 1: Existing Kubernetes Secret (Recommended for Production)

Use a pre-existing Kubernetes TLS secret:

```yaml
tls:
  enabled: true
  existingSecret: my-tls-secret
```

Create the secret manually:

```bash
kubectl create secret tls my-tls-secret \
  --cert=path/to/tls.crt \
  --key=path/to/tls.key
```

#### Method 2: Inline Certificates (Testing Only)

Provide base64-encoded certificates inline:

```yaml
tls:
  enabled: true
  cert: "LS0tLS1CRUdJTi..."  # base64-encoded certificate
  key: "LS0tLS1CRUdJTi..."   # base64-encoded private key
```

Generate self-signed certificates for testing:

```bash
# Generate certificate and key
openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
  -keyout tls.key -out tls.crt \
  -subj "/CN=brokkr.example.com"

# Base64 encode for values file
cat tls.crt | base64
cat tls.key | base64
```

**WARNING:** Inline certificates are not recommended for production. Use `existingSecret` instead.

#### Method 3: cert-manager (Recommended for Production)

Use cert-manager for automatic certificate generation and renewal:

```yaml
tls:
  enabled: true
  certManager:
    enabled: true
    issuer: letsencrypt-prod
    issuerKind: ClusterIssuer

ingress:
  enabled: true
  className: nginx
  hosts:
    - host: brokkr.example.com
      paths:
        - path: /
          pathType: Prefix
  tls:
    - secretName: brokkr-tls  # cert-manager will create this
      hosts:
        - brokkr.example.com
```

Prerequisites for cert-manager:
1. Install cert-manager in your cluster
2. Create a ClusterIssuer or Issuer:

```yaml
apiVersion: cert-manager.io/v1
kind: ClusterIssuer
metadata:
  name: letsencrypt-prod
spec:
  acme:
    server: https://acme-v02.api.letsencrypt.org/directory
    email: admin@example.com
    privateKeySecretRef:
      name: letsencrypt-prod
    solvers:
    - http01:
        ingress:
          class: nginx
```

### Ingress Configuration

Enable external access via Kubernetes Ingress:

```yaml
ingress:
  enabled: true
  className: nginx
  annotations:
    cert-manager.io/cluster-issuer: "letsencrypt-prod"
    nginx.ingress.kubernetes.io/ssl-redirect: "true"
  hosts:
    - host: brokkr.example.com
      paths:
        - path: /
          pathType: Prefix
  tls:
    - secretName: brokkr-tls
      hosts:
        - brokkr.example.com
```

### Resource Configuration

Configure resource requests and limits:

```yaml
resources:
  requests:
    memory: 256Mi
    cpu: 100m
  limits:
    memory: 512Mi
    cpu: 500m
```

### Security Context

The broker runs as a non-root user by default:

```yaml
securityContext:
  runAsNonRoot: true
  runAsUser: 10001
  fsGroup: 10001
```

## Values

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `image.repository` | string | `"ghcr.io/colliery-io/brokkr-broker"` | Container image repository |
| `image.tag` | string | `"latest"` | Container image tag |
| `image.pullPolicy` | string | `"IfNotPresent"` | Image pull policy |
| `replicaCount` | int | `1` | Number of broker replicas |
| `service.type` | string | `"ClusterIP"` | Kubernetes service type |
| `service.port` | int | `3000` | Service port |
| `postgresql.enabled` | bool | `true` | Enable bundled PostgreSQL |
| `postgresql.external.host` | string | `""` | External PostgreSQL host |
| `postgresql.external.port` | int | `5432` | External PostgreSQL port |
| `postgresql.external.database` | string | `"brokkr"` | Database name |
| `postgresql.external.username` | string | `"brokkr"` | Database username |
| `postgresql.external.password` | string | `"brokkr"` | Database password |
| `postgresql.existingSecret` | string | `""` | Existing secret for database URL |
| `tls.enabled` | bool | `false` | Enable TLS/SSL |
| `tls.existingSecret` | string | `""` | Existing TLS secret name |
| `tls.cert` | string | `""` | Base64-encoded certificate (testing only) |
| `tls.key` | string | `""` | Base64-encoded private key (testing only) |
| `tls.certManager.enabled` | bool | `false` | Enable cert-manager integration |
| `tls.certManager.issuer` | string | `"letsencrypt-prod"` | cert-manager issuer name |
| `tls.certManager.issuerKind` | string | `"ClusterIssuer"` | Issuer kind |
| `ingress.enabled` | bool | `false` | Enable ingress |
| `ingress.className` | string | `"nginx"` | Ingress class name |
| `ingress.annotations` | object | `{}` | Ingress annotations |
| `ingress.hosts` | array | See values.yaml | Ingress host configuration |
| `ingress.tls` | array | See values.yaml | Ingress TLS configuration |
| `resources.requests.memory` | string | `"256Mi"` | Memory request |
| `resources.requests.cpu` | string | `"100m"` | CPU request |
| `resources.limits.memory` | string | `"512Mi"` | Memory limit |
| `resources.limits.cpu` | string | `"500m"` | CPU limit |

## Examples

### Development Setup

```bash
helm install dev-broker charts/brokkr-broker
```

This deploys with:
- Bundled PostgreSQL (ephemeral or persistent based on values)
- ClusterIP service (internal only)
- No TLS
- Default resource limits

### Production Setup with Let's Encrypt

```bash
helm install prod-broker charts/brokkr-broker \
  --set postgresql.enabled=false \
  --set postgresql.existingSecret=prod-db-secret \
  --set tls.enabled=true \
  --set tls.certManager.enabled=true \
  --set tls.certManager.issuer=letsencrypt-prod \
  --set ingress.enabled=true \
  --set ingress.hosts[0].host=broker.example.com \
  --set ingress.hosts[0].paths[0].path=/ \
  --set ingress.hosts[0].paths[0].pathType=Prefix
```

### Production Setup with Existing Certificates

```bash
# Create TLS secret
kubectl create secret tls broker-tls \
  --cert=broker.crt \
  --key=broker.key

# Install chart
helm install prod-broker charts/brokkr-broker \
  --set postgresql.enabled=false \
  --set postgresql.existingSecret=prod-db-secret \
  --set tls.enabled=true \
  --set tls.existingSecret=broker-tls \
  --set ingress.enabled=true \
  --set ingress.className=nginx \
  --set ingress.hosts[0].host=broker.example.com
```

## Troubleshooting

### Certificate Issues

If pods fail to start with certificate errors:

1. Verify the secret exists and contains valid certificate data:
```bash
kubectl get secret <tls-secret-name> -o yaml
```

2. Check that the certificate has not expired:
```bash
kubectl get secret <tls-secret-name> -o jsonpath='{.data.tls\.crt}' | base64 -d | openssl x509 -noout -dates
```

3. Ensure the certificate matches the expected hostname:
```bash
kubectl get secret <tls-secret-name> -o jsonpath='{.data.tls\.crt}' | base64 -d | openssl x509 -noout -text | grep DNS
```

### Database Connection Issues

Check the database URL configuration:

```bash
kubectl get configmap <release-name>-brokkr-broker -o yaml
```

Verify database connectivity from a pod:

```bash
kubectl run -it --rm debug --image=postgres:16 --restart=Never -- psql <database-url>
```

### Viewing Logs

```bash
kubectl logs -l app.kubernetes.io/name=brokkr-broker --tail=100 -f
```

## Uninstallation

```bash
helm uninstall my-broker
```

Note: This does not delete PersistentVolumeClaims created by the PostgreSQL subchart. Delete them manually if needed:

```bash
kubectl delete pvc -l app.kubernetes.io/instance=my-broker
```
