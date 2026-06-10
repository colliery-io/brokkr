# Container Images Reference

This reference documents Brokkr's container images, repository locations, tag formats, and image characteristics. For build and publish procedures, see [Building and Publishing Images](../how-to/build-and-publish-images.md).

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

Created when a git tag matching `v*.*.*` is pushed. The image tags carry no `v` prefix.

| Tag Format | Example | Description | Mutable |
|------------|---------|-------------|---------|
| `{major}.{minor}.{patch}` | `1.2.3` | Full semantic version | No |
| `{major}.{minor}` | `1.2` | Latest patch in minor version | Yes |
| `{major}` | `1` | Latest minor in major version | Yes |
| `latest` | `latest` | Most recent stable release | Yes |

**Example**: Tagging release `v1.2.3` creates:
```
ghcr.io/colliery-io/brokkr-broker:1.2.3
ghcr.io/colliery-io/brokkr-broker:1.2
ghcr.io/colliery-io/brokkr-broker:1
ghcr.io/colliery-io/brokkr-broker:latest
```

### Commit SHA Tags

Created for every commit that triggers a container build.

| Tag Format | Example | Description | Mutable |
|------------|---------|-------------|---------|
| `{branch}-{short-sha}` | `develop-abc1234` | Branch-prefixed short commit SHA | No |

**Example**: Commit `abc1234def5678` on the `develop` branch creates:
```
ghcr.io/colliery-io/brokkr-broker:develop-abc1234
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

Created for pull request builds.

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

A digest reference is immutable: it always resolves to the same image content, whereas mutable tags (`1.2`, `1`, `latest`, branch tags) move as new images are published.

## Inspecting Images

### View Manifest

```bash
docker manifest inspect ghcr.io/colliery-io/brokkr-broker:1.2.3
```

### List Available Tags

Visit the package page:
```
https://github.com/orgs/colliery-io/packages/container/brokkr-broker
```

### Check Image Architecture

```bash
docker image inspect ghcr.io/colliery-io/brokkr-broker:1.2.3 | grep Architecture
```

## Image Layer Structure

Brokkr images use multi-stage builds optimized for size and security.

### Broker and Agent Images

1. **Planner stage**: Generates cargo-chef recipe
2. **Cacher stage**: Builds dependencies (cached layer)
3. **Builder stage**: Compiles Rust binaries
4. **Final stage**: Minimal Debian slim with runtime dependencies

### UI Image

1. **Single stage**: Node.js Alpine (`node:18-alpine`) with npm install and application start

## Image Size Reference

Approximate compressed image sizes:

| Component | AMD64 | ARM64 |
|-----------|-------|-------|
| Broker | ~60 MB | ~58 MB |
| Agent | ~65 MB | ~62 MB |
| UI | ~40 MB | ~38 MB |

*Note: Sizes vary by release and dependency versions*

## Related Documentation

- [Building and Publishing Images](../how-to/build-and-publish-images.md) - Build, publish, and deployment procedures
- [Publishing Strategy](../explanation/publishing-strategy.md) - Understanding the tagging and distribution strategy
