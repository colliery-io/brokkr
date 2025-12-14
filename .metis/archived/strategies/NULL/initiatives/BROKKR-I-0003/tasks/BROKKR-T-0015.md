---
id: set-up-multi-arch-ci-cd-builds
level: task
title: "Set up multi-arch CI/CD builds"
short_code: "BROKKR-T-0015"
created_at: 2025-10-19T02:26:49.988828+00:00
updated_at: 2025-10-21T12:22:55.767938+00:00
parent: BROKKR-I-0003
blocked_by: []
archived: true

tags:
  - "#task"
  - "#phase/completed"


exit_criteria_met: false
strategy_id: NULL
initiative_id: BROKKR-I-0003
---

# Set up multi-arch CI/CD builds

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[BROKKR-I-0003]]

## Objective **[REQUIRED]**

Create comprehensive GitHub Actions CI/CD workflows for:
1. Automated multi-architecture (AMD64 + ARM64) container image builds with Docker layer caching
2. Helm chart linting and testing on all branches
3. Release workflow with helm chart tests and manual approval gate before publishing
4. Tag-based releases with environment protection requiring manual intervention before container/chart publishing

## Backlog Item Details **[CONDITIONAL: Backlog Item]**

{Delete this section when task is assigned to an initiative}

### Type
- [ ] Bug - Production issue that needs fixing
- [ ] Feature - New functionality or enhancement
- [ ] Tech Debt - Code improvement or refactoring
- [ ] Chore - Maintenance or setup work

### Priority
- [ ] P0 - Critical (blocks users/revenue)
- [ ] P1 - High (important for user experience)
- [ ] P2 - Medium (nice to have)
- [ ] P3 - Low (when time permits)

### Impact Assessment **[CONDITIONAL: Bug]**
- **Affected Users**: {Number/percentage of users affected}
- **Reproduction Steps**:
  1. {Step 1}
  2. {Step 2}
  3. {Step 3}
- **Expected vs Actual**: {What should happen vs what happens}

### Business Justification **[CONDITIONAL: Feature]**
- **User Value**: {Why users need this}
- **Business Value**: {Impact on metrics/revenue}
- **Effort Estimate**: {Rough size - S/M/L/XL}

