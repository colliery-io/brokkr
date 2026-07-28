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

When a release git tag (e.g., `v1.2.3`) is pushed, the workflow publishes image tags **without** the `v` prefix:

- **Full version** (`1.2.3`): Exact release identifier
- **Minor version** (`1.2`): Latest patch within that minor version — moves with each patch release
- **Major version** (`1`): Latest minor within that major version — moves with each release
- **Latest** (`latest`): Most recent stable release

**Rationale**: This allows users to choose their update cadence:
- Pin to `1.2.3` for no automatic updates
- Use `1.2` to get patch updates automatically
- Use `1` to track the major version
- Use `latest` for the bleeding edge (not recommended for production)

### Branch Tags

A push to a tracked branch republishes that branch's tag (e.g. `main`), overwriting whatever it pointed at before.

**Rationale**:
- Gives a stable name for "the current state of this branch"
- Deliberately mutable: the point is to follow the branch, not to freeze a commit

There is no per-commit tag. The role a `{branch}-{short-sha}` tag would play — pinning a build for reproducibility or bisection — is served by digest references, which are already immutable and unique for the lifetime of the registry. Adding a second immutable identifier for the same image would double the tag churn on the package without adding a capability.

### Nightly Tag

A scheduled nightly workflow on `main` publishes a `:nightly` image for the broker and agent.

**Rationale**:
- Lets early adopters track the bleeding edge of `main` without waiting for a tagged release
- Provides a continuously-validated build artifact that exercises the full test suite daily
- Surfaces breakage on `main` quickly via auto-filed GitHub issues

### Pull Request Tags

A pull request that touches the broker, agent, or chart sources publishes a `pr-{number}` tag (e.g. `pr-123`), refreshed on every push to the PR.

**Rationale**:
- Test changes in isolation before merging
- Share pre-merge builds with reviewers or QA
- Verify changes work in containerized environments

## Tag Immutability

Not all tags are created equal. Understanding mutability is critical for production deployments:

### Immutable Tags

These tags never change once created:
- Full semantic version: `1.2.3`
- Digest references: `@sha256:...`

### Mutable Tags

These tags are updated with new pushes:
- Minor and major versions: `1.2`, `1` (move to the newest matching release)
- Branch tags: `main` (rebuilt on every qualifying push)
- Nightly: `nightly` (rebuilt daily from `main`)
- Latest: `latest`
- PR tags: `pr-123` (rebuilt on every push to the PR)

### Production Deployment Recommendation

For production deployments, **always use digest references** instead of tags:

```yaml
# Best - digest reference (immutable)
image: ghcr.io/colliery-io/brokkr-broker@sha256:9fc91fae...

# Good - full semantic version (immutable)
image: ghcr.io/colliery-io/brokkr-broker:1.2.3

# Acceptable - minor version (mutable; gets patches automatically)
image: ghcr.io/colliery-io/brokkr-broker:1.2

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
docker pull ghcr.io/colliery-io/brokkr-broker:1.0.0
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

- **GitHub Actions**: Publishing jobs authenticate to GHCR with a repository secret scoped to package writes, and the release jobs sit behind a protected environment that requires human approval
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
- Release tags (`v*`) → semantic version tags, plus `latest`
- Scheduled nightly run on `main` → `nightly` tag
- Pushes to `main` and pull requests that touch build inputs → branch and `pr-{number}` tags
- Release tags (`v*`) → the `brokkr-broker` and `brokkr-agent` Helm charts, packaged at the release version and pushed to `oci://ghcr.io/colliery-io/charts`

Charts ship in lockstep with the images: the chart version and app version are both taken from the git tag, so `1.2.3` of a chart always installs `1.2.3` of its image.

**Benefits**:
- Consistent build environment
- Multi-architecture builds guaranteed
- Images and charts released together, never separately
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
- **Image attestations**: Build provenance and SLSA compliance

None of these are wired into the release pipeline today.

## Related Documentation

- [Container Images Reference](../reference/container-images.md) - Repository URLs and tag formats
