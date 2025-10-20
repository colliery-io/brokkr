# Release Workflow

This document describes the release process for Brokkr, including how to trigger releases and the manual approval gates.

## Overview

Brokkr uses a tag-based release workflow with manual approval gates to ensure quality and control over what gets published to production registries.

## Release Process

### 1. Trigger a Release

Releases are triggered by pushing a version tag to the repository:

```bash
# Create and push a version tag
git tag v1.0.0
git push origin v1.0.0
```

Version tags must follow semantic versioning and start with `v`:
- `v1.0.0` - Production release
- `v1.0.0-rc1` - Release candidate (marked as pre-release)
- `v1.0.0-beta1` - Beta release (marked as pre-release)
- `v1.0.0-alpha1` - Alpha release (marked as pre-release)

### 2. Automated Testing Phase

When a version tag is pushed, the release workflow automatically:

1. **Helm Chart Tests** - Full k3s deployment tests via `angreal helm test`
   - Deploys broker and agent charts to k3s cluster
   - Tests all RBAC configurations and values files
   - Validates health endpoints and connectivity
   - Same comprehensive tests as local `angreal helm test all`
   - **Note**: This phase takes 10-15 minutes for full validation

2. **Container Image Builds** - Builds multi-arch images for AMD64 and ARM64
   - Builds broker and agent images for both platforms
   - Creates manifests but does NOT push them yet
   - Stores image digests as artifacts

### 3. Manual Approval Gate

After tests pass and images are built, the workflow waits for manual approval before publishing. This is when you should:

1. **Review Test Results** - Check that all tests passed
2. **Review Changes** - Verify the changes being released are correct
3. **Check Version** - Ensure the version tag is correct

To approve the release:

1. Go to the GitHub Actions tab
2. Click on the "Release" workflow run
3. Look for the "publish-container-images" and "publish-helm-charts" jobs
4. Click "Review deployments"
5. Select the "release" environment
6. Click "Approve and deploy"

### 4. Publishing Phase

After approval, the workflow:

1. **Publishes Container Images**
   - Pushes multi-arch manifests to ghcr.io
   - Tags images with version, major.minor, major, and latest
   - Example: `ghcr.io/colliery-io/brokkr-broker:1.0.0`

2. **Publishes Helm Charts**
   - Packages broker and agent charts
   - Creates GitHub Release with packaged charts
   - Includes auto-generated release notes
   - Marks pre-release appropriately (rc, beta, alpha)

## GitHub Environment Setup

The manual approval gate requires a GitHub Environment named "release" with protection rules.

### One-Time Setup

Repository administrators need to configure the release environment:

1. Go to repository **Settings** → **Environments**
2. Click **New environment**
3. Name it `release`
4. Configure protection rules:
   - ✅ **Required reviewers**: Add 1-2 trusted reviewers
   - ⏱️ **Wait timer**: Optional, e.g., 5 minutes (prevents immediate deploys)
   - 🔒 **Deployment branches**: Limit to tags matching `v*` pattern
5. Click **Save protection rules**

### Required Reviewers

Choose reviewers carefully. They should:
- Understand the release process
- Have authority to approve production deployments
- Be available to review releases in a timely manner

Recommended: Add at least 2 reviewers for redundancy.

## Workflow Diagram

```
┌─────────────────┐
│  Push Tag v*    │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Helm Tests     │  ← Validates all charts and values
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  Build Images   │  ← Multi-arch builds (amd64 + arm64)
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ ⏸  WAIT FOR     │  ← Manual approval required
│   APPROVAL      │
└────────┬────────┘
         │ (approved)
         ▼
┌─────────────────┐
│ Publish Images  │  ← Push to ghcr.io
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│ Publish Charts  │  ← Create GitHub Release
└─────────────────┘
```

## Rollback Procedure

If a release needs to be rolled back:

### Rolling Back Container Images

