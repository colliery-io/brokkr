---
title: Container Image Publishing Strategy
weight: 50
---

# Container Image Publishing Strategy

This document explains Brokkr's approach to building and publishing container images, including the reasoning behind our tagging strategy, multi-architecture support, and distribution decisions.

## Publishing to GitHub Container Registry

Brokkr uses GitHub Container Registry (GHCR) as its primary container image registry for several reasons:

- **Integrated authentication**: Leverages GitHub's existing access control and tokens
- **Co-located with source**: Images live alongside the code repository
- **Cost effective**: Free for public open-source projects
- **Multi-architecture support**: Full support for AMD64 and ARM64 platforms
- **OCI compliance**: Standards-compliant container registry

## Public vs Private Distribution

Brokkr container images are published **publicly** despite being licensed under Elastic License 2.0. This decision balances openness with commercial protection:

### Why Public?

- **Easy evaluation**: Users can try Brokkr without requesting access
- **Community adoption**: Lower barrier to entry encourages experimentation
- **Source already available**: The code is public on GitHub, so binaries being public is consistent
- **Modern expectations**: Developers expect to `docker pull` open-source-adjacent projects

### License Protection Remains

Making images publicly accessible does not grant additional rights beyond the Elastic License 2.0:

- Users cannot offer Brokkr as a managed service
- Commercial restrictions still apply
- The license terms must be honored regardless of distribution method
- Source-available ≠ open-source

This approach follows the model used by Elasticsearch, Kibana, and other ELv2 projects.

## Image Tagging Strategy

Brokkr uses multiple tagging strategies to support different use cases and deployment patterns.

### Semantic Versioning for Releases

When a release is tagged (e.g., `v1.2.3`), multiple version tags are created:

- **Full version** (`v1.2.3`): Exact release identifier
- **Minor version** (`v1.2`): Latest patch within minor version
- **Major version** (`v1`): Latest minor within major version
- **Latest** (`latest`): Most recent stable release

**Rationale**: This allows users to choose their update cadence:
- Pin to `v1.2.3` for no automatic updates
- Use `v1.2` to get patch updates automatically
- Use `v1` to track the major version
- Use `latest` for the bleeding edge (not recommended for production)

### Commit SHA Tags

Every commit that triggers a build gets a SHA-based tag (e.g., `sha-abc1234`).

**Rationale**:
- Enables exact reproducibility
- Supports bisecting issues to specific commits
- Provides audit trail for security and compliance
- Immutable and unique across the repository's lifetime

### Branch Tags

Active branches get updated tags matching the branch name (e.g., `main`, `develop`).

**Rationale**:
- Development teams can track branch heads
- Continuous integration environments can use consistent tag names
- Useful for testing changes before they're released

### Pull Request Tags (Optional)

Pull requests can optionally generate tags (e.g., `pr-123`).

**Rationale**:
- Test changes in isolation before merging
- Share pre-merge builds with reviewers or QA
- Verify changes work in containerized environments

## Tag Immutability

Not all tags are created equal. Understanding mutability is critical for production deployments:

### Immutable Tags

These tags never change once created:
- Semantic versions: `v1.2.3`, `v1.2`, `v1`
- SHA tags: `sha-abc1234`

### Mutable Tags

These tags are updated with new pushes:
- Branch names: `main`, `develop`, `feature-xyz`
- Latest: `latest`
- PR tags: `pr-123` (if the PR is updated)

### Production Deployment Recommendation

For production deployments, **always use digest references** instead of tags:

```yaml
# Best - digest reference (immutable)
image: ghcr.io/colliery-io/brokkr-broker@sha256:9fc91fae...

# Good - semantic version (immutable)
image: ghcr.io/colliery-io/brokkr-broker:v1.2.3

# Acceptable - minor version (gets patches)
image: ghcr.io/colliery-io/brokkr-broker:v1.2

# Avoid - mutable tags
image: ghcr.io/colliery-io/brokkr-broker:latest
```

Using digests ensures that a deployment always references the exact image that was tested and approved, preventing unexpected changes from tag updates.

## Multi-Architecture Support

All Brokkr images are built for both AMD64 and ARM64 architectures.

### Why Multi-Architecture?

- **Apple Silicon support**: Developers on M1/M2/M3 Macs run ARM64 natively
- **AWS Graviton**: ARM64 instances offer better price/performance
- **Edge computing**: ARM64 is common in edge and IoT deployments
- **Future-proofing**: ARM64 adoption is accelerating across cloud providers

### Implementation

Brokkr uses Docker Buildx to create multi-architecture manifest lists. When you pull an image, Docker automatically selects the correct architecture:

```bash
# Same command works on AMD64 and ARM64
docker pull ghcr.io/colliery-io/brokkr-broker:v1.0.0
```

The manifest list contains references to both architectures, and Docker pulls the appropriate one based on the host platform.

### Local Development Considerations

Local builds with `--load` can only target a single architecture due to Docker limitations. The build tools automatically detect your platform and build for it:

- Apple Silicon (M1/M2/M3): Builds `linux/arm64`
- Intel/AMD systems: Builds `linux/amd64`

For multi-architecture builds, use `--push` to publish directly to the registry without loading locally.

## Security Considerations

### Image Content Security

Before any image is published:

- **No embedded secrets**: Credentials must never be baked into images
- **Build argument hygiene**: Ensure build args don't leak sensitive data
- **Minimal base images**: Use slim Debian images to reduce attack surface
- **Dependency scanning**: Automated scanning for known vulnerabilities (planned)

### Authentication and Authorization

- **GitHub Actions**: Uses built-in `GITHUB_TOKEN` with automatic permissions
- **Manual publishing**: Requires Personal Access Token with `write:packages` scope
- **Token security**: Tokens stored as GitHub secrets, never committed to source

### Public Registry Security

Public images mean:
- Anyone can pull and inspect the images
- Image layers and content are visible
- Security through obscurity does not apply

Therefore, all security must be built into the application itself, not rely on image privacy.

## Automated vs Manual Publishing

### Automated Publishing (Preferred)

GitHub Actions workflows handle publishing for:
- Release tags → semantic version tags
- Main branch pushes → `main` tag and SHA tags
- Develop branch pushes → `develop` tag and SHA tags

**Benefits**:
- Consistent build environment
- Multi-architecture builds guaranteed
- Security scanning integrated
- Audit trail in GitHub Actions logs

### Manual Publishing

Manual publishing is supported for:
- Testing the build process
- Emergency releases
- Local development verification

Manual builds use the `angreal build multi-arch` command with authentication to GHCR.

## Future Enhancements

Planned improvements to the publishing strategy:

- **Image signing**: Cosign signatures for supply chain security
- **SBOM generation**: Software Bill of Materials for dependency tracking
- **Vulnerability scanning**: Automated Trivy or Grype integration
- **Helm chart publishing**: OCI-based Helm chart distribution via GHCR
- **Image attestations**: Build provenance and SLSA compliance

## Related Documentation

- [Container Images Reference](../reference/container-images.md) - Repository URLs and tag formats
- [Multi-Architecture Builds](./multi-arch-builds.md) - Technical details on multi-arch support
- [CI/CD Pipeline](../reference/cicd.md) - Automated build workflows
