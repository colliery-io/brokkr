---
id: set-up-multi-arch-ci-cd-builds
level: task
title: "Set up multi-arch CI/CD builds"
short_code: "BROKKR-T-0015"
created_at: 2025-10-19T02:26:49.988828+00:00
updated_at: 2025-10-19T02:26:49.988828+00:00
parent: BROKKR-I-0003
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/todo"


exit_criteria_met: false
strategy_id: NULL
initiative_id: BROKKR-I-0003
---

# Set up multi-arch CI/CD builds

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[BROKKR-I-0003]]

## Objective **[REQUIRED]**

Create GitHub Actions workflows for automated multi-architecture (AMD64 + ARM64) container image builds with Docker layer caching, triggered on pull requests and main/develop branch pushes to ensure consistent builds across platforms.

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

## Acceptance Criteria **[REQUIRED]**

- [ ] GitHub Actions workflow for multi-arch image building
- [ ] Matrix strategy for AMD64 and ARM64 platforms
- [ ] Docker layer caching configured for faster builds
- [ ] Workflow triggers on PR, push to main, push to develop
- [ ] Build succeeds for both broker and agent images
- [ ] Multi-arch manifest created and verified
- [ ] Build time optimized with caching (target <10 min for both architectures)
- [ ] Workflow tested on feature branch before merging

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

*To be added during implementation*
