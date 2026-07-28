# Brokkr Broker Helm Chart

This Helm chart deploys the Brokkr control plane broker to a Kubernetes cluster.

## Prerequisites

- **Kubernetes 1.23 or later.** Unlike the agent chart, `Chart.yaml` here declares no
  `kubeVersion`, so Helm will not stop you on an older cluster — but the chart will not fully
  work: the bundled Bitnami PostgreSQL subchart (18.x) states Kubernetes 1.23+, and
  `autoscaling.enabled` renders `autoscaling/v2` (1.23+) while `podDisruptionBudget.enabled`
  renders `policy/v1` (1.21+). If you also run the agent chart in the same cluster, its
  `kubeVersion: ">=1.29.0-0"` pin makes **1.29** the effective floor for a Brokkr fleet — see
  [Installing Brokkr](https://github.com/colliery-io/brokkr/blob/main/docs/src/getting-started/installation.md).
- Helm 3.8+ (3.8 is the floor for `helm install oci://…` against the published charts;
  installing from a local checkout works with any Helm 3.x)
- PostgreSQL database (bundled by default, or external)
- (Optional) cert-manager for automatic TLS certificate management **at the ingress** — the
  broker itself never terminates TLS, see [TLS/SSL Configuration](#tlsssl-configuration)
- (Optional) Ingress controller for external access
- (Optional) Prometheus Operator, if you enable `metrics.serviceMonitor`

Unlike the agent chart, this chart installs nothing cluster-wide: every resource it creates
(Deployment, Service, ConfigMap, optional Ingress/HPA/PDB/NetworkPolicy/ServiceMonitor, plus the
PostgreSQL subchart when bundled) lives in the release namespace.

## Installation

### Basic Installation

First mint the admin credential — the command prints a PAK (the secret you authenticate with) and its hash (what the broker stores):

```bash
brokkr-broker generate-pak
# Or without a local binary (the image's entrypoint is the broker):
docker run --rm ghcr.io/colliery-io/brokkr-broker:latest generate-pak
```

Then deploy with default settings (bundled PostgreSQL, ClusterIP service), passing the hash:

```bash
helm install my-broker charts/brokkr-broker \
  --set broker.pakHash=<pak-hash-from-generate-pak>
```

**WARNING:** installing without `broker.pakHash` or `broker.pakHashExistingSecret` leaves the broker running with the publicly-known development admin PAK compiled into the binary — anyone who can reach the broker can authenticate as admin. Always set one of the two, even on dev clusters. Setting `broker.pakHash` to an empty string is the same as omitting it: the chart only renders the variable when the value is non-empty.

### Production Installation

For production, store the admin PAK hash in a pre-created Secret, use external PostgreSQL, and terminate TLS at the ingress (the broker itself serves plain HTTP — see [TLS/SSL Configuration](#tlsssl-configuration)):

```bash
# 1. Mint the admin credential and store its hash in a Secret
docker run --rm ghcr.io/colliery-io/brokkr-broker:latest generate-pak
kubectl create secret generic broker-admin-pak-hash \
  --from-literal=BROKKR__BROKER__PAK_HASH=<pak-hash-from-generate-pak>

# 2. Install
helm install my-broker charts/brokkr-broker \
  --set postgresql.enabled=false \
  --set postgresql.external.host=prod-postgres.example.com \
  --set postgresql.external.password=secure-password \
  --set broker.pakHashExistingSecret=broker-admin-pak-hash \
  --set ingress.enabled=true \
  --set ingress.hosts[0].host=broker.example.com \
  --set ingress.tls[0].secretName=broker-tls \
  --set ingress.tls[0].hosts[0]=broker.example.com
```

## Configuration

### Admin Credential (Day-Zero Bootstrap)

On its true first startup the broker stores an admin PAK hash from `BROKKR__BROKER__PAK_HASH`. The chart provides two ways to set it:

```yaml
broker:
  # Dev/test: rendered into the ConfigMap in plaintext
  pakHash: '<pak-hash-from-generate-pak>'

  # Production: sourced from a pre-existing Secret via secretKeyRef
  pakHashExistingSecret: broker-admin-pak-hash
  pakHashExistingSecretKey: BROKKR__BROKER__PAK_HASH  # default
```

When `pakHashExistingSecret` is set it takes precedence and `pakHash` is ignored. When **neither** is set, the chart omits `BROKKR__BROKER__PAK_HASH` entirely and the broker falls back to the publicly-known development hash embedded in the binary — the matching PAK is public, so never run this way outside a throwaway environment. An empty `pakHash` behaves identically to an unset one.

Notes on behavior:

- The hash is a bare 64-character hex SHA-256 digest, exactly as `generate-pak` prints it (no `sha256:` prefix).
- The broker only self-generates a PAK (writing it to `/tmp/brokkr-keys/key.txt` in the pod, deleted on graceful shutdown) when `BROKKR__BROKER__PAK_HASH` is present in its environment but empty — something the chart values cannot produce. With a hash configured, no key file is ever written.
- To replace a lost admin PAK, mint a new pair with `generate-pak`, update the configured hash (value or Secret), and run `brokkr-broker rotate admin` in the pod; a plain restart does not re-run the bootstrap.

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

#### Multi-Tenant Deployments (Schema Isolation)

For multi-tenant deployments, multiple broker instances can share a single PostgreSQL database by using different schemas. Each broker instance operates in complete isolation within its own PostgreSQL schema.

**Use Cases:**
- Multiple environments (dev/staging/prod) sharing one database
- Multi-customer SaaS deployments with data isolation
- Cost-efficient infrastructure with PostgreSQL-enforced isolation

**Configuration:**

```yaml
postgresql:
  enabled: false
  external:
    host: shared-postgres.example.com
    port: 5432
    database: brokkr
    username: brokkr
    password: secure-password
    schema: tenant_a  # Schema for data isolation
```

**Example: Multi-Environment Setup**

Deploy three broker instances to different namespaces, all using the same PostgreSQL. Mint a separate admin credential per instance with `generate-pak` (see [Admin Credential](#admin-credential-day-zero-bootstrap)) — each schema gets its own admin PAK:

```bash
# Development environment
helm install dev-broker charts/brokkr-broker \
  --namespace dev \
  --set postgresql.enabled=false \
  --set postgresql.external.host=shared-postgres.example.com \
  --set postgresql.external.schema=brokkr_dev \
  --set broker.pakHash=<dev-pak-hash>

# Staging environment
helm install staging-broker charts/brokkr-broker \
  --namespace staging \
  --set postgresql.enabled=false \
  --set postgresql.external.host=shared-postgres.example.com \
  --set postgresql.external.schema=brokkr_staging \
  --set broker.pakHash=<staging-pak-hash>

# Production environment (prefer a Secret over a plaintext value)
helm install prod-broker charts/brokkr-broker \
  --namespace production \
  --set postgresql.enabled=false \
  --set postgresql.external.host=shared-postgres.example.com \
  --set postgresql.external.schema=brokkr_prod \
  --set broker.pakHashExistingSecret=prod-admin-pak-hash
```

**Schema Provisioning:**

Each schema must be created before the broker starts. The broker will automatically run migrations within its configured schema:

```sql
-- Connect to PostgreSQL as admin
CREATE SCHEMA IF NOT EXISTS brokkr_dev;
CREATE SCHEMA IF NOT EXISTS brokkr_staging;
CREATE SCHEMA IF NOT EXISTS brokkr_prod;

-- Grant permissions to broker user
GRANT ALL PRIVILEGES ON SCHEMA brokkr_dev TO brokkr;
GRANT ALL PRIVILEGES ON SCHEMA brokkr_staging TO brokkr;
GRANT ALL PRIVILEGES ON SCHEMA brokkr_prod TO brokkr;
```

**Data Isolation:**

- Each broker sees only its own schema's data
- PostgreSQL enforces isolation at the schema level
- No application-level filtering required
- Impossible to accidentally query across tenants

**Backward Compatibility:**

When `schema` is not set (or empty string), the broker uses the default `public` schema. This maintains compatibility with existing single-tenant deployments.

### TLS/SSL Configuration

**The broker does not terminate TLS.** The binary serves plain HTTP on port 3000 and reads no TLS configuration. All TLS for broker traffic must terminate in front of the broker — normally at the ingress, or at a service mesh / reverse proxy.

**REMOVED — the chart's `tls.*` values no longer exist.** Earlier chart versions accepted `tls.enabled`, `tls.existingSecret`, `tls.cert`/`tls.key`, and `tls.certManager.*`. Those values only mounted certificate files into the pod and set `BROKKR__TLS__*` environment variables that the broker never reads — they never encrypted anything, so they have been removed rather than left as a false signal. Helm ignores stale `tls.*` entries left in your values files, but you should delete them.

**If you were relying on them:** any broker traffic you believed was encrypted — including PAKs in `Authorization` headers — was plaintext unless a TLS-terminating ingress or proxy was already in front. Move TLS to the ingress as shown below, and rotate any PAKs that transited untrusted networks.

#### Method 1: Ingress with an Existing TLS Secret

Create the secret and reference it from the ingress:

```bash
kubectl create secret tls my-tls-secret \
  --cert=path/to/tls.crt \
  --key=path/to/tls.key
```

```yaml
ingress:
  enabled: true
  className: nginx
  hosts:
    - host: brokkr.example.com
      paths:
        - path: /
          pathType: Prefix
  tls:
    - secretName: my-tls-secret
      hosts:
        - brokkr.example.com
```

#### Method 2: Ingress with cert-manager (Recommended for Production)

Use cert-manager for automatic certificate generation and renewal, driven by an ingress annotation:

```yaml
ingress:
  enabled: true
  className: nginx
  annotations:
    cert-manager.io/cluster-issuer: letsencrypt-prod
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

The broker runs as a non-root user by default. Note the keys are `podSecurityContext` and
`containerSecurityContext` — there is no top-level `securityContext` value:

```yaml
podSecurityContext:
  runAsNonRoot: true
  runAsUser: 10001
  runAsGroup: 10001
  fsGroup: 10001
  # seccompProfile:          # recommended for production
  #   type: RuntimeDefault

containerSecurityContext:
  allowPrivilegeEscalation: false
  readOnlyRootFilesystem: false   # true is safe: /tmp is an emptyDir
  runAsNonRoot: true
  runAsUser: 10001
  capabilities:
    drop: [ALL]
```

Each object is rendered verbatim, so overriding one replaces it wholesale — restate any
defaults you want to keep.

## Migration notes

Two values were renamed or removed because neither ever had an effect. Helm ignores stale entries
left in your values files, but you should delete them. (For the earlier removal of `tls.*`, see
[TLS/SSL Configuration](#tlsssl-configuration).)

| Old key | New key | Notes |
|---------|---------|-------|
| `metrics.enabled` | `networkPolicy.allowMetricsScraping` | The old name implied it turned the `/metrics` endpoint on or off. It never did; the broker always serves `/metrics`. Its only effect was gating the NetworkPolicy ingress rule, which is what the new name says. Default is unchanged (`true`). |
| `telemetry.collector.*` | *(removed)* | The chart never rendered a collector sidecar. Setting `telemetry.collector.enabled: true` only repointed `BROKKR__TELEMETRY__OTLP_ENDPOINT` at `http://localhost:4317`, where nothing was listening — it silently broke otherwise-working telemetry. `BROKKR__TELEMETRY__OTLP_ENDPOINT` now always renders from `telemetry.otlpEndpoint`. |

`service.annotations` is not a migration — it was documented but rendered nowhere, and now renders
onto the Service's `metadata.annotations`. If you had set it, the annotations will appear on the
next upgrade.

## Values

This table covers **every key in `values.yaml`** for chart version 0.8.4, plus the keys the
templates read that `values.yaml` only mentions in comments (marked *not in values.yaml*).
It is hand-maintained, so `values.yaml` remains the authoritative source — check it with
`helm show values charts/brokkr-broker` (or `oci://ghcr.io/colliery-io/charts/brokkr-broker`)
if this table and the chart ever disagree. There is no `tls.*` section: those values were
removed, see [TLS/SSL Configuration](#tlsssl-configuration).

### Image and workload

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `image.repository` | string | `"ghcr.io/colliery-io/brokkr-broker"` | Container image repository |
| `image.tag` | string | `"latest"` | Container image tag. Pin a release tag in production. |
| `image.pullPolicy` | string | `"IfNotPresent"` | Image pull policy |
| `image.pullSecrets` | array | unset (commented in `values.yaml`) | `imagePullSecrets` entries for private registries, e.g. `[{name: ghcr-secret}]` |
| `replicaCount` | int | `1` | Number of broker replicas. Ignored while `autoscaling.enabled` is true. |
| `nameOverride` | string | unset (*not in values.yaml*) | Overrides the chart name portion of generated resource names |
| `fullnameOverride` | string | unset (*not in values.yaml*) | Overrides the full generated resource name |
| `extraEnv` | array | `[]` | Extra container env entries appended verbatim (plain `value` or `valueFrom`). The escape hatch for settings the chart does not expose. |

### Service and ingress

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `service.type` | string | `"ClusterIP"` | Service type (`ClusterIP`, `NodePort`, `LoadBalancer`) |
| `service.port` | int | `3000` | Service port; targets the container's `http` port |
| `service.annotations` | object | `{}` | Annotations set on the Service's `metadata`. This is where LoadBalancer configuration goes (e.g. `service.beta.kubernetes.io/aws-load-balancer-type: "nlb"`). Omitted from the manifest when empty. |
| `service.nodePort` | int | unset (*not in values.yaml*) | Explicit node port; only honoured when `service.type: NodePort` |
| `ingress.enabled` | bool | `false` | Create an Ingress |
| `ingress.className` | string | `"nginx"` | `ingressClassName` |
| `ingress.annotations` | object | `{}` | Ingress annotations — where cert-manager (`cert-manager.io/cluster-issuer`) and SSL-redirect settings go |
| `ingress.hosts` | array | `[{host: brokkr.example.com, paths: [{path: /, pathType: Prefix}]}]` | Ingress hosts and paths |
| `ingress.tls` | array | `[{secretName: brokkr-tls, hosts: [brokkr.example.com]}]` | Ingress TLS — the **only** supported TLS path, since the broker serves plain HTTP |

### Database

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `postgresql.enabled` | bool | `true` | Deploy the bundled Bitnami PostgreSQL subchart. Set `false` for an external database. |
| `postgresql.external.host` | string | `''` | External PostgreSQL host (required when `postgresql.enabled: false`) |
| `postgresql.external.port` | int | `5432` | External PostgreSQL port; also used for the NetworkPolicy egress rule |
| `postgresql.external.database` | string | `"brokkr"` | Database name |
| `postgresql.external.username` | string | `"brokkr"` | Database username (unused when `postgresql.existingSecret` is set) |
| `postgresql.external.password` | string | `"brokkr"` | Database password. Rendered into the ConfigMap inside `BROKKR__DATABASE__URL` in plaintext — prefer `postgresql.existingSecret`. |
| `postgresql.external.schema` | string | `''` | PostgreSQL schema (`BROKKR__DATABASE__SCHEMA`) for multi-tenant isolation; empty means `public` |
| `postgresql.existingSecret` | string | `''` | Name of a pre-existing Secret holding the full connection URL. When set, `BROKKR__DATABASE__URL` is injected via `secretKeyRef` and omitted from the ConfigMap. Applies to bundled and external alike. |
| `postgresql.existingSecretKey` | string | `"database-url"` | Key within that Secret holding the connection URL |
| `postgresql.auth.username` | string | `"brokkr"` | Bundled PostgreSQL username (subchart value; also used to build the connection URL) |
| `postgresql.auth.password` | string | `"brokkr"` | Bundled PostgreSQL password. **Change this or use `existingSecret` — the default is a published credential.** |
| `postgresql.auth.database` | string | `"brokkr"` | Bundled PostgreSQL database name |
| `postgresql.primary.persistence.enabled` | bool | `true` | Bundled PostgreSQL persistence. `false` means data is lost on pod restart. |
| `postgresql.primary.persistence.storageClass` | string | `""` | PVC storage class; empty uses the cluster default |
| `postgresql.primary.persistence.size` | string | `"8Gi"` | PVC size |
| `postgresql.primary.resources.requests.memory` / `.cpu` | string | `"256Mi"` / `"250m"` | Bundled PostgreSQL requests |
| `postgresql.primary.resources.limits.memory` / `.cpu` | string | `"512Mi"` / `"500m"` | Bundled PostgreSQL limits |

Everything under `postgresql.auth` and `postgresql.primary` is passed to the
[Bitnami PostgreSQL subchart](https://github.com/bitnami/charts/tree/main/bitnami/postgresql),
which accepts many more keys than the chart's `values.yaml` lists.

### Broker application settings

`configReload` marks which of these apply without a restart; see the header comment in
`values.yaml` for the authoritative hot-reload list.

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `broker.logLevel` | string | `"info"` | `BROKKR__LOG__LEVEL` (`trace`/`debug`/`info`/`warn`/`error`). Hot-reloadable. The binary's own default is `debug`, so running outside the chart is noisier. |
| `broker.diagnosticCleanupIntervalSeconds` | int | `900` | How often diagnostics are pruned. Hot-reloadable. |
| `broker.diagnosticMaxAgeHours` | int | `1` | Diagnostic retention. Hot-reloadable. |
| `broker.webhookDeliveryIntervalSeconds` | int | `5` | Webhook delivery worker poll interval. Read once at worker start — **requires a pod restart**. |
| `broker.webhookDeliveryBatchSize` | int | `50` | Deliveries claimed per pass. Read once at worker start — **requires a pod restart**. |
| `broker.webhookCleanupRetentionDays` | int | `7` | Delivery-record retention. Read once at cleanup-worker start — **requires a pod restart**. |
| `broker.webhookEncryptionKey` | string | unset (commented in `values.yaml`) | Hex-encoded 32-byte key protecting webhook secrets at rest. Rendered into the ConfigMap in plaintext (dev/test only); ignored when `webhookEncryptionKeyExistingSecret` is set. **Unset means a fresh random key each boot, making previously encrypted webhook data unreadable after a restart.** |
| `broker.webhookEncryptionKeyExistingSecret` | string | `""` | Name of a pre-existing Secret to source the webhook encryption key from. Injected via `secretKeyRef`, kept out of the ConfigMap/values/git (GitOps-friendly). |
| `broker.webhookEncryptionKeyExistingSecretKey` | string | `"BROKKR__BROKER__WEBHOOK_ENCRYPTION_KEY"` | Key within that Secret holding the webhook encryption key |
| `broker.pakHash` | string | unset (commented in `values.yaml`) | Admin PAK hash for day-zero bootstrap. Rendered into the ConfigMap in plaintext (dev/test only); ignored when `pakHashExistingSecret` is set. **Unset or empty leaves the publicly-known development admin PAK active** — see [Admin Credential](#admin-credential-day-zero-bootstrap). |
| `broker.pakHashExistingSecret` | string | `""` | Name of a pre-existing Secret to source the admin PAK hash from. Injected via `secretKeyRef`. |
| `broker.pakHashExistingSecretKey` | string | `"BROKKR__BROKER__PAK_HASH"` | Key within that Secret holding the admin PAK hash |
| `configReload.enabled` | bool | `true` | Sets `BROKKR_CONFIG_WATCHER_ENABLED`, so the broker watches its own ConfigMap and reloads hot-reloadable settings. Note the Deployment sets no `serviceAccountName` and the chart creates no RBAC, so the pod runs as the namespace `default` ServiceAccount — grant it `get`/`watch` on ConfigMaps separately if the watcher must work. |
| `configReload.debounceSeconds` | int | `5` | Debounce window between successive reloads |

### CORS

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `cors.allowedOrigins` | array | `["*"]` | `BROKKR__CORS__ALLOWED_ORIGINS` (joined with commas). Hot-reloadable. **The default allows every origin — narrow it for any internet-reachable broker.** |
| `cors.allowedMethods` | array | `[GET, POST, PUT, PATCH, DELETE, OPTIONS]` | Allowed HTTP methods |
| `cors.allowedHeaders` | array | `[Authorization, Content-Type, Accept]` | Allowed request headers |
| `cors.maxAgeSeconds` | int | `3600` | Preflight cache lifetime. Hot-reloadable. |

### Scaling and availability

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `autoscaling.enabled` | bool | `false` | Create a HorizontalPodAutoscaler (`autoscaling/v2`); overrides `replicaCount` |
| `autoscaling.minReplicas` | int | `1` | HPA minimum replicas |
| `autoscaling.maxReplicas` | int | `5` | HPA maximum replicas |
| `autoscaling.targetCPUUtilizationPercentage` | int | `70` | CPU target for scaling |
| `autoscaling.targetMemoryUtilizationPercentage` | int | unset (commented in `values.yaml`) | Optional memory target |
| `autoscaling.behavior` | object | unset (commented in `values.yaml`) | HPA `behavior` block, passed through verbatim |
| `podDisruptionBudget.enabled` | bool | `false` | Create a PodDisruptionBudget (`policy/v1`) |
| `podDisruptionBudget.minAvailable` | int/string | `1` | Minimum available pods during voluntary disruptions. With `replicaCount: 1` this blocks node drains — pair PDBs with more than one replica. |
| `podDisruptionBudget.maxUnavailable` | int/string | unset (commented in `values.yaml`) | Alternative to `minAvailable`; set only one |

### Resources and security context

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `resources.requests.memory` | string | `"256Mi"` | Memory request |
| `resources.requests.cpu` | string | `"100m"` | CPU request |
| `resources.limits.memory` | string | `"512Mi"` | Memory limit |
| `resources.limits.cpu` | string | `"500m"` | CPU limit |
| `apparmor.enabled` | bool | `false` | Add the `container.apparmor.security.beta.kubernetes.io/broker` pod annotation. Requires AppArmor-enabled nodes. |
| `apparmor.profile` | string | `"runtime/default"` | AppArmor profile referenced by that annotation |
| `podSecurityContext` | object | `{runAsNonRoot: true, runAsUser: 10001, runAsGroup: 10001, fsGroup: 10001}` | Rendered verbatim as the pod `securityContext`. Replacing this object replaces all of it — re-state the defaults you want to keep. `seccompProfile: {type: RuntimeDefault}` is commented out in `values.yaml` and recommended for hardening. |
| `containerSecurityContext` | object | `{allowPrivilegeEscalation: false, readOnlyRootFilesystem: false, runAsNonRoot: true, runAsUser: 10001, capabilities: {drop: [ALL]}}` | Rendered verbatim as the container `securityContext`. The chart already mounts an `emptyDir` at `/tmp`, so `readOnlyRootFilesystem: true` is the hardened setting. |

### Metrics

The broker serves `/metrics` on its HTTP port. The endpoint is always served; no chart value
disables it.

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `metrics.serviceMonitor.enabled` | bool | `false` | Create a Prometheus Operator `ServiceMonitor` scraping `/metrics` on the Service's `http` port |
| `metrics.serviceMonitor.interval` | string | `"30s"` | Scrape interval |
| `metrics.serviceMonitor.scrapeTimeout` | string | unset (commented in `values.yaml`) | Scrape timeout; must be shorter than `interval` |
| `metrics.serviceMonitor.additionalLabels` | object | unset (commented in `values.yaml`) | Extra labels so your Prometheus `serviceMonitorSelector` matches |

### NetworkPolicy

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `networkPolicy.enabled` | bool | `false` | Create a NetworkPolicy for the broker pod (Ingress + Egress) |
| `networkPolicy.allowIngressFrom` | array | `[]` | Selectors allowed to reach the broker's API port. Empty means any pod in the same namespace. |
| `networkPolicy.allowMetricsScraping` | bool | `true` | Whether the metrics ingress rule is rendered at all. Set `false` to deny scraping outright even when `allowMetricsFrom` is populated. Does **not** turn off the `/metrics` endpoint — the broker always serves it. |
| `networkPolicy.allowMetricsFrom` | array | `[]` | Selectors allowed to scrape metrics. Only applied when `allowMetricsScraping` is true. |
| `networkPolicy.allowWebhookEgress` | bool | `true` | Permit egress so webhook deliveries can reach external HTTPS endpoints. Setting `false` breaks outbound webhooks. |
| `networkPolicy.additionalEgressRules` | array | `[]` | Extra egress rules appended verbatim |

### Telemetry (OpenTelemetry tracing)

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `telemetry.enabled` | bool | `false` | Set `BROKKR__TELEMETRY__ENABLED`; also sets the service name and sampling rate. Requires a pod restart to change. |
| `telemetry.otlpEndpoint` | string | `"http://otel-collector:4317"` | OTLP/gRPC endpoint for trace export |
| `telemetry.samplingRate` | float | `0.1` | Fraction of traces sampled (0.0–1.0) |

The chart does not run an OpenTelemetry Collector sidecar; point `telemetry.otlpEndpoint` at a
collector you run yourself (or at your service mesh's OTLP receiver). See
[Migration notes](#migration-notes) if you have `telemetry.collector.*` in your values.

## Examples

### Development Setup

```bash
brokkr-broker generate-pak   # save the printed PAK and hash

helm install dev-broker charts/brokkr-broker \
  --set broker.pakHash=<pak-hash-from-generate-pak>
```

This deploys with:
- Bundled PostgreSQL (ephemeral or persistent based on values)
- ClusterIP service (internal only)
- Plain HTTP (the broker never terminates TLS; add an ingress for HTTPS)
- Default resource limits
- Your own admin PAK hash (never rely on the built-in development default)

### Production Setup with Let's Encrypt

```bash
# Mint the admin credential and store its hash in a Secret
docker run --rm ghcr.io/colliery-io/brokkr-broker:latest generate-pak
kubectl create secret generic broker-admin-pak-hash \
  --from-literal=BROKKR__BROKER__PAK_HASH=<pak-hash>

helm install prod-broker charts/brokkr-broker \
  --set postgresql.enabled=false \
  --set postgresql.existingSecret=prod-db-secret \
  --set broker.pakHashExistingSecret=broker-admin-pak-hash \
  --set ingress.enabled=true \
  --set ingress.annotations."cert-manager\.io/cluster-issuer"=letsencrypt-prod \
  --set ingress.hosts[0].host=broker.example.com \
  --set ingress.hosts[0].paths[0].path=/ \
  --set ingress.hosts[0].paths[0].pathType=Prefix \
  --set ingress.tls[0].secretName=broker-tls \
  --set ingress.tls[0].hosts[0]=broker.example.com
```

TLS terminates at the ingress; cert-manager creates and renews the `broker-tls` secret.

### Production Setup with Existing Certificates

```bash
# Create TLS secret for the ingress
kubectl create secret tls broker-tls \
  --cert=broker.crt \
  --key=broker.key

# Mint the admin credential and store its hash in a Secret
docker run --rm ghcr.io/colliery-io/brokkr-broker:latest generate-pak
kubectl create secret generic broker-admin-pak-hash \
  --from-literal=BROKKR__BROKER__PAK_HASH=<pak-hash>

# Install chart (TLS terminates at the ingress)
helm install prod-broker charts/brokkr-broker \
  --set postgresql.enabled=false \
  --set postgresql.existingSecret=prod-db-secret \
  --set broker.pakHashExistingSecret=broker-admin-pak-hash \
  --set ingress.enabled=true \
  --set ingress.className=nginx \
  --set ingress.hosts[0].host=broker.example.com \
  --set ingress.tls[0].secretName=broker-tls \
  --set ingress.tls[0].hosts[0]=broker.example.com
```

### Multi-Tenant Setup (Schema Isolation)

Deploy multiple broker instances sharing a single PostgreSQL database with schema-based isolation:

```bash
# Create database secret (shared by all tenants)
kubectl create secret generic shared-db-secret \
  --from-literal=database-url='postgres://brokkr:password@postgres.example.com:5432/brokkr'

# Mint a separate admin credential per tenant (one generate-pak run each)
docker run --rm ghcr.io/colliery-io/brokkr-broker:latest generate-pak  # tenant A
docker run --rm ghcr.io/colliery-io/brokkr-broker:latest generate-pak  # tenant B
kubectl create secret generic tenant-a-admin-pak-hash --namespace tenant-a \
  --from-literal=BROKKR__BROKER__PAK_HASH=<tenant-a-pak-hash>
kubectl create secret generic tenant-b-admin-pak-hash --namespace tenant-b \
  --from-literal=BROKKR__BROKER__PAK_HASH=<tenant-b-pak-hash>

# Deploy tenant A broker
helm install tenant-a-broker charts/brokkr-broker \
  --namespace tenant-a \
  --set postgresql.enabled=false \
  --set postgresql.external.schema=tenant_a \
  --set postgresql.existingSecret=shared-db-secret \
  --set broker.pakHashExistingSecret=tenant-a-admin-pak-hash \
  --set ingress.enabled=true \
  --set ingress.hosts[0].host=tenant-a.example.com

# Deploy tenant B broker
helm install tenant-b-broker charts/brokkr-broker \
  --namespace tenant-b \
  --set postgresql.enabled=false \
  --set postgresql.external.schema=tenant_b \
  --set postgresql.existingSecret=shared-db-secret \
  --set broker.pakHashExistingSecret=tenant-b-admin-pak-hash \
  --set ingress.enabled=true \
  --set ingress.hosts[0].host=tenant-b.example.com
```

Without a per-tenant `broker.pakHash` / `broker.pakHashExistingSecret`, every tenant shares the publicly-known development admin PAK.

**Note:** Ensure schemas are created in PostgreSQL before deploying:

```sql
CREATE SCHEMA IF NOT EXISTS tenant_a;
CREATE SCHEMA IF NOT EXISTS tenant_b;
GRANT ALL PRIVILEGES ON SCHEMA tenant_a TO brokkr;
GRANT ALL PRIVILEGES ON SCHEMA tenant_b TO brokkr;
```

## Troubleshooting

### Certificate Issues

If HTTPS access through the ingress fails with certificate errors (the broker itself never uses these certificates):

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