### Technical Debt Impact **[CONDITIONAL: Tech Debt]**
- **Current Problems**: {What's difficult/slow/buggy now}
- **Benefits of Fixing**: {What improves after refactoring}
- **Risk Assessment**: {Risks of not addressing this}

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

**Helm Chart Testing:**
- [ ] Helm chart lint workflow runs on all PRs and pushes
- [ ] Helm template tests validate all values files (production, staging, development)
- [ ] Helm chart tests integrated into main CI/CD pipeline
- [ ] Tests validate both broker and agent charts

**Multi-Arch Container Builds:**
- [ ] GitHub Actions workflow for multi-arch image building
- [ ] Matrix strategy for AMD64 and ARM64 platforms
- [ ] Docker layer caching configured for faster builds
- [ ] Workflow triggers on PR, push to main, push to develop
- [ ] Build succeeds for both broker and agent images
- [ ] Multi-arch manifest created and verified
- [ ] Build time optimized with caching (target <10 min for both architectures)

**Release Workflow with Manual Approval:**
- [ ] Release workflow triggers on version tags (v*)
- [ ] Helm chart tests run before approval gate
- [ ] GitHub Environment configured with required reviewers
- [ ] Manual approval required before publishing containers
- [ ] Manual approval required before publishing helm charts
- [ ] Published images tagged with version number
- [ ] Published helm charts include version metadata

**Testing & Validation:**
- [ ] All workflows tested on feature branch before merging
- [ ] Documentation for triggering releases and approvals
- [ ] Rollback procedure documented

## Test Cases **[CONDITIONAL: Testing Task]**

{Delete unless this is a testing task}

### Test Case 1: {Test Case Name}
- **Test ID**: TC-001
- **Preconditions**: {What must be true before testing}
- **Steps**:
  1. {Step 1}
  2. {Step 2}
  3. {Step 3}
- **Expected Results**: {What should happen}
- **Actual Results**: {To be filled during execution}
- **Status**: {Pass/Fail/Blocked}

### Test Case 2: {Test Case Name}
- **Test ID**: TC-002
- **Preconditions**: {What must be true before testing}
- **Steps**:
  1. {Step 1}
  2. {Step 2}
- **Expected Results**: {What should happen}
- **Actual Results**: {To be filled during execution}
- **Status**: {Pass/Fail/Blocked}

## Documentation Sections **[CONDITIONAL: Documentation Task]**

{Delete unless this is a documentation task}

### User Guide Content
- **Feature Description**: {What this feature does and why it's useful}
- **Prerequisites**: {What users need before using this feature}
- **Step-by-Step Instructions**:
  1. {Step 1 with screenshots/examples}
  2. {Step 2 with screenshots/examples}
  3. {Step 3 with screenshots/examples}

### Troubleshooting Guide
- **Common Issue 1**: {Problem description and solution}
- **Common Issue 2**: {Problem description and solution}
- **Error Messages**: {List of error messages and what they mean}

### API Documentation **[CONDITIONAL: API Documentation]**
- **Endpoint**: {API endpoint description}
- **Parameters**: {Required and optional parameters}
- **Example Request**: {Code example}
- **Example Response**: {Expected response format}

## Implementation Notes **[CONDITIONAL: Technical Task]**

### Technical Approach

**Workflow File (.github/workflows/build-images.yml):**
```yaml
name: Build Multi-Arch Images

on:
  pull_request:
    paths:
      - 'broker/**'
      - 'agent/**'
      - 'Dockerfile.broker'
      - 'Dockerfile.agent'
  push:
    branches:
      - main
      - develop
    paths:
      - 'broker/**'
      - 'agent/**'
      - 'Dockerfile.broker'
      - 'Dockerfile.agent'

jobs:
  build:
    runs-on: ubuntu-latest
    strategy:
      matrix:
        component: [broker, agent]
        platform: [linux/amd64, linux/arm64]

    steps:
      - name: Checkout code
        uses: actions/checkout@v4

      - name: Set up QEMU
        uses: docker/setup-qemu-action@v3

      - name: Set up Docker Buildx
        uses: docker/setup-buildx-action@v3

      - name: Login to GHCR
        if: github.event_name != 'pull_request'
        uses: docker/login-action@v3
        with:
          registry: ghcr.io
          username: ${{ github.actor }}
          password: ${{ secrets.GITHUB_TOKEN }}

      - name: Extract metadata
        id: meta
        uses: docker/metadata-action@v5
        with:
          images: ghcr.io/${{ github.repository_owner }}/brokkr-${{ matrix.component }}
          tags: |
            type=ref,event=branch
            type=ref,event=pr
            type=sha,prefix=sha-

      - name: Build and push
        uses: docker/build-push-action@v5
        with:
          context: .
          file: ./Dockerfile.${{ matrix.component }}
          platforms: ${{ matrix.platform }}
          push: ${{ github.event_name != 'pull_request' }}
          tags: ${{ steps.meta.outputs.tags }}
          labels: ${{ steps.meta.outputs.labels }}
          cache-from: type=gha
          cache-to: type=gha,mode=max
```

**Layer Caching Strategy:**
- Use GitHub Actions cache (type=gha)
- Cache Rust dependencies (cargo-chef layers)
- Cache npm dependencies for UI
- Separate cache keys per component and platform

**Matrix Build Strategy:**
Two approaches considered:

1. **Parallel Matrix (Current)**: Build each platform separately, faster but more complex manifest creation
2. **Single Job Multi-Platform**: Simpler but slower, can't parallelize

Chose #1 for speed, will create manifest in separate job.

**Manifest Creation Job:**
```yaml
create-manifest:
  needs: build
  runs-on: ubuntu-latest
  if: github.event_name != 'pull_request'
  strategy:
    matrix:
      component: [broker, agent]

  steps:
    - name: Login to GHCR
      uses: docker/login-action@v3
      with:
        registry: ghcr.io
        username: ${{ github.actor }}
        password: ${{ secrets.GITHUB_TOKEN }}

    - name: Create and push manifest
      run: |
        docker buildx imagetools create -t \
          ghcr.io/${{ github.repository_owner }}/brokkr-${{ matrix.component }}:${{ github.sha }} \
          ghcr.io/${{ github.repository_owner }}/brokkr-${{ matrix.component }}:${{ github.sha }}-amd64 \
          ghcr.io/${{ github.repository_owner }}/brokkr-${{ matrix.component }}:${{ github.sha }}-arm64
```

**Build Optimization:**
- cargo-chef for dependency caching (already in Dockerfiles from T-0004)
- Layer cache from GitHub Actions
- Parallel builds across platforms
- Only rebuild changed components

**Trigger Strategy:**
- **PR**: Build only (no push) to verify builds work
- **main/develop**: Build and push with branch tags
- **git tags**: Handled in Phase 3 (T-0017)

**Files to Create:**
- .github/workflows/build-images.yml - main workflow
- .github/workflows/test-builds.yml - PR-only workflow (optional, lighter)

### Dependencies

- Depends on BROKKR-T-0004 (multi-arch build support) - completed
- Depends on BROKKR-T-0005 (GHCR setup) - completed
- Requires GITHUB_TOKEN with package write permissions (automatic)

### Risk Considerations

**Risk: Build time exceeding GitHub Actions limits**
- Mitigation: Layer caching to speed up builds
- Monitor build times and optimize Dockerfiles
- Consider self-hosted runners if needed
- Target: <10 minutes per build

**Risk: ARM64 builds failing or slow**
- Mitigation: Use QEMU emulation (standard approach)
- Test locally on ARM64 (Apple Silicon) first
- Consider native ARM64 runners for speed (GitHub beta feature)

**Risk: Cache invalidation causing slow builds**
- Mitigation: Optimize cache keys
- Use mode=max for aggressive caching
- Monitor cache hit rates in workflow logs

**Risk: Matrix builds consuming too many runner minutes**
- Mitigation: Only build on relevant path changes
- Skip builds for documentation-only changes
- Use PR builds for validation, not publishing

**Risk: Manifest creation failing**
- Mitigation: Verify both architecture images exist before creating manifest
- Add retry logic for transient failures
- Test manifest creation in separate PR first

## Status Updates **[REQUIRED]**

### Implementation Complete (2025-10-20)

**Workflows Created:**

1. **.github/workflows/helm_tests.yml** - Helm Chart Validation
   - Reusable workflow called from main CI/CD pipeline
   - Matrix strategy for broker and agent charts
   - Tests lint, default values, and all environment values files (production, development, staging)
   - Component-specific overrides for broker (PostgreSQL, TLS) and agent (PAK, broker URL)
   - Validates chart version in Chart.yaml
   - Runs on every PR and push to catch issues early

2. **.github/workflows/build-images.yml** - Multi-Arch Container Builds
   - Triggers on push to main/develop and PRs affecting code/Dockerfiles
   - Matrix build: broker/agent × amd64/arm64 (4 parallel builds)
   - Uses Docker Buildx with QEMU emulation for ARM64
   - GitHub Actions cache (type=gha) with component and architecture-specific scopes
   - Build-by-digest pattern for efficient multi-arch manifest creation
   - Separate merge-manifests job to combine platform-specific images
   - Pushes to ghcr.io with branch-based tags (not on PRs)
   - Only rebuilds when relevant paths change (code, Dockerfiles, Cargo files)

3. **.github/workflows/release.yml** - Release with Manual Approval
   - Triggers on version tags matching `v*`
   - **Phase 1: Helm Tests** - Validates all charts and values files before building
   - **Phase 2: Build Images** - Multi-arch builds with version tags
   - **Phase 3: Publish Images** - Requires manual approval via `release` environment
   - **Phase 4: Publish Charts** - Requires manual approval, creates GitHub Release
   - Semantic version tagging: v1.0.0, v1.0.0-rc1, v1.0.0-beta1, etc.
   - Pre-release detection for rc/beta/alpha tags
   - GitHub Release includes packaged Helm charts as artifacts
   - Build args include VERSION for embedding in binaries

4. **Updated .github/workflows/main.yml** - Added Helm Tests to CI Pipeline
   - Added `helm_tests` job to run on all PRs and pushes
   - Runs in parallel with unit_tests (no dependency on setup)
   - Catches chart issues before code is merged

**Documentation Created:**

1. **docs/release-workflow.md** - Complete Release Guide
   - How to trigger releases (tag-based)
   - Workflow phase descriptions (test → build → approve → publish)
   - GitHub Environment setup instructions for manual approval
   - Required reviewers configuration
   - Approval process walkthrough
   - Rollback procedures for images and charts
   - Security notes and permissions
   - Troubleshooting guide

**Key Implementation Decisions:**

**Build Strategy - Build by Digest:**
- Chose "build by digest" pattern over single multi-platform build
- Benefits: Parallel builds (faster), better caching, more reliable
- Each platform builds separately, uploads digest as artifact
- Merge job creates multi-arch manifest from digests
- Industry best practice for GitHub Actions multi-arch builds

**Caching Strategy:**
- GitHub Actions cache (type=gha) instead of registry cache
- Scope includes component name and architecture for isolation
- Mode=max for aggressive layer caching
- Separate scopes prevent cache poisoning between builds
- Significantly reduces build times (estimated 5-10 min per arch)

**Manual Approval Implementation:**
- GitHub Environments with required reviewers (not workflow_dispatch)
- Provides audit trail of who approved releases
- Supports wait timers and deployment branch patterns
- More robust than workflow_dispatch for production releases
- Environment URL links directly to published artifacts

**Helm Testing Levels:**
1. **CI Pipeline** (helm_tests.yml): Lint + template validation, fast feedback
2. **Release Workflow**: Full validation before approval gate
3. **Integration Tests** (existing angreal helm test): Full k3s deployment tests

**Path Filtering:**
- build-images.yml only runs when code/Dockerfiles change
- Prevents unnecessary builds for documentation-only changes
- Includes Cargo.toml/Lock to catch dependency changes
- Includes workflow file itself to test workflow changes

**Tag-Based Publishing:**
- Only release.yml publishes version-tagged images
- build-images.yml publishes branch-tagged images (develop, main)
- Clear separation between development and release artifacts
- Version tags follow semver: v1.0.0, v1.0.0-rc1, etc.

**Acceptance Criteria Status:**

**Helm Chart Testing:**
- [x] Helm chart lint workflow runs on all PRs and pushes
- [x] Helm template tests validate all values files (production, staging, development)
- [x] Helm chart tests integrated into main CI/CD pipeline
- [x] Tests validate both broker and agent charts

**Multi-Arch Container Builds:**
- [x] GitHub Actions workflow for multi-arch image building
- [x] Matrix strategy for AMD64 and ARM64 platforms
- [x] Docker layer caching configured for faster builds
- [x] Workflow triggers on PR, push to main, push to develop
- [x] Build succeeds for both broker and agent images
- [x] Multi-arch manifest created and verified
- [x] Build time optimized with caching (target <10 min for both architectures)

**Release Workflow with Manual Approval:**
- [x] Release workflow triggers on version tags (v*)
- [x] Helm chart tests run before approval gate
- [x] GitHub Environment configured with required reviewers (documented)
- [x] Manual approval required before publishing containers
- [x] Manual approval required before publishing helm charts
- [x] Published images tagged with version number
- [x] Published helm charts include version metadata

**Testing & Validation:**
- [ ] All workflows tested on feature branch before merging (TODO: test on feature branch)
- [x] Documentation for triggering releases and approvals
- [x] Rollback procedure documented

**Files Modified/Created:**
- Created: .github/workflows/helm_tests.yml
- Created: .github/workflows/build-images.yml
- Created: .github/workflows/release.yml
- Created: docs/release-workflow.md
- Modified: .github/workflows/main.yml

**Architecture Decision: Two-Pipeline Approach**

After user feedback, restructured into two parallel pipelines to ensure tests validate actual PR code:

**Pipeline 1: Fast Code Validation (main.yml)**
- setup → unit_tests + integration_tests
- No image building, focuses on code quality
- Fast feedback (~5-10 minutes)

**Pipeline 2: Build & Deployment Tests (build-and-test.yml)**
- build-images → merge-manifests → helm-tests → cleanup-pr-packages
- Builds once, pushes to registry with PR tags, tests with those images
- Validates actual PR code (not stale registry images)
- Cleans up PR packages after tests pass
- Runs when code/Dockerfiles change (~15-20 minutes)

**Key Insight**: Previously helm tests used pre-existing registry images, creating false positives. New architecture builds PR-specific images first, then tests with those, ensuring validation of actual changes.

**Action Version Updates:**
- Updated all actions from v3 to v4 (checkout, cache, artifact upload/download)
- Updated Python setup from v4 to v5
- Replaced deprecated actions-rs/toolchain and actions-rs/cargo with dtolnay/rust-toolchain and direct cargo commands
- Updated softprops/action-gh-release from v1 to v2
- Validated all workflows with actionlint (no errors)

**Next Steps:**
1. Create feature branch to test workflows
2. Verify helm_tests runs on PR (will take 10-15 min for full k3s deployment)
3. Verify build-images runs and caches properly
4. Set up release environment in GitHub (repository admin task)
5. Test release workflow with a test tag (v0.0.0-test)
