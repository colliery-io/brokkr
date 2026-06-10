# Building and Publishing Images

This guide covers building Brokkr container images locally, publishing them to a registry, and referencing them in Kubernetes deployments. For repositories, tag formats, and image characteristics, see the [Container Images Reference](../reference/container-images.md).

## Build images locally

Use the `angreal build multi-arch` task:

```bash
angreal build multi-arch <component> [--tag <tag>] [--push] [--platforms <platforms>] [--registry <url>]
```

**Arguments:**

| Argument | Default | Description |
|----------|---------|-------------|
| `component` | (required) | `broker`, `agent`, `ui`, or `all` |
| `--tag` | `dev` | Image tag |
| `--platforms` | `linux/amd64,linux/arm64` | Comma-separated platform list |
| `--registry` | `ghcr.io/colliery-io` | Registry URL prefix |
| `--push` | off | Push to the registry instead of loading locally |

Without `--push`, the image is loaded into the local Docker daemon. Local loads support only a single platform — if the platform list contains multiple entries, the task warns and builds only for your current architecture.

```bash
# Build the broker for your current platform
angreal build multi-arch broker --tag dev

# Build the agent for a specific platform
angreal build multi-arch agent --tag test --platforms linux/amd64

# Build all components
angreal build multi-arch all --tag 1.0.0
```

The task creates (or reuses) a Docker Buildx builder named `brokkr-builder`.

## Publish to a registry

Add `--push` to publish directly to the registry. With `--push`, the build targets both AMD64 and ARM64 unless you narrow `--platforms`:

```bash
# Push the broker with multi-arch support
angreal build multi-arch broker --tag 1.0.0 --push

# Push all components
angreal build multi-arch all --tag 1.0.0 --push

# Push to a custom registry
angreal build multi-arch broker --tag dev --registry myregistry.io/myorg --push
```

## Authenticate for publishing

### GitHub Personal Access Token

Manual publishing to GHCR requires a token with the `write:packages` scope.

Set the environment variable:

```bash
export GITHUB_TOKEN=ghp_yourtokenhere
```

Log in to the registry:

```bash
docker login ghcr.io -u <your-github-username> --password "$GITHUB_TOKEN"
```

### GitHub Actions

Automated workflows authenticate with repository secrets; no manual setup is needed for CI builds.

## Pull images

Images are publicly accessible and do not require authentication:

```bash
docker pull ghcr.io/colliery-io/brokkr-broker:1.0.0
```

Docker automatically selects the appropriate architecture. To explicitly choose one:

```bash
docker pull --platform linux/amd64 ghcr.io/colliery-io/brokkr-broker:1.0.0
docker pull --platform linux/arm64 ghcr.io/colliery-io/brokkr-broker:1.0.0
```

To pin an exact image regardless of tag movement, pull by digest:

```bash
docker pull ghcr.io/colliery-io/brokkr-broker@sha256:9fc91fae0f07c60ccbec61d86ff93fe825f92c42e5136295552ae196200dbe86
```

## Reference images in Kubernetes

### By semantic version

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
        image: ghcr.io/colliery-io/brokkr-broker:1.2.3
```

### By digest

Digest references are immutable — the deployment always runs the exact image content the digest identifies:

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

## Related Documentation

- [Container Images Reference](../reference/container-images.md) - Repositories, tag formats, digests, and image sizes
- [Publishing Strategy](../explanation/publishing-strategy.md) - Tagging and distribution strategy
