# Brokkr Agent Helm Chart

This Helm chart deploys the Brokkr agent to a Kubernetes cluster. The agent connects to a Brokkr broker and reports cluster state.

## Prerequisites

- **Kubernetes 1.29 or later.** `Chart.yaml` declares `kubeVersion: ">=1.29.0-0"`, so
  Helm refuses to install or upgrade this chart on anything older:

  ```
  Error: chart requires kubeVersion: >=1.29.0-0 which is incompatible with Kubernetes v1.28.0
  ```

  The floor was introduced together with the Shipwright Build integration and is
  documented in `Chart.yaml` as the Kubernetes baseline that integration requires
  (the same 1.29 floor is stated in
  [Shipwright Builds](https://github.com/colliery-io/brokkr/blob/main/docs/src/how-to/shipwright-builds.md)).
  Note that the pin is **chart-wide and unconditional** — setting `shipwright.enabled=false`
  does not lower it. If you need the agent on an older cluster you would have to relax
  `kubeVersion` in a fork; nothing else in the agent's own manifests is known to require 1.29.
- Helm 3.8+ (3.8 is the floor for `helm install oci://…` against the published charts;
  installing from a local checkout works with any Helm 3.x)
- A running Brokkr broker instance
- Broker Pre-Authenticated Key (PAK) for agent authentication
- (Optional) Prometheus Operator, if you enable `metrics.podMonitor`
- For a **default** install, cluster-admin rights and a cluster with no existing Tekton or
  Shipwright installation — see the next section

## What a Default Install Does to Your Cluster

> **Read this before installing into a shared or multi-tenant cluster.**
>
> With chart defaults (`shipwright.enabled: true`, `shipwright.install.tekton: true`,
> `shipwright.install.shipwright: true`), installing "just an agent" also installs
> **cluster-scoped build infrastructure that is not namespaced to your release**.

A pre-install/pre-upgrade hook Job (`templates/shipwright-install-job.yaml`) runs `kubectl apply`
against upstream release manifests and creates:

| What | Where | Triggered when |
|------|-------|----------------|
| **Tekton Pipelines `v0.68.1`** — 9 CRDs, the `tekton-pipelines` and `tekton-pipelines-resolvers` namespaces, controller/webhook/events-controller Deployments, cluster RBAC, and 3 admission webhook configurations | cluster-wide | `shipwright.enabled` **and** `shipwright.install.tekton` |
| **Shipwright Build `v0.18.1`** — 4 CRDs, the `shipwright-build` namespace, controller and webhook Deployments, cluster RBAC | cluster-wide | `shipwright.enabled` **and** `shipwright.install.shipwright` |
| Webhook serving certificate + CRD `caBundle` patches on `builds/buildruns/buildstrategies/clusterbuildstrategies.shipwright.io` | cluster-wide | as above, per `shipwright.certManagement` |
| Upstream sample `ClusterBuildStrategy` objects (buildah, kaniko, buildpacks, …) | cluster-wide | additionally `shipwright.install.sampleStrategies` |
| A `ServiceAccount` bound to the built-in **`cluster-admin`** ClusterRole, used to run the Job | your release namespace + a cluster-scoped `ClusterRoleBinding` | whenever the Job renders |

Versions come from `shipwright.install.tektonVersion` / `shipwright.install.shipwrightVersion`;
the table shows the current chart defaults. Consequences worth knowing up front:

- **It collides with an existing Tekton/Shipwright.** The Job applies the pinned upstream
  manifests with `kubectl apply --server-side=true` (no `--force-conflicts`), so against an
  installation owned by another field manager it will either take ownership of those
  cluster-scoped objects or fail on field conflicts — and the Job's `set -e` turns a failure
  into a failed `helm install`/`helm upgrade`. Either way the blast radius is the whole
  cluster, not just your namespace.
- **It requires cluster-admin, and grants it.** Whoever installs must be able to create a
  `ClusterRoleBinding` to `cluster-admin`. A namespace-scoped tenant cannot install with
  defaults, and should not be given the ability to.
- **It re-runs on every `helm upgrade`** — the hook is `pre-install,pre-upgrade`.
- **It needs egress** to `storage.googleapis.com` and `github.com` from the Job pod. Air-gapped
  clusters must opt out and install Tekton/Shipwright by mirror.
- **`helm uninstall` does not remove any of it.** The Job's `kubectl apply` output is not part
  of the Helm release. See [Uninstallation](#uninstallation).
- Setting `rbac.clusterWide: false` limits the *agent's* RBAC only. It does **not** disable
  the installer hook or make the install namespace-local.

### Opting Out

```bash
# Keep build work orders, but do NOT install Tekton/Shipwright (you provide them).
# shipwright.enabled=true still grants the agent RBAC for shipwright.io / tekton.dev.
helm install my-agent charts/brokkr-agent \
  --set shipwright.install.tekton=false \
  --set shipwright.install.shipwright=false \
  ...

# No build work orders at all: no hook Job, and no shipwright.io/tekton.dev RBAC.
helm install my-agent charts/brokkr-agent \
  --set shipwright.enabled=false \
  ...
```

Either form drops the hook Job, the installer ServiceAccount, and the `cluster-admin`
ClusterRoleBinding from the rendered manifests. Confirm before you install:

```bash
helm template my-agent charts/brokkr-agent --set shipwright.enabled=false | grep -c cluster-admin  # → 0
```

For the full build workflow see
[Shipwright Builds](https://github.com/colliery-io/brokkr/blob/main/docs/src/how-to/shipwright-builds.md).

## Installation

### Basic Installation

Deploy with default settings (cluster-wide RBAC — **and the cluster-wide Tekton/Shipwright
install described above**):

```bash
helm install my-agent charts/brokkr-agent \
  --set broker.url=http://my-broker:3000 \
  --set broker.pak=your-pak-token \
  --set broker.clusterName=production-cluster
```

For a multi-tenant cluster, the agent-only form is:

```bash
helm install my-agent charts/brokkr-agent \
  --namespace tenant-namespace \
  --set broker.url=http://my-broker:3000 \
  --set broker.pak=your-pak-token \
  --set broker.clusterName=production-cluster \
  --set shipwright.enabled=false \
  --set rbac.clusterWide=false
```

### Installation with Custom Agent Name

```bash
helm install my-agent charts/brokkr-agent \
  --set broker.url=http://my-broker:3000 \
  --set broker.pak=your-pak-token \
  --set broker.clusterName=production-cluster \
  --set broker.agentName=prod-k8s-agent
```

## Configuration

### Broker Connection

The agent requires connection details to communicate with the broker:

```yaml
broker:
  url: http://brokkr-broker:3000  # Broker service URL
  agentName: ""                    # Optional agent identifier (auto-generated if empty)
  clusterName: ""                  # Cluster identifier for broker
  pak: ""                          # Pre-Authenticated Key for agent authentication
  generatorIds: []                 # Generator scopes this agent serves (see below)
```

#### Generator scope (`broker.generatorIds`)

`generatorIds` is the agent's deploy-time **generator-registration scope** (ADR-0009,
BROKKR-I-0030). Each entry is a generator UUID; the agent self-registers with each on
startup, and the broker will only target it with stacks owned by those generators.

- **Empty (the default) = system/fleet scope only.** The broker auto-registers every
  agent with the built-in `__system__` generator at creation, so an empty list still
  reconciles fleet-management stacks — it just will **not** serve any application
  generator's stacks until you list their UUIDs. This is the system/fleet generator,
  **not** the admin generator.
- Accepts a YAML list (preferred) or a comma-separated string.

```bash
helm install my-agent charts/brokkr-agent \
  --set broker.url=http://my-broker:3000 \
  --set broker.clusterName=production \
  --set "broker.generatorIds={1b9d6bcd-bbfd-4b2d-9b5d-ab8dfbbd4bed,7c9e6679-7425-40de-944b-e07fc1f90ae7}"
```

Rendered into the agent ConfigMap as the `BROKKR__AGENT__GENERATOR_IDS` environment
variable (equivalently settable via `agent.generator_ids` in a config file or the
`--generator-ids` CLI flag).

**Security Note**: The PAK is a sensitive credential. Setting `broker.pak` renders it
into the agent ConfigMap in plaintext — fine for dev/test, but in production (and any
GitOps workflow) source it from a pre-existing Kubernetes Secret via `broker.existingSecret`:

```bash
# Secret keyed to match the env var name (the default existingSecretKey):
kubectl create secret generic agent-credentials \
  --from-literal=BROKKR__AGENT__PAK=your-pak-token

helm install my-agent charts/brokkr-agent \
  --set broker.url=http://my-broker:3000 \
  --set broker.clusterName=production \
  --set broker.existingSecret=agent-credentials
```

The PAK is injected into the pod via `secretKeyRef` (overriding the ConfigMap), so the
raw credential never lands in a values file or git history. If your Secret stores the PAK
under a different key, set `broker.existingSecretKey` to match. This pairs naturally with
[external-secrets-operator](https://external-secrets.io/), which can vend the PAK from
Vault / 1Password / AWS Secrets Manager into the Secret at deploy time.

### Agent Polling and Health Configuration

```yaml
agent:
  pollingInterval: 30       # Seconds between broker polls
  wsUrl: null               # Override the internal WebSocket URL; derived from broker.url when null
  deploymentHealth:
    enabled: true           # Report health of deployed workloads back to the broker
    intervalSeconds: 60     # Health check interval (30 is the practical minimum)
```

Leave `agent.wsUrl` null unless WebSocket traffic must traverse a different ingress or load
balancer than the REST API; otherwise the agent derives it from `broker.url`
(`http`→`ws`, `https`→`wss`, path `/internal/ws/agent`).

### RBAC Configuration

The agent requires Kubernetes API access to observe cluster state. Two modes are supported:

#### Cluster-Wide Access (Default)

Grants the agent access to all namespaces and cluster-scoped resources:

```yaml
rbac:
  create: true
  clusterWide: true
```

**Use when**:
- You want complete cluster visibility
- The agent should monitor all namespaces
- You have cluster-admin permissions to install

**Creates**: `ClusterRole` and `ClusterRoleBinding`

#### Namespace-Scoped Access

Restricts the agent to only the namespace where it's deployed:

```yaml
rbac:
  clusterWide: false
```

**Use when**:
- Operating in a multi-tenant cluster
- You want to limit the agent's scope
- You only have namespace-admin permissions

**Creates**: `Role` and `RoleBinding`

**Limitations**: Cannot access cluster-scoped resources (nodes, persistent volumes, cluster roles)

#### Custom Additional Permissions

Extend the agent's permissions for custom resources:

```yaml
rbac:
  additionalRules:
    - apiGroups: ["custom.io"]
      resources: ["customresources"]
      verbs: ["get", "list", "watch"]
    - apiGroups: [""]
      resources: ["resourcequotas"]
      verbs: ["get", "list"]
```

#### Secret Access (off by default)

The agent is **not** granted access to Secrets unless you ask for it:

```yaml
rbac:
  secretAccess:
    enabled: false       # true adds list/watch on secrets (metadata: names, namespaces, labels)
    readContents: false  # true additionally adds get — read access to secret *contents* in scope
```

With `rbac.clusterWide: true`, "in scope" means every Secret in the cluster. Turn `readContents`
on only when a workflow genuinely requires reading secret data, and prefer
`rbac.clusterWide: false` to bound the blast radius.

#### Disabling RBAC Creation

If you manage RBAC separately:

```yaml
rbac:
  create: false

serviceAccount:
  create: false
  name: my-existing-service-account
```

For detailed information about RBAC permissions and security implications, see [RBAC.md](./RBAC.md).

### Service Account Configuration

```yaml
serviceAccount:
  create: true
  name: ""  # Auto-generated if empty
```

To use an existing service account:

```yaml
serviceAccount:
  create: false
  name: my-service-account
```

### Resource Configuration

Configure resource requests and limits:

```yaml
resources:
  requests:
    memory: 128Mi
    cpu: 50m
  limits:
    memory: 256Mi
    cpu: 200m
```

### Security Context

The agent runs as a non-root user by default. Note the keys are `podSecurityContext` and
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

Three values were renamed or removed because none of them ever had an effect. Helm ignores stale
entries left in your values files, but you should delete them.

| Old key | New key | Notes |
|---------|---------|-------|
| `metrics.serviceMonitor.*` | `metrics.podMonitor.*` | Same sub-keys (`enabled`, `interval`, `scrapeTimeout`, `additionalLabels`). The old `ServiceMonitor` template selected a Service port named `health`, but this chart renders no Service, so it scraped nothing. The chart now renders a `PodMonitor` against a named container port. Nothing regresses — the old key never produced a working scrape. |
| `metrics.enabled` | `networkPolicy.allowMetricsScraping` | The old name implied it turned the `/metrics` endpoint on or off. It never did; the agent always serves `/metrics`. Its only effect was gating the NetworkPolicy ingress rule, which is what the new name says. Default is unchanged (`true`). |
| `telemetry.collector.*` | *(removed)* | The chart never rendered a collector sidecar. Setting `telemetry.collector.enabled: true` only repointed `BROKKR__TELEMETRY__OTLP_ENDPOINT` at `http://localhost:4317`, where nothing was listening — it silently broke otherwise-working telemetry. `BROKKR__TELEMETRY__OTLP_ENDPOINT` now always renders from `telemetry.otlpEndpoint`. |

## Values

This table covers **every key in `values.yaml`** for chart version 0.8.4, plus the keys the
templates read that `values.yaml` only mentions in comments (marked *not in values.yaml*).
It is hand-maintained, so `values.yaml` remains the authoritative source — check it with
`helm show values charts/brokkr-agent` (or `oci://ghcr.io/colliery-io/charts/brokkr-agent`)
if this table and the chart ever disagree.

### Image and workload

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `image.repository` | string | `"ghcr.io/colliery-io/brokkr-agent"` | Container image repository |
| `image.tag` | string | `"latest"` | Container image tag. Pin a release tag in production. |
| `image.pullPolicy` | string | `"IfNotPresent"` | Image pull policy |
| `image.pullSecrets` | array | unset (*not in values.yaml*) | `imagePullSecrets` entries for private registries, e.g. `[{name: ghcr-secret}]` |
| `replicaCount` | int | `1` | Number of agent replicas. The agent is not designed to run active/active — leave at 1. |
| `nameOverride` | string | unset (*not in values.yaml*) | Overrides the chart name portion of generated resource names |
| `fullnameOverride` | string | unset (*not in values.yaml*) | Overrides the full generated resource name |
| `hostAliases` | array | `[]` | `hostAliases` entries injected into the pod, for resolving a broker that lives outside the cluster |

### Broker connection

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `broker.url` | string | `"http://brokkr-broker:3000"` | Broker service URL (`BROKKR__AGENT__BROKER_URL`) |
| `broker.agentName` | string | `""` | Agent identifier. Must match the name the agent was registered under on the broker, or startup self-lookup fails. |
| `broker.clusterName` | string | `""` | Cluster identifier. Must match the registered cluster name. |
| `broker.pak` | string | `""` | Pre-Authenticated Key (rendered into the ConfigMap in plaintext; dev/test only). Ignored when `broker.existingSecret` is set. |
| `broker.existingSecret` | string | `""` | Name of a pre-existing Secret to source the PAK from. When set, the PAK is injected via `secretKeyRef` and kept out of the ConfigMap/values/git (GitOps-friendly). |
| `broker.existingSecretKey` | string | `"BROKKR__AGENT__PAK"` | Key within `broker.existingSecret` holding the PAK. |
| `broker.generatorIds` | list/string | `[]` | Generator UUIDs the agent self-registers with on startup (`BROKKR__AGENT__GENERATOR_IDS`). Empty = system/fleet scope only. |
| `broker.port` | int | `3000` (*not in values.yaml*) | Broker port used **only** to build the NetworkPolicy egress rule. Set it if your broker does not listen on 3000 and `networkPolicy.enabled: true`. |

### Agent behaviour

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `agent.pollingInterval` | int | `30` | Seconds between broker polls (`BROKKR__AGENT__POLLING_INTERVAL`) |
| `agent.wsUrl` | string | `null` | Override for the internal WebSocket upgrade URL (`BROKKR__AGENT__WS_URL`). Left null, it is derived from `broker.url` (`http`→`ws`, `https`→`wss`, path `/internal/ws/agent`). Set only when WS traffic must traverse a different ingress than REST. |
| `agent.deploymentHealth.enabled` | bool | `true` | Enable deployment health checking (`BROKKR__AGENT__DEPLOYMENT_HEALTH_ENABLED`) |
| `agent.deploymentHealth.intervalSeconds` | int | `60` | Health check interval in seconds; 30 is the practical minimum |

### Shipwright / Tekton build infrastructure

See [What a Default Install Does to Your Cluster](#what-a-default-install-does-to-your-cluster)
— these defaults mutate cluster-global state.

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `shipwright.enabled` | bool | `true` | Master switch for the build integration. Gates the installer hook Job **and** the agent's `shipwright.io` / `tekton.dev` ClusterRole rules. Set `false` if you never use build work orders. |
| `shipwright.install.tekton` | bool | `true` | Install Tekton Pipelines cluster-wide via the pre-install/pre-upgrade hook Job |
| `shipwright.install.tektonVersion` | string | `"v0.68.1"` | Tekton Pipelines release applied by the hook Job |
| `shipwright.install.shipwright` | bool | `true` | Install Shipwright Build cluster-wide via the hook Job |
| `shipwright.install.shipwrightVersion` | string | `"v0.18.1"` | Shipwright Build release applied by the hook Job |
| `shipwright.install.sampleStrategies` | bool | `true` | Also apply the upstream sample `ClusterBuildStrategy` set (buildah, kaniko, buildpacks, …) cluster-wide |
| `shipwright.install.image` | string | `"alpine/k8s:1.30.2"` | Image for the installer Job (needs `kubectl`; `openssl` is added at runtime via `apk`) |
| `shipwright.install.timeout` | int | `600` | Installer Job `activeDeadlineSeconds` |
| `shipwright.certManagement` | string | `"self-signed"` | How the Shipwright webhook certificate is provisioned: `self-signed` (Job generates a CA and patches CRD `caBundle`s) or `cert-manager` |
| `shipwright.certManager.issuerName` | string | `"selfsigned-issuer"` | Issuer used for the webhook Certificate (`certManagement: cert-manager` only) |
| `shipwright.certManager.issuerKind` | string | `"Issuer"` | `Issuer` (the chart creates a self-signed one in `shipwright-build`) or `ClusterIssuer` (you provide it) |
| `shipwright.certManager.duration` | string | `"2160h"` | Certificate lifetime (90 days) |
| `shipwright.certManager.renewBefore` | string | `"720h"` | Renew window before expiry (30 days) |
| `shipwright.installSampleStrategies` | bool | `false` | Render this chart's own bundled buildah `ClusterBuildStrategy`. Only takes effect when `shipwright.install.sampleStrategies` is `false`. |

### RBAC and service account

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `rbac.create` | bool | `true` | Create the agent Role/ClusterRole and binding |
| `rbac.clusterWide` | bool | `true` | `ClusterRole`/`ClusterRoleBinding` (true) or namespaced `Role`/`RoleBinding` (false). When false the deployment also sets `BROKKR__AGENT__WATCH_NAMESPACE` to the release namespace. Does **not** affect the Shipwright installer hook. |
| `rbac.secretAccess.enabled` | bool | `false` | Grant the agent `list`/`watch` on Secrets. Off by default; enabling exposes Secret metadata in scope. |
| `rbac.secretAccess.readContents` | bool | `false` | Additionally grant `get` on Secrets — this is read access to **Secret contents** in scope. Enable only if a workflow demands it. |
| `rbac.additionalRules` | array | `[]` | Extra policy rules appended verbatim to the agent role |
| `serviceAccount.create` | bool | `true` | Create the agent ServiceAccount |
| `serviceAccount.name` | string | `""` | ServiceAccount name; auto-generated from the release when empty |

### Resources and security context

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `resources.requests.memory` | string | `"128Mi"` | Memory request |
| `resources.requests.cpu` | string | `"50m"` | CPU request |
| `resources.limits.memory` | string | `"256Mi"` | Memory limit |
| `resources.limits.cpu` | string | `"200m"` | CPU limit |
| `apparmor.enabled` | bool | `false` | Add the `container.apparmor.security.beta.kubernetes.io/agent` pod annotation. Requires AppArmor-enabled nodes. |
| `apparmor.profile` | string | `"runtime/default"` | AppArmor profile referenced by that annotation |
| `podSecurityContext` | object | `{runAsNonRoot: true, runAsUser: 10001, runAsGroup: 10001, fsGroup: 10001}` | Rendered verbatim as the pod `securityContext`. Replacing this object replaces all of it — re-state the defaults you want to keep. `seccompProfile: {type: RuntimeDefault}` is commented out in `values.yaml` and recommended for hardening. |
| `containerSecurityContext` | object | `{allowPrivilegeEscalation: false, readOnlyRootFilesystem: false, runAsNonRoot: true, runAsUser: 10001, capabilities: {drop: [ALL]}}` | Rendered verbatim as the container `securityContext`. The chart already mounts an `emptyDir` at `/tmp`, so `readOnlyRootFilesystem: true` is the hardened setting. |

### Metrics

The agent serves `/metrics` — alongside `/healthz` and `/readyz` — on the container port named
`health` (8080). The endpoint is always served; no chart value disables it. Because the agent
chart renders no `Service` (nothing in the cluster calls the agent), Prometheus Operator scrapes
the **pod** directly via a `PodMonitor`, not a `ServiceMonitor`.

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `metrics.podMonitor.enabled` | bool | `false` | Create a Prometheus Operator `PodMonitor` scraping `/metrics` on the pod's `health` container port |
| `metrics.podMonitor.interval` | string | `"30s"` | Scrape interval |
| `metrics.podMonitor.scrapeTimeout` | string | unset (commented in `values.yaml`) | Scrape timeout; must be shorter than `interval` |
| `metrics.podMonitor.additionalLabels` | object | unset (commented in `values.yaml`) | Extra labels so your Prometheus `podMonitorSelector` matches |

> **Note:** Prometheus Operator matches `PodMonitor` objects with its `podMonitorSelector`, which
> is a *separate* selector from `serviceMonitorSelector`. If you previously set
> `metrics.serviceMonitor.additionalLabels` to satisfy `serviceMonitorSelector`, check that your
> Prometheus resource also selects `PodMonitor`s.

### NetworkPolicy

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `networkPolicy.enabled` | bool | `false` | Create a NetworkPolicy for the agent pod (Ingress + Egress) |
| `networkPolicy.kubernetesApiCidr` | string | `"0.0.0.0/0"` | CIDR allowed for egress to the API server on 443/6443. The default allows any destination — narrow it to your API server for real restriction. |
| `networkPolicy.brokerEndpoint` | object | unset (commented in `values.yaml`) | `podSelector`/`namespaceSelector` identifying the broker. When unset, broker egress falls back to `0.0.0.0/0` on `broker.port`. |
| `networkPolicy.allowMetricsScraping` | bool | `true` | Whether the metrics ingress rule is rendered at all. Set `false` to deny scraping of port 8080 outright even when `allowMetricsFrom` is populated. Does **not** turn off the `/metrics` endpoint — the agent always serves it. |
| `networkPolicy.allowMetricsFrom` | array | `[]` | Selectors allowed to scrape port 8080. Only applied when `allowMetricsScraping` is true; empty means no metrics ingress is permitted. |
| `networkPolicy.additionalEgressRules` | array | `[]` | Extra egress rules appended verbatim |

### Telemetry (OpenTelemetry tracing)

| Key | Type | Default | Description |
|-----|------|---------|-------------|
| `telemetry.enabled` | bool | `false` | Set `BROKKR__TELEMETRY__ENABLED`; also sets the service name and sampling rate |
| `telemetry.otlpEndpoint` | string | `"http://otel-collector:4317"` | OTLP/gRPC endpoint for trace export |
| `telemetry.samplingRate` | float | `0.1` | Fraction of traces sampled (0.0–1.0) |

The chart does not run an OpenTelemetry Collector sidecar; point `telemetry.otlpEndpoint` at a
collector you run yourself (or at your service mesh's OTLP receiver). See
[Migration notes](#migration-notes) if you have `telemetry.collector.*` in your values.

## Examples

### Development Setup

```bash
helm install dev-agent charts/brokkr-agent \
  --set broker.url=http://dev-broker:3000 \
  --set broker.pak=dev-pak-token \
  --set broker.clusterName=dev-cluster
```

### Production Setup with External Broker

```bash
helm install prod-agent charts/brokkr-agent \
  --set broker.url=https://broker.example.com \
  --set broker.pak=prod-pak-token \
  --set broker.clusterName=prod-us-east-1 \
  --set broker.agentName=prod-primary-agent \
  --set agent.pollingInterval=60 \
  --set resources.requests.memory=256Mi \
  --set resources.requests.cpu=100m
```

### Multi-Tenant Namespace-Scoped Deployment

```bash
helm install tenant-agent charts/brokkr-agent \
  --namespace tenant-namespace \
  --set broker.url=http://shared-broker:3000 \
  --set broker.pak=tenant-pak-token \
  --set broker.clusterName=shared-cluster \
  --set broker.agentName=tenant-a-agent \
  --set rbac.clusterWide=false \
  --set shipwright.enabled=false
```

`shipwright.enabled=false` is what keeps this install namespace-local. Without it the chart still
runs the `cluster-admin` installer Job and installs Tekton/Shipwright cluster-wide, regardless of
`rbac.clusterWide`.

### Deployment with Custom Resources

```bash
helm install agent-with-crds charts/brokkr-agent \
  --set broker.url=http://broker:3000 \
  --set broker.pak=pak-token \
  --set broker.clusterName=cluster \
  --set-json 'rbac.additionalRules=[{"apiGroups":["custom.io"],"resources":["customresources"],"verbs":["get","list","watch"]}]'
```

## RBAC Permissions

The agent requires broad Kubernetes permissions for both cluster observation and resource deployment. Default permissions include:

**Observation (get, list, watch)**: pods, namespaces, nodes, services, endpoints, configmaps, persistentvolumes, persistentvolumeclaims, events, deployments, statefulsets, daemonsets, replicasets, jobs, cronjobs, ingresses, networkpolicies, roles, rolebindings, clusterroles, clusterrolebindings

**Deployment (all verbs)**: The agent also receives wildcard permissions (`"*"` resources, `"*"` verbs) on core, apps, batch, networking, and RBAC API groups to enable server-side apply of arbitrary Kubernetes manifests. When Shipwright is enabled, wildcard access to `shipwright.io` and `tekton.dev` API groups is also granted.

For detailed information about why each permission is needed and security implications, see [RBAC.md](./RBAC.md).

## Troubleshooting

### Agent Cannot Connect to Broker

**Symptom**: Agent logs show connection errors to broker

**Solutions**:
1. Verify broker URL is correct: `kubectl get configmap <release-name>-brokkr-agent -o yaml`
2. Check broker is accessible: `kubectl run -it --rm debug --image=curlimages/curl --restart=Never -- curl http://broker:3000/healthz`
3. Verify network policies allow traffic from agent to broker

### Agent Cannot Access Kubernetes Resources

**Symptom**: Agent logs show "Forbidden" or "unauthorized" errors

**Solutions**:
1. Verify RBAC resources were created: `kubectl get clusterrole,clusterrolebinding -l app.kubernetes.io/name=brokkr-agent`
2. Test permissions: `kubectl auth can-i list pods --as=system:serviceaccount:<namespace>:<service-account>`
3. See [RBAC.md](./RBAC.md) for detailed troubleshooting

### Invalid PAK Token

**Symptom**: Agent logs show authentication errors

**Solutions**:
1. Verify PAK is correct in ConfigMap or Secret
2. Generate a new PAK from the broker
3. PAKs do not expire — if the agent's key stopped working, it was rotated or revoked (the agent record was deleted on the broker)

### Agent Not Polling

**Symptom**: Agent starts but doesn't poll broker

**Solutions**:
1. Check agent logs: `kubectl logs -l app.kubernetes.io/name=brokkr-agent --tail=100 -f`
2. Verify polling interval is configured: `kubectl get configmap <release-name>-brokkr-agent -o yaml`
3. Check if agent is stuck in a crash loop: `kubectl get pods -l app.kubernetes.io/name=brokkr-agent`

### Viewing Logs

```bash
kubectl logs -l app.kubernetes.io/name=brokkr-agent --tail=100 -f
```

## Security Considerations

1. **PAK Protection**: Store the PAK in a Kubernetes Secret (`broker.existingSecret`), not in values files
2. **RBAC Scope**: Use namespace-scoped mode (`rbac.clusterWide: false`) in multi-tenant environments
3. **Cluster-wide install side effects**: With defaults, the chart installs Tekton and Shipwright
   cluster-wide through a Job bound to `cluster-admin`. In a multi-tenant cluster set
   `shipwright.enabled=false` (or at least `shipwright.install.tekton=false` and
   `shipwright.install.shipwright=false`) — see
   [What a Default Install Does to Your Cluster](#what-a-default-install-does-to-your-cluster)
4. **Secret Access**: Secret access is **off** by default (`rbac.secretAccess.enabled: false`).
   Enabling it grants `list`/`watch`; `rbac.secretAccess.readContents: true` additionally grants
   `get`, i.e. read access to Secret contents in scope — see [RBAC.md](./RBAC.md) for mitigations
5. **Deployment permissions**: Even without secret access, the agent holds wildcard verbs on core,
   apps, batch, and networking (plus RBAC when cluster-wide) so it can apply arbitrary manifests
6. **Resource Limits**: Configure appropriate resource limits to prevent resource exhaustion
7. **Network Policies**: Restrict agent network access to only the broker (`networkPolicy.enabled`,
   and narrow `networkPolicy.kubernetesApiCidr` from its `0.0.0.0/0` default)

## Validating chart changes

`helm lint` checks syntax and schema. It cannot tell you that a value is accepted, documented,
and renders nothing — four values shipped in exactly that state and were only caught by reading
the templates by hand (BROKKR-T-0308). The render assertions guard against a repeat:

```bash
angreal helm check-values
```

`helm template` only — no cluster, no docker, a couple of seconds. It fails if:

- a shipped values file (`values.yaml`, `values-dev.yaml`, `values/*.yaml`) stops rendering, or
  renders nothing;
- one of the security- or capability-relevant values stops changing the rendered output:
  `metrics.podMonitor.enabled` rendering a PodMonitor whose `port` names a container port the
  Deployment actually declares, `networkPolicy.allowMetricsScraping`, `telemetry.otlpEndpoint`,
  and the `broker.existingSecret` pairing (the plaintext PAK kept out of the ConfigMap **and** a
  `secretKeyRef` added);
- any leaf key in any shipped values file is not referenced under `templates/`. Keys that are
  legitimately unreferenced are listed with a reason in `VALUES_KEY_ALLOWLIST` in
  `.angreal/task_helm.py`. Add a reason when you add an entry.

CI runs this on every PR touching `charts/**`, as the "Helm Template Validation" job. If PyYAML
is not importable from your angreal interpreter, run
`uvx --from angreal --with pyyaml angreal helm check-values`.

## Uninstallation

```bash
helm uninstall my-agent
```

This removes the resources in the Helm release: the Deployment, ConfigMap, ServiceAccount, the
agent's Role/ClusterRole and binding, and (if enabled) the NetworkPolicy and PodMonitor.

**It does not remove Tekton Pipelines or Shipwright Build.** Those were applied by the
pre-install hook Job with `kubectl apply`, so they are not tracked by the Helm release and
survive uninstall — including their CRDs, `tekton-pipelines` / `tekton-pipelines-resolvers` /
`shipwright-build` namespaces, cluster RBAC, admission webhooks, and any sample
`ClusterBuildStrategy` objects. To check what is left behind:

```bash
kubectl get ns tekton-pipelines shipwright-build
kubectl get crd | grep -E 'tekton\.dev|shipwright\.io'
kubectl get clusterbuildstrategies
```

Removing them means deleting the upstream manifests yourself, at the versions that were
installed — and only after confirming nothing else in the cluster depends on them:

```bash
kubectl delete -f https://github.com/shipwright-io/build/releases/download/v0.18.1/release.yaml
kubectl delete -f https://storage.googleapis.com/tekton-releases/pipeline/previous/v0.68.1/release.yaml
```

The installer ServiceAccount and its `cluster-admin` ClusterRoleBinding are cleaned up by the
hook's `hook-succeeded` delete policy, so they should not linger after a successful install. If
the Job failed, check for and remove them explicitly:

```bash
kubectl get clusterrolebinding | grep -- -installer
```

## Architecture

```
┌─────────────────┐
│  Brokkr Broker  │
│                 │
└────────▲────────┘
         │
         │ HTTP/HTTPS
         │ PAK Auth
         │
┌────────┴────────┐
│  Brokkr Agent   │
│                 │
│  Control Loop:  │
│  1. Poll Broker │
│  2. Read K8s    │
│  3. Report Back │
└────────┬────────┘
         │
         │ RBAC
         │
┌────────▼────────┐
│ Kubernetes API  │
│                 │
│  Resources:     │
│  - Pods         │
│  - Deployments  │
│  - Services     │
│  - etc.         │
└─────────────────┘
```

## Development Phases

**Phase 1** (Complete): Basic agent deployment and broker connection
**Phase 2** (Complete): Comprehensive RBAC for cluster observation and deployment management

See [RBAC.md](./RBAC.md) for detailed information about RBAC permissions.
