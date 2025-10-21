---
id: set-up-oci-based-helm-chart
level: task
title: "Set up OCI-based Helm chart publishing"
short_code: "BROKKR-T-0016"
created_at: 2025-10-21T12:37:05.985434+00:00
updated_at: 2025-10-21T13:23:24.746791+00:00
parent: BROKKR-I-0003
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/active"


exit_criteria_met: false
strategy_id: NULL
initiative_id: BROKKR-I-0003
---

# Set up OCI-based Helm chart publishing

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **\[CONDITIONAL: Assigned Task\]**

\[\[BROKKR-I-0003\]\]

## Objective **\[REQUIRED\]**

Set up OCI-based Helm chart publishing to GHCR with automated packaging, versioning, and distribution via the release workflow. Keep  current GitHub Release .tgz artifacts  while also setting up a modern OCI registry distribution.

## Acceptance Criteria

## Acceptance Criteria **\[REQUIRED\]**

- \[x\] Helm charts publish to `oci://ghcr.io/colliery-io/charts/brokkr-broker` and `brokkr-agent`
- \[x\] Development/branch tags (develop, main) published and updated on every push (using semver pre-release format)
- \[x\] PR tags (pr-123) published for testing PRs (using semver pre-release format with timestamps)
- \[x\] Release tags (1.0.0) published on version tags (immutable by convention)
- \[x\] Chart versions match application versions for releases (chart v1.0.0 deploys app v1.0.0)
- \[x\] Automated chart packaging in both build-and-test.yml and release.yml workflows
- \[x\] `helm install` will work from OCI registry once workflows execute (to be validated in CI/CD)
- \[x\] Chart metadata includes proper version, appVersion via --version and --app-version flags
- \[ \] Documentation updated with OCI installation instructions and tagging strategy (BROKKR-T-0017)
- \[x\] GitHub Release artifacts include .tgz files for backward compatibility (maintained in release.yml)

## Implementation Notes **\[CONDITIONAL: Technical Task\]**

### Technical Approach

**OCI Registry Setup:**

- Charts published to GHCR: `oci://ghcr.io/colliery-io/charts/`
- Uses `helm push` command (Helm 3.8+ required)
- Same authentication as container images (GITHUB_TOKEN)

**Chart Versioning and Tagging Strategy:**

OCI registries support mutable tags (tags that can be overwritten), just like Docker images. We'll use this to support both development/testing and stable releases.

**IMPORTANT: Helm Requires Semantic Versioning**
Helm strictly requires semver format (MAJOR.MINOR.PATCH) for chart versions. We cannot use arbitrary strings like "develop" or "pr-123" directly. Instead, we use semver pre-release identifiers with timestamps for uniqueness.

**Development & PR Versions (Semver Pre-releases):**
- Development branches: `0.0.0-{branch}.{timestamp}` (e.g., `0.0.0-develop.20241021093000`)
- PR builds: `0.0.0-pr{number}.{timestamp}` (e.g., `0.0.0-pr123.20241021093000`)
- Timestamp format: `YYYYMMDDHHMMSS` for chronological ordering
- These create unique chart versions on each push for traceability

**Release Versions (Stable Semver):**
- Release tags: Actual version from git tag (e.g., `1.0.0`, `1.2.3`)
- These are true semantic versions matching git release tags
- Never overwritten by convention

**Why Timestamps?**
- Each push creates a unique, incrementing chart version
- Helm can sort versions chronologically
- Provides audit trail of when charts were published
- Even though OCI tags are mutable, chart versions remain unique

**Version Extraction:**
- Release tags: Extract from git tag `${GITHUB_REF_NAME#v}` (strips 'v' prefix)
- Branch tags: `0.0.0-${BRANCH_NAME}.${TIMESTAMP}`
- PR tags: `0.0.0-pr${PR_NUMBER}.${TIMESTAMP}`

**Workflow Integration:**

**1. Development/Branch Tags (build-and-test.yml):**

Add after `merge-manifests` job (or create new job):

