# Container Builds with Shipwright

Brokkr integrates with [Shipwright Build](https://shipwright.io/) to provide native container image building capabilities. This guide covers installation, configuration, and usage of Shipwright builds through Brokkr's work order system.

## Overview

Shipwright Build is a CNCF Sandbox project that provides a framework for building container images on Kubernetes. Brokkr uses Shipwright as the execution engine for build work orders, allowing you to:

- Build container images from Git repositories
- Push images to container registries
- Leverage production-ready build strategies (buildah, kaniko)
- Manage builds through Brokkr's work order API

## Prerequisites

### Kubernetes Version

Shipwright integration requires **Kubernetes 1.29 or later** due to dependencies on newer API features.

```bash
# Verify your Kubernetes version
kubectl version
```

### Cluster Requirements

- Sufficient resources for build pods (recommended: 4GB memory, 2 CPU cores available)
- Network access to container registries you'll push to
- Network access to Git repositories you'll build from

## Installation Options

### Option 1: Bundled Installation (Recommended)

The brokkr-agent Helm chart includes Shipwright Build and Tekton Pipelines as vendored dependencies. This is enabled by default.

```bash
# Install agent with bundled Shipwright (default)
helm install brokkr-agent oci://ghcr.io/colliery-io/charts/brokkr-agent \
  --set broker.url=http://brokkr-broker:3000 \
  --set broker.pak="<YOUR_PAK>" \
  --wait
```

This installs:
- **Tekton Pipelines** (v0.68.1) - Task execution engine
- **Shipwright Build** (v0.18.1) - Build orchestration
- **Sample ClusterBuildStrategies** (buildah, kaniko, etc.)

The bundled versions are pinned in the chart's values (`shipwright.install.tektonVersion` and `shipwright.install.shipwrightVersion` in `charts/brokkr-agent/values.yaml`) and are the versions the integration is tested against. Override them there if you need different releases.

### Option 2: Bring Your Own Shipwright

If you already have Shipwright and Tekton installed, or need specific versions, disable the bundled installation but **keep `shipwright.enabled=true`** — that flag also gates the agent's RBAC for `shipwright.io` and `tekton.dev` resources, which build work orders require:

```bash
# Skip bundled installation, keep Shipwright integration (and RBAC) enabled
helm install brokkr-agent oci://ghcr.io/colliery-io/charts/brokkr-agent \
  --set broker.url=http://brokkr-broker:3000 \
  --set broker.pak="<YOUR_PAK>" \
  --set shipwright.install.tekton=false \
  --set shipwright.install.shipwright=false \
  --wait
```

Set `shipwright.enabled=false` only if you are not using build work orders at all.

#### Manual Shipwright Installation

If installing Shipwright manually, prefer the versions the chart bundles (Tekton v0.68.1, Shipwright v0.18.1) — older releases may work but are not what Brokkr is tested against:

```bash
# Install Tekton Pipelines
kubectl apply -f https://storage.googleapis.com/tekton-releases/pipeline/previous/v0.68.1/release.yaml

# Wait for Tekton to be ready
kubectl wait --for=condition=Ready pods -l app=tekton-pipelines-controller -n tekton-pipelines --timeout=300s

# Install Shipwright Build
kubectl apply -f https://github.com/shipwright-io/build/releases/download/v0.18.1/release.yaml

# Wait for Shipwright to be ready
kubectl wait --for=condition=Ready pods -l app=shipwright-build-controller -n shipwright-build --timeout=300s

# Install sample build strategies
kubectl apply -f https://github.com/shipwright-io/build/releases/download/v0.18.1/sample-strategies.yaml
```

## Verifying Installation

### Check Components

```bash
# Verify Tekton Pipelines
kubectl get pods -n tekton-pipelines
# Expected: tekton-pipelines-controller and tekton-pipelines-webhook Running

# Verify Shipwright Build
kubectl get pods -n shipwright-build
# Expected: shipwright-build-controller Running

# Verify ClusterBuildStrategies
kubectl get clusterbuildstrategies
# Expected: buildah (and others if sample strategies installed)
```

### Test Build Capability

Create a simple test build to verify the installation:

```yaml
# test-build.yaml
apiVersion: shipwright.io/v1beta1
kind: Build
metadata:
  name: test-build
spec:
  source:
    type: Git
    git:
      url: https://github.com/shipwright-io/sample-go
  strategy:
    name: buildah
    kind: ClusterBuildStrategy
  output:
    image: ttl.sh/brokkr-test-$(date +%s):1h
---
apiVersion: shipwright.io/v1beta1
kind: BuildRun
metadata:
  generateName: test-build-run-
spec:
  build:
    name: test-build
```

```bash
# Apply the test build
kubectl apply -f test-build.yaml

# Watch the build progress
kubectl get buildruns -w

# Check build logs
kubectl logs -l buildrun.shipwright.io/name=test-build-run-xxxxx -c step-build
```

## Configuration

### Build Strategies

The bundled installation includes the official sample ClusterBuildStrategies (buildah, kaniko, buildpacks, etc.) by default via `shipwright.install.sampleStrategies`:

```bash
# Sample strategies are installed by default; re-enable explicitly if needed
helm upgrade brokkr-agent oci://ghcr.io/colliery-io/charts/brokkr-agent \
  --reuse-values \
  --set shipwright.install.sampleStrategies=true
```

### Disable Sample Strategies

If you only want Shipwright without sample strategies:

```bash
helm install brokkr-agent oci://ghcr.io/colliery-io/charts/brokkr-agent \
  --set broker.url=http://brokkr-broker:3000 \
  --set broker.pak="<YOUR_PAK>" \
  --set shipwright.enabled=true \
  --set shipwright.install.sampleStrategies=false
```

## RBAC Configuration

Build work orders require the agent to manage `shipwright.io` and `tekton.dev` resources. The brokkr-agent chart includes these RBAC rules in the agent ClusterRole whenever `shipwright.enabled=true` (even when the bundled installation is skipped):

```yaml
# Automatically included when shipwright.enabled=true
- apiGroups: ["shipwright.io"]
  resources: ["*"]
  verbs: ["*"]
- apiGroups: ["tekton.dev"]
  resources: ["*"]
  verbs: ["*"]
```

If you disable `shipwright.enabled`, the agent loses these permissions and build work orders will fail.

## Registry Authentication

To push images to private registries, create a Kubernetes secret:

```bash
# Create registry credentials secret
kubectl create secret docker-registry registry-creds \
  --docker-server=ghcr.io \
  --docker-username=<username> \
  --docker-password=<token> \
  --docker-email=<email>
```

Reference this secret in your Build spec:

```yaml
apiVersion: shipwright.io/v1beta1
kind: Build
metadata:
  name: my-build
spec:
  source:
    type: Git
    git:
      url: https://github.com/org/repo
  strategy:
    name: buildah
    kind: ClusterBuildStrategy
  output:
    image: ghcr.io/org/my-image:latest
    pushSecret: registry-creds  # Reference your secret here
```

## Git Authentication

For private Git repositories:

```bash
# Create Git credentials secret (HTTPS)
kubectl create secret generic git-creds \
  --from-literal=username=<username> \
  --from-literal=password=<token>

# Or for SSH
kubectl create secret generic git-ssh-creds \
  --from-file=ssh-privatekey=/path/to/id_rsa
```

Reference in your Build:

```yaml
spec:
  source:
    type: Git
    git:
      url: https://github.com/org/private-repo
      cloneSecret: git-creds
```

## Triggering a Build Through Brokkr

Builds run through Brokkr's work order system. Create a work order with `work_type: "build"` and the Shipwright Build YAML in `yaml_content`, targeting agents by ID, label, or annotation:

```bash
curl -X POST "http://broker:3000/api/v1/work-orders" \
  -H "Authorization: Bearer $ADMIN_PAK" \
  -H "Content-Type: application/json" \
  -d '{
    "work_type": "build",
    "yaml_content": "apiVersion: shipwright.io/v1beta1\nkind: Build\nmetadata:\n  name: my-build\nspec:\n  source:\n    type: Git\n    git:\n      url: https://github.com/org/repo\n  strategy:\n    name: buildah\n    kind: ClusterBuildStrategy\n  output:\n    image: ghcr.io/org/my-image:latest\n    pushSecret: registry-creds",
    "targeting": {
      "labels": ["builder"]
    }
  }'
```

A matching agent claims the work order, applies the Build, creates a BuildRun, and watches it to completion with a 15-minute timeout. On success, the work order completes with the image digest as its result message; on failure, the error details are recorded instead. Completed work orders move to the work-order log:

```bash
# View completed builds (the message field carries the image digest)
curl "http://broker:3000/api/v1/work-order-log?work_type=build" \
  -H "Authorization: Bearer $ADMIN_PAK"
```

See the [Work Orders Reference](../reference/work-orders.md) for the full request schema, targeting semantics, and retry behavior.

## Troubleshooting

### Build Pods Not Starting

```bash
# Check for pending pods
kubectl get pods -l build.shipwright.io/name

# Check events
kubectl get events --sort-by='.lastTimestamp'

# Verify ServiceAccount has required permissions
kubectl auth can-i create pods --as=system:serviceaccount:default:default
```

### Shipwright Controller Not Ready

```bash
# Check controller logs
kubectl logs -n shipwright-build -l app=shipwright-build-controller

# Check for CRD installation
kubectl get crd builds.shipwright.io buildruns.shipwright.io
```

### Tekton Pipeline Failures

```bash
# Check Tekton controller logs
kubectl logs -n tekton-pipelines -l app=tekton-pipelines-controller

# Check TaskRun status
kubectl get taskruns
kubectl describe taskrun <taskrun-name>
```

## Next Steps

- Learn about [Work Orders](../reference/work-orders.md) for managing builds through Brokkr
- Configure [Monitoring](../reference/monitoring.md) for build metrics
