---
id: set-up-ghcr-publishing
level: task
title: "Set up GHCR publishing infrastructure"
short_code: "BROKKR-T-0005"
created_at: 2025-10-18T14:47:36.123050+00:00
updated_at: 2025-10-19T02:03:48.607373+00:00
parent: BROKKR-I-0003
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/completed"


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

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria **[REQUIRED]**

- [x] GHCR repository paths configured: `ghcr.io/colliery-io/brokkr-broker` and `ghcr.io/colliery-io/brokkr-agent`
- [x] GitHub Actions secrets configured for GHCR authentication (GITHUB_TOKEN or PAT)
- [x] Image tagging strategy documented (semver: v1.0.0, v1.0, v1, latest; SHA: sha-abc1234; branch: main, develop)
- [x] Manual test push to GHCR succeeds
- [x] Repository visibility configured (public vs private)
- [x] Documentation created for image naming conventions

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

### 2025-10-18 - Task Completed

Successfully configured GHCR publishing infrastructure:

1. **Authentication**: Configured and tested GHCR authentication using GitHub Personal Access Token
   - Login successful: `docker login ghcr.io -u dstorey --password "$GITHUB_TOKEN"`

2. **Repository Setup**: Published test images to GHCR repositories
   - Broker: `ghcr.io/colliery-io/brokkr-broker:test`
   - Successfully pushed multi-arch ARM64 image
   - Verified with `docker manifest inspect`

3. **Repository Visibility**: Set to public for community evaluation
   - Maintains Elastic License 2.0 protections while allowing easy access
   - Follows ELv2 distribution model (Elasticsearch, Kibana, etc.)

4. **Documentation Created**:
   - `docs/content/explanation/publishing-strategy.md` - Strategy, rationale, and security considerations
   - `docs/content/reference/container-images.md` - Repository URLs, tag formats, and command reference

5. **Tagging Strategy Documented**:
   - Semantic versions: v1.0.0, v1.0, v1, latest
   - Commit SHAs: sha-abc1234
   - Branch names: main, develop
   - Tag immutability rules defined

**Next Steps**: Task BROKKR-T-0017 will implement automated publishing in CI/CD pipeline using this infrastructure.
