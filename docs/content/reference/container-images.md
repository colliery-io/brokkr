---
title: Container Images
weight: 20
---

# Container Images Reference

This reference documents Brokkr's container images, repository locations, tag formats, and publishing commands.

## Image Repositories

All Brokkr images are published to GitHub Container Registry (GHCR) under the `colliery-io` organization.

### Available Images

| Component | Repository | Purpose |
|-----------|------------|---------|
| Broker | `ghcr.io/colliery-io/brokkr-broker` | Central management service |
| Agent | `ghcr.io/colliery-io/brokkr-agent` | Kubernetes cluster agent |
| UI | `ghcr.io/colliery-io/brokkr-ui` | Administrative web interface |

### Supported Architectures

All images support the following platforms:
- `linux/amd64` - x86_64 architecture
- `linux/arm64` - ARM64/aarch64 architecture

## Tag Format Specifications

### Semantic Version Tags

Created when a git tag matching `v*.*.*` is pushed.

| Tag Format | Example | Description | Mutable |
|------------|---------|-------------|---------|
| `v{major}.{minor}.{patch}` | `v1.2.3` | Full semantic version | No |
| `v{major}.{minor}` | `v1.2` | Latest patch in minor version | Yes |
| `v{major}` | `v1` | Latest minor in major version | Yes |
| `latest` | `latest` | Most recent stable release | Yes |

**Example**: Tagging release `v1.2.3` creates:
```
ghcr.io/colliery-io/brokkr-broker:v1.2.3
ghcr.io/colliery-io/brokkr-broker:v1.2
ghcr.io/colliery-io/brokkr-broker:v1
ghcr.io/colliery-io/brokkr-broker:latest
```

### Commit SHA Tags

Created for every commit that triggers a container build.

| Tag Format | Example | Description | Mutable |
|------------|---------|-------------|---------|
| `sha-{short-sha}` | `sha-abc1234` | 7-character commit SHA | No |

**Example**: Commit `abc1234def5678` creates:
```
ghcr.io/colliery-io/brokkr-broker:sha-abc1234
```

### Branch Tags

Created for pushes to tracked branches.

| Tag Format | Example | Description | Mutable |
|------------|---------|-------------|---------|
| `{branch-name}` | `main` | Branch name (sanitized) | Yes |
| `develop` | `develop` | Development branch | Yes |

**Example**: Push to `develop` branch creates:
```
ghcr.io/colliery-io/brokkr-broker:develop
```

### Pull Request Tags

Optionally created for pull request builds.

| Tag Format | Example | Description | Mutable |
|------------|---------|-------------|---------|
| `pr-{number}` | `pr-123` | Pull request number | Yes |

**Example**: PR #123 creates:
```
ghcr.io/colliery-io/brokkr-broker:pr-123
```

## Image Digests

Every image has a unique SHA256 digest that never changes:

```
ghcr.io/colliery-io/brokkr-broker@sha256:9fc91fae0f07c60ccbec61d86ff93fe825f92c42e5136295552ae196200dbe86
```

**Production recommendation**: Always use digest references for deployments to ensure immutability.

## Building Images

### Local Build (Single Architecture)

Build for your current platform:

```bash
angreal build multi-arch <component> --tag <tag>
```

**Parameters**:
- `<component>`: `broker`, `agent`, `ui`, or `all`
- `--tag <tag>`: Image tag (default: `dev`)
- `--registry <url>`: Registry URL (default: `ghcr.io/colliery-io`)
- `--platforms <platforms>`: Platform list (default: current platform for local builds)

**Examples**:
```bash
# Build broker for current platform
angreal build multi-arch broker --tag dev

# Build agent for specific platform
angreal build multi-arch agent --tag test --platforms linux/amd64

# Build all components
angreal build multi-arch all --tag v1.0.0
```

### Publishing to Registry

Add `--push` to publish directly to the registry:

```bash
angreal build multi-arch <component> --tag <tag> --push
```

**Important**: When using `--push`, the build automatically targets both AMD64 and ARM64 unless `--platforms` is specified.

**Examples**:
```bash
# Push broker with multi-arch support
angreal build multi-arch broker --tag v1.0.0 --push

# Push all components
angreal build multi-arch all --tag v1.0.0 --push

# Push to custom registry
angreal build multi-arch broker --tag dev --registry myregistry.io/myorg --push
```

## Pulling Images

### Public Images

Images are publicly accessible and do not require authentication:

```bash
docker pull ghcr.io/colliery-io/brokkr-broker:v1.0.0
```

### Using Specific Architectures

Docker automatically selects the appropriate architecture. To explicitly choose:

```bash
docker pull --platform linux/amd64 ghcr.io/colliery-io/brokkr-broker:v1.0.0
docker pull --platform linux/arm64 ghcr.io/colliery-io/brokkr-broker:v1.0.0
```

### Using Digests

For immutable deployments:

```bash
docker pull ghcr.io/colliery-io/brokkr-broker@sha256:9fc91fae0f07c60ccbec61d86ff93fe825f92c42e5136295552ae196200dbe86
```

## Authentication for Publishing

### GitHub Personal Access Token

Required for manual publishing. Create a token with `write:packages` scope.

**Set environment variable**:
```bash
export GITHUB_TOKEN=ghp_yourtokenhere
```

**Login to registry**:
```bash
docker login ghcr.io -u <your-github-username> --password "$GITHUB_TOKEN"
```

### GitHub Actions

Automated workflows use the built-in `GITHUB_TOKEN` secret with automatic permissions.

## Inspecting Images

### View Manifest

```bash
docker manifest inspect ghcr.io/colliery-io/brokkr-broker:v1.0.0
```

### List Available Tags

Visit the package page:
```
https://github.com/orgs/colliery-io/packages/container/brokkr-broker
```

### Check Image Architecture

```bash
docker image inspect ghcr.io/colliery-io/brokkr-broker:v1.0.0 | grep Architecture
```

## Image Layer Structure

Brokkr images use multi-stage builds optimized for size and security.

### Broker and Agent Images

1. **Planner stage**: Generates cargo-chef recipe
2. **Cacher stage**: Builds dependencies (cached layer)
3. **Builder stage**: Compiles Rust binaries
4. **Final stage**: Minimal Debian slim with runtime dependencies

### UI Image

1. **Builder stage**: npm build
2. **Final stage**: Node.js runtime

## Kubernetes Deployment

### Using Semantic Versions

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: brokkr-broker
spec:
  template:
    spec:
      containers:
      - name: broker
        image: ghcr.io/colliery-io/brokkr-broker:v1.2.3
```

### Using Digests (Recommended)

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: brokkr-broker
spec:
  template:
    spec:
      containers:
      - name: broker
        image: ghcr.io/colliery-io/brokkr-broker@sha256:9fc91fae0f07c60ccbec61d86ff93fe825f92c42e5136295552ae196200dbe86
```

## Image Size Reference

Approximate compressed image sizes:

| Component | AMD64 | ARM64 |
|-----------|-------|-------|
| Broker | ~60 MB | ~58 MB |
| Agent | ~65 MB | ~62 MB |
| UI | ~40 MB | ~38 MB |

*Note: Sizes vary by release and dependency versions*

## Related Documentation

- [Publishing Strategy](../explanation/publishing-strategy.md) - Understanding the tagging and distribution strategy
- [Multi-Architecture Builds](../explanation/multi-arch-builds.md) - Technical details on multi-arch support
