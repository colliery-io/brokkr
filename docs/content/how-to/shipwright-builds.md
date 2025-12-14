---
title: "Container Builds with Shipwright"
weight: 1
---

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
kubectl version --short
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
- **Tekton Pipelines** (v0.37.2) - Task execution engine
- **Shipwright Build** (v0.10.0) - Build orchestration
- **buildah ClusterBuildStrategy** - Default build strategy

### Option 2: Bring Your Own Shipwright

If you already have Shipwright and Tekton installed, or need specific versions:

```bash
# Disable bundled Shipwright
helm install brokkr-agent oci://ghcr.io/colliery-io/charts/brokkr-agent \
  --set broker.url=http://brokkr-broker:3000 \
  --set broker.pak="<YOUR_PAK>" \
  --set shipwright.enabled=false \
  --wait
```

#### Manual Shipwright Installation

If installing Shipwright manually, ensure you have compatible versions:

```bash
# Install Tekton Pipelines (v0.59.0 or later recommended)
kubectl apply -f https://storage.googleapis.com/tekton-releases/pipeline/previous/v0.59.0/release.yaml

# Wait for Tekton to be ready
kubectl wait --for=condition=Ready pods -l app=tekton-pipelines-controller -n tekton-pipelines --timeout=300s

# Install Shipwright Build (v0.13.0 or later recommended)
kubectl apply -f https://github.com/shipwright-io/build/releases/download/v0.13.0/release.yaml

# Wait for Shipwright to be ready
kubectl wait --for=condition=Ready pods -l app=shipwright-build-controller -n shipwright-build --timeout=300s

# Install sample build strategies
kubectl apply -f https://github.com/shipwright-io/build/releases/download/v0.13.0/sample-strategies.yaml
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

The bundled installation includes a `buildah` ClusterBuildStrategy. You can install additional strategies:

```bash
# Install all sample strategies (buildah, kaniko, buildpacks, etc.)
helm upgrade brokkr-agent oci://ghcr.io/colliery-io/charts/brokkr-agent \
  --reuse-values \
  --set shipwright.installSampleStrategies=true
```

### Disable Sample Strategies

If you only want Shipwright without sample strategies:

```bash
helm install brokkr-agent oci://ghcr.io/colliery-io/charts/brokkr-agent \
  --set broker.url=http://brokkr-broker:3000 \
  --set broker.pak="<YOUR_PAK>" \
  --set shipwright.enabled=true \
  --set shipwright.installSampleStrategies=false
```

## RBAC Configuration

The brokkr-agent automatically includes RBAC rules for Shipwright when enabled:

```yaml
# Automatically included when shipwright.enabled=true
- apiGroups: ["shipwright.io"]
  resources: ["builds", "buildruns"]
  verbs: ["get", "list", "watch", "create", "update", "patch", "delete"]
- apiGroups: ["shipwright.io"]
  resources: ["buildstrategies", "clusterbuildstrategies"]
  verbs: ["get", "list", "watch"]
```

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

- Learn about [Work Orders](../reference/work-orders) for managing builds through Brokkr
- See [Build Examples](../tutorials/build-examples) for common build patterns
- Configure [Monitoring](../reference/monitoring) for build metrics
