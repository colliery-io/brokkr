---
id: set-up-ghcr-publishing
level: task
title: "Set up GHCR publishing infrastructure"
short_code: "BROKKR-T-0005"
created_at: 2025-10-18T14:47:36.123050+00:00
updated_at: 2025-10-18T14:47:36.123050+00:00
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

# Set up GHCR publishing infrastructure

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[BROKKR-I-0003]]

## Objective **[REQUIRED]**

Configure GitHub Container Registry (GHCR) for container image publishing with proper authentication, repository structure, and tagging strategy.

## Acceptance Criteria **[REQUIRED]**

- [ ] GHCR repository paths configured: `ghcr.io/colliery-io/brokkr-broker` and `ghcr.io/colliery-io/brokkr-agent`
- [ ] GitHub Actions secrets configured for GHCR authentication (GITHUB_TOKEN or PAT)
- [ ] Image tagging strategy documented (semver: v1.0.0, v1.0, v1, latest; SHA: sha-abc1234; branch: main, develop)
- [ ] Manual test push to GHCR succeeds
- [ ] Repository visibility configured (public vs private)
- [ ] Documentation created for image naming conventions

## Implementation Notes **[CONDITIONAL: Technical Task]**

### Technical Approach

**GHCR Configuration:**

1. **Repository Structure**:
   - Broker: `ghcr.io/colliery-io/brokkr-broker`
   - Agent: `ghcr.io/colliery-io/brokkr-agent`
   - Charts (Phase 3): `ghcr.io/colliery-io/charts/brokkr-broker`, `ghcr.io/colliery-io/charts/brokkr-agent`

2. **Authentication Setup**:
   - Use built-in `GITHUB_TOKEN` for GitHub Actions (automatic, no manual setup)
   - For manual/local pushes: Create Personal Access Token (PAT) with `write:packages` scope
   - Login: `echo $GITHUB_TOKEN | docker login ghcr.io -u USERNAME --password-stdin`

3. **Tagging Strategy**:
   - **Semantic versions** (on git tags): `v1.0.0`, `v1.0`, `v1`, `latest`
   - **Commit SHAs** (on PR/main): `sha-abc1234`
   - **Branch names** (on push): `main`, `develop`
   - **PR numbers** (optional): `pr-123`

4. **Test Manual Push**:
   ```bash
   # Build and tag
   docker build -f docker/Dockerfile.broker -t ghcr.io/colliery-io/brokkr-broker:test .

   # Push to GHCR
   docker push ghcr.io/colliery-io/brokkr-broker:test
   ```

5. **Repository Visibility**:
   - Set packages to public for open-source project
   - Configure in GitHub repo settings → Packages

**Files to Create:**
- `docs/publishing.md` - Document tagging strategy and publishing process
- `scripts/tag-and-push.sh` - Helper script for manual publishing

### Dependencies

- Depends on BROKKR-T-0004 (multi-arch builds) for building images to publish
- Enables BROKKR-T-0017 (image publishing workflow)

### Risk Considerations

**Risk: GITHUB_TOKEN permissions insufficient for package publishing**
- Mitigation: Verify token has `write:packages` permission in workflow
- Fallback: Use repository/organization PAT if needed

**Risk: Public packages exposing internal information**
- Mitigation: Review Dockerfile contents for sensitive data
- Ensure no secrets embedded in images

**Risk: Tagging strategy conflicts or overwrites**
- Mitigation: Document tag immutability rules
- Use digest references for production deployments

## Status Updates **[REQUIRED]**

*To be added during implementation*