1. **Re-tag previous version as latest** (if needed):
   ```bash
   docker pull ghcr.io/colliery-io/brokkr-broker:1.0.0
   docker tag ghcr.io/colliery-io/brokkr-broker:1.0.0 ghcr.io/colliery-io/brokkr-broker:latest
   docker push ghcr.io/colliery-io/brokkr-broker:latest
   ```

2. **Update Helm deployments** to use previous version:
   ```bash
   helm upgrade brokkr-broker charts/brokkr-broker \
     --set image.tag=1.0.0 \
     --reuse-values
   ```

### Rolling Back Helm Charts

1. Go to **Releases** page
2. Find the problematic release
3. Click **Edit release**
4. Check **Set as a pre-release** or **Create a draft release**
5. Update release notes to indicate issues
6. Users can download previous chart versions from earlier releases

### Deleting a Tag (Last Resort)

If a release was tagged incorrectly:

```bash
# Delete local tag
git tag -d v1.0.0

# Delete remote tag
git push origin :refs/tags/v1.0.0
```

This will NOT remove published images or charts, only the tag itself.

## CI/CD Workflows

All CI/CD workflows use **angreal** as the single source of truth for test execution. This ensures parity between local testing and CI testing.

### helm_tests.yml

Runs on every PR and push to validate Helm charts via `angreal helm test`:
- Deploys broker and agent charts to k3s cluster
- Tests all RBAC modes and values files (production, development, staging)
- Validates health endpoints and pod readiness
- Full end-to-end deployment testing (same as running `angreal helm test all` locally)
- **Note**: This is a comprehensive test that takes several minutes

### build-images.yml

Builds multi-arch images on PR and push to main/develop:
- Matrix build for broker/agent × amd64/arm64
- Pushes to ghcr.io with branch tags (not version tags)
- Uses GitHub Actions cache for faster builds
- Runs only when Rust code or Dockerfiles change

### release.yml

Triggered by version tags:
- Runs full Helm test suite
- Builds release images with version tags
- Requires manual approval before publishing
- Publishes to ghcr.io and GitHub Releases

## Local Testing Before Release

Before pushing a version tag, test locally using angreal to catch issues early:

```bash
# Test Helm charts (same as CI runs)
angreal helm test all

# Build and test multi-arch images locally (optional)
docker buildx build --platform linux/amd64,linux/arm64 -f Dockerfile.broker -t test:latest .
docker buildx build --platform linux/amd64,linux/arm64 -f Dockerfile.agent -t test:latest .
```

The CI workflows execute the same `angreal` commands, ensuring local/CI parity.

## Troubleshooting

### Release workflow stuck waiting for approval

**Problem**: Workflow shows "Waiting" status for publish jobs.

**Solution**:
1. Check if you're a required reviewer in the environment settings
2. Go to the workflow run and click "Review deployments"
3. If you're not a reviewer, ask an authorized person to approve

### Approval rejected, need to re-run

**Problem**: Release was rejected but you want to publish it.

**Solution**:
1. You cannot re-approve the same workflow run
2. Delete the tag and recreate it:
   ```bash
   git tag -d v1.0.0
   git push origin :refs/tags/v1.0.0
   git tag v1.0.0
   git push origin v1.0.0
   ```
3. This triggers a fresh workflow run that can be approved

### Images built but not published

**Problem**: Build jobs completed but publish jobs never ran.

**Solution**:
1. Check if the release environment exists and has required reviewers
2. Verify the environment name in release.yml matches exactly: `release`
3. Ensure you have permissions to approve deployments

## Security Notes

- **GITHUB_TOKEN** has write permissions to packages - this is required for publishing
- Only repository maintainers can configure the release environment
- Required reviewers should be limited to trusted team members
- Pre-release tags (rc, beta, alpha) still require approval but are marked as pre-release in GitHub

## Related Documentation

- [Helm Chart Values Files](../charts/brokkr-broker/values/README.md)
- [Multi-Arch Container Builds](../docs/multi-arch-builds.md)
- [GitHub Environments Documentation](https://docs.github.com/en/actions/deployment/targeting-different-environments/using-environments-for-deployment)