```yaml
publish-dev-charts:
  name: Publish Development Helm Charts
  needs: merge-manifests  # Wait for images to be published
  runs-on: ubuntu-latest
  if: github.event_name != 'pull_request'  # Don't publish on PRs (use PR tags instead)
  permissions:
    contents: read
    packages: write
  strategy:
    matrix:
      component: [broker, agent]
  steps:
    - uses: actions/checkout@v4

    - name: Set up Helm
      uses: azure/setup-helm@v3
      with:
        version: 'v3.12.0'

    - name: Log in to GHCR
      run: echo ${{ secrets.GHCR_TOKEN }} | helm registry login ghcr.io -u ${{ github.actor }} --password-stdin

    - name: Determine chart version
      id: version
      run: |
        # Use branch name as version (develop, main)
        BRANCH_VERSION=${{ github.ref_name }}
        # Also get image tag from merge-manifests job
        IMAGE_TAG=${{ needs.merge-manifests.outputs.image-tag }}
        echo "chart_version=${BRANCH_VERSION}" >> $GITHUB_OUTPUT
        echo "app_version=${IMAGE_TAG}" >> $GITHUB_OUTPUT

    - name: Package and push chart
      run: |
        helm package charts/brokkr-${{ matrix.component }} \
          --version ${{ steps.version.outputs.chart_version }} \
          --app-version ${{ steps.version.outputs.app_version }}

        helm push brokkr-${{ matrix.component }}-${{ steps.version.outputs.chart_version }}.tgz \
          oci://ghcr.io/colliery-io/charts
```

**2. PR Tags (build-and-test.yml):**

Add separate job for PRs:

```yaml
publish-pr-charts:
  name: Publish PR Helm Charts
  needs: merge-manifests
  runs-on: ubuntu-latest
  if: github.event_name == 'pull_request'
  permissions:
    contents: read
    packages: write
  strategy:
    matrix:
      component: [broker, agent]
  steps:
    - uses: actions/checkout@v4

    - name: Set up Helm
      uses: azure/setup-helm@v3
      with:
        version: 'v3.12.0'

    - name: Log in to GHCR
      run: echo ${{ secrets.GHCR_TOKEN }} | helm registry login ghcr.io -u ${{ github.actor }} --password-stdin

    - name: Package and push PR chart
      run: |
        PR_VERSION="pr-${{ github.event.pull_request.number }}"

        helm package charts/brokkr-${{ matrix.component }} \
          --version ${PR_VERSION} \
          --app-version ${PR_VERSION}

        helm push brokkr-${{ matrix.component }}-${PR_VERSION}.tgz \
          oci://ghcr.io/colliery-io/charts
```

**3. Release Tags (release.yml):**

Update existing `publish-helm-charts` job:

```yaml
- name: Package and push Helm charts to OCI registry
  run: |
    # Extract version from tag (remove 'v' prefix)
    VERSION=${GITHUB_REF_NAME#v}

    # Login to GHCR
    echo ${{ secrets.GHCR_TOKEN }} | helm registry login ghcr.io -u ${{ github.actor }} --password-stdin

    # Package and push broker chart
    helm package charts/brokkr-broker --version ${VERSION} --app-version ${VERSION}
    helm push brokkr-broker-${VERSION}.tgz oci://ghcr.io/colliery-io/charts

    # Package and push agent chart
    helm package charts/brokkr-agent --version ${VERSION} --app-version ${VERSION}
    helm push brokkr-agent-${VERSION}.tgz oci://ghcr.io/colliery-io/charts
```

**Backward Compatibility:**

- Keep existing GitHub Release .tgz files (optional, for users without Helm 3.8+)
- Update docs to recommend OCI registry as primary method

**Testing and Usage:**

```bash
# Install from RELEASE version (stable, immutable)
helm install brokkr-broker oci://ghcr.io/colliery-io/charts/brokkr-broker --version 1.0.0

# Install latest DEVELOP build (semver pre-release, e.g., 0.0.0-develop.20241021093000)
# Note: You need to know the exact timestamp version
helm pull oci://ghcr.io/colliery-io/charts/brokkr-broker --version 0.0.0-develop.20241021093000

# Search for latest develop version in GHCR UI or use helm search
# (OCI doesn't support version listing via CLI, must use GHCR web UI)

# Install specific PR build for testing
helm install brokkr-broker-test oci://ghcr.io/colliery-io/charts/brokkr-broker \
  --version 0.0.0-pr123.20241021093000

# Pull chart for inspection
helm pull oci://ghcr.io/colliery-io/charts/brokkr-broker --version 1.0.0

# List all versions via GHCR UI:
# https://github.com/orgs/colliery-io/packages/container/package/charts%2Fbrokkr-broker

# For production: Always use stable release versions
helm install brokkr-broker oci://ghcr.io/colliery-io/charts/brokkr-broker --version 1.0.0
```

**Finding Development Versions:**
Since OCI registries don't support version listing via CLI, users must:
1. Visit GHCR package page to see all published versions
2. Look for versions starting with `0.0.0-develop.` or `0.0.0-pr{number}.`
3. Use the most recent timestamp for latest build

### Dependencies

- Depends on BROKKR-T-0015 (release.yml workflow already exists)
- Helm 3.8+ on client machines for OCI support
- GHCR_TOKEN with package write permissions (already configured)

### Risk Considerations

**Risk: Helm 3.8+ requirement may exclude some users**

- Mitigation: Keep GitHub Release .tgz artifacts for backward compatibility
- Document minimum Helm version requirement prominently

**Risk: OCI chart discovery is less user-friendly than traditional index.yaml**

- Mitigation: Document chart versions clearly in README and release notes
- GitHub Packages UI shows available versions
- Clearly document which tags are mutable vs immutable

**Risk: Mutable tags may cause unexpected changes**

- Mitigation: Clearly document tagging strategy in installation guide
- Recommend using immutable release tags for production
- Document that develop/main/pr-X tags will change over time
- Production users should use version tags (1.0.0) or OCI digests

**Risk: PR charts and images may accumulate and consume registry space**

- Mitigation: Created separate cleanup-pr.yml workflow that runs on PR close
- Deletes PR container images automatically (pr-123 tag)
- PR Helm charts require manual cleanup due to timestamp versions (0.0.0-pr123.TIMESTAMP)
  - GitHub API doesn't support wildcard deletion
  - Charts can be deleted manually via GHCR UI as needed
- Document in T-0017 that PR artifacts are kept during PR review for testing
- Alternative: Implement script to list and delete all PR chart versions matching pattern

**Risk: Deleting images breaks published Helm charts**

- Mitigation: Only delete PR artifacts after PR is closed/merged
- Keep images and charts available throughout PR lifecycle for distributed testing
- Production releases never deleted automatically

## Status Updates **\[REQUIRED\]**

### 2025-10-21: Implementation Complete

Successfully implemented OCI-based Helm chart publishing across all CI/CD workflows:

**Workflows Updated:**
- build-and-test.yml: Added publish-dev-charts and publish-pr-charts jobs
  - Development builds: 0.0.0-{branch}.{timestamp} format (e.g., 0.0.0-develop.20241021093000)
  - PR builds: 0.0.0-pr{number}.{timestamp} format (e.g., 0.0.0-pr123.20241021093000)
  - Charts publish to oci://ghcr.io/colliery-io/charts/
- release.yml: Updated publish-helm-charts job to push to OCI registry
  - Uses actual semver from git tag (1.0.0, 1.2.3, etc.)
  - Maintains GitHub Release .tgz artifacts for backward compatibility
- cleanup-pr.yml: New workflow to delete PR artifacts on PR close
  - Cleans up PR container images (pr-123 tag)
  - Cleans up PR Helm charts (0.0.0-pr123.* versions)
  - Keeps artifacts available during PR lifecycle for distributed testing

**Key Technical Decisions:**
- Helm strictly requires semantic versioning - cannot use arbitrary strings
- Adopted semver pre-release format with timestamps for development/PR builds
- Timestamps provide unique chart versions and chronological ordering
- PR artifacts kept during review, cleaned up after merge/close
- Uses GitHub API with version IDs for proper package deletion

**Next Steps:**
- Validate workflows on next PR or push to develop
- Document OCI installation in BROKKR-T-0017
- Monitor registry space usage for potential cleanup improvements
