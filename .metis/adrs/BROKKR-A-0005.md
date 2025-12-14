---
id: 001-shipwright-build-integration-for
level: adr
title: "Shipwright Build Integration for Container Image Builds"
number: 1
short_code: "BROKKR-A-0005"
created_at: 2025-10-22T21:38:44.990295+00:00
updated_at: 2025-10-22T22:41:12.326786+00:00
decision_date:
decision_maker: Dylan Storey
parent:
archived: false

tags:
  - "#adr"
  - "#phase/decided"


exit_criteria_met: false
strategy_id: NULL
initiative_id: NULL
---

# ADR-5: Shipwright Build Integration for Container Image Builds

## Context **[REQUIRED]**

BROKKR-I-0001 (Ephemeral Work System with Container Builds) originally planned to implement container image builds through a custom BuildRequest CRD with a buildah operator sidecar. This approach would require:

**Custom Implementation Requirements:**
- Custom BuildRequest CRD specification
- New brokkr-buildah-operator crate (independent CRD controller)
- Git clone functionality with authentication
- buildah CLI integration and execution
- Registry push logic with secret-based authentication
- Multi-container agent pod orchestration (agent + buildah-operator sidecar)
- Build-specific security contexts and capabilities management
- Comprehensive testing of build logic, failure scenarios, and edge cases
- Multi-architecture build implementation
- Vulnerability scanning implementation
- Build caching strategies
- Estimated timeline: 11-15 weeks

**External Research Findings:**
Research into Kubernetes-native build systems revealed Shipwright Build (CNCF Sandbox project) as a mature, production-ready alternative:

- **Maturity**: v1beta1 API stability (v0.17.0 as of September 2025)
- **Backing**: CNCF Sandbox project, maintained by Red Hat, IBM
- **Production Usage**: IBM Cloud Code Engine, Red Hat OpenShift Builds v2
- **Features**: Multi-arch builds, vulnerability scanning (Trivy), comprehensive retry logic, build caching
- **Tekton Dependency**: Requires Tekton Pipelines v0.59+ as execution backend
- **Strategy Pattern**: Extensible ClusterBuildStrategy system (buildah, Kaniko, Buildpacks, etc.)

The key question: Should we build custom build infrastructure from scratch (complete control, 11-15 weeks) or leverage mature CNCF tooling (production-ready, 8-10 weeks, external dependency)?

## Decision **[REQUIRED]**

Adopt a **hybrid approach** for container image builds:

1. **Initial Implementation**: Integrate Shipwright Build (v1beta1) as the build execution engine for BROKKR-I-0001
2. **Generic Foundation**: Preserve generic ephemeral work system (valuable for tests, backups, migrations regardless of build implementation)
3. **Evaluation Checkpoint**: At 8-10 weeks, formally evaluate Shipwright against requirements
4. **Preserve Optionality**: Maintain option to build custom buildah operator if Shipwright has critical limitations

**Implementation Details:**
- Agent creates Shipwright Build/BuildRun CRDs instead of custom BuildRequest
- Shipwright + Tekton (installed on agent clusters) handle build execution
- Agent watches BuildRun status and reports to broker
- Agent remains single-container (no buildah operator sidecar)
- work_type: 'shipwright-build' in ephemeral work system

## Alternatives Analysis **[CONDITIONAL: Complex Decision]**

| Option | Pros | Cons | Risk Level | Implementation Cost |
|--------|------|------|------------|---------------------|
| **Shipwright Build** (chosen) | v1beta1 API stability; Production features (multi-arch, vuln scan, retry); CNCF/Red Hat/IBM maintained; 40% time savings; Battle-tested | Tekton dependency; External dependency risk; Operational complexity; Team learning curve | Medium | 8-10 weeks |
| **Custom buildah Operator** | Complete control; No external dependencies; Simpler stack; Lighter weight; ADR-2 aligned | 11-15 weeks; Maintenance burden; Security responsibility; Feature gaps; Reinventing wheel | Low-Medium | 11-15 weeks |
| **Direct Tekton** | Flexible Pipelines; v1.0 mature; Rich ecosystem | Lower-level abstraction; More YAML; Still Tekton dependency | Medium | 9-12 weeks |
| **Kaniko Only** | Unprivileged; Simple image | No orchestration; Still need custom operator | Medium | 10-14 weeks |

## Rationale **[REQUIRED]**

Shipwright Build was chosen through a hybrid, de-risked approach for the following reasons:

**1. Time to Value (40% Reduction)**
- Custom implementation: 11-15 weeks for production-ready builds
- Shipwright integration: 8-10 weeks for production-ready builds
- Savings: 5-6 weeks can be reinvested in other initiatives or additional ephemeral work types

**2. Production Readiness from Day One**
- v1beta1 API stability provides confidence in interface consistency
- Battle-tested in production: IBM Cloud Code Engine (global scale), OpenShift Builds v2
- Maintained by CNCF with backing from Red Hat and IBM (not a hobby project)
- Quarterly releases with regular security updates

**3. Feature Completeness**
- Multi-architecture builds (amd64, arm64, ppc64le, s390x) without custom implementation
- Vulnerability scanning via Trivy integration (critical for security-conscious users)
- Comprehensive retry logic already implemented and tested
- Build caching strategies proven in production
- These features would take additional weeks to implement custom

**4. Reduced Maintenance Burden**
- Team not responsible for buildah security model, rootless builds, git clone logic
- Security patches and updates handled by upstream project
- Bug fixes benefit from large community (CNCF project)
- Frees team to focus on Brokkr-specific value (ephemeral work system, broker/agent orchestration)

**5. Generic Ephemeral Work System Still Valuable**
- The investment in ephemeral work queue, targeting, retry logic is useful regardless of build implementation
- Future work types (test execution, backups, migrations) benefit from generic system
- Not "wasted work" if Shipwright doesn't work out

**6. Preserved Optionality**
- Evaluation checkpoint at 8-10 weeks provides decision point
- Can build custom buildah operator later if Shipwright has critical limitations
- Hybrid approach de-risks external dependency commitment
- Learning from Shipwright integration informs custom operator design if needed

**7. Alignment with Brokkr's Vision**
- Brokkr vision emphasizes "complementing existing investments" not reinventing
- Using mature CNCF tooling aligns with ecosystem integration philosophy
- Enables faster delivery of end-to-end deployment pipeline (builds → deploy via Brokkr)

**Tradeoff Acceptance:**
The Tekton dependency and operational complexity are accepted tradeoffs because:
- Tekton is v1.0 stable, widely deployed, well-documented
- Operational complexity is offset by not maintaining custom build infrastructure
- Agent clusters will already have operational infrastructure (monitoring, logging, etc.)

## Consequences **[REQUIRED]**

### Positive
- **Faster Time to Market**: 8-10 weeks vs 11-15 weeks (40% reduction) delivers build capability sooner
- **Production-Ready Features**: Multi-arch builds, vulnerability scanning, build caching without custom implementation
- **Reduced Maintenance**: Upstream project (CNCF/Red Hat/IBM) handles security patches, bug fixes, feature additions
- **Simpler Agent Architecture**: Single-container agent pod (no buildah operator sidecar needed)
- **Battle-Tested Reliability**: Leverage production-proven system (IBM Cloud Code Engine, OpenShift)
- **Generic System Investment**: Ephemeral work queue reusable for future work types (tests, backups, migrations)
- **Evaluation-Based Decision**: Checkpoint at 8-10 weeks allows informed decision to continue or pivot
- **Security by Default**: Shipwright's mature security model (rootless builds, secret management) proven in production
- **Community Support**: Large CNCF community, extensive documentation, regular releases

### Negative
- **External Dependency**: Reliance on Shipwright/Tekton roadmap and maintenance (mitigation: can build custom later)
- **Tekton Requirement**: Agent clusters must have Tekton v0.59+ installed (adds operational component)
- **Operational Complexity**: More components to monitor and troubleshoot (Tekton + Shipwright vs single operator)
- **Learning Curve**: Team must understand Shipwright Build/BuildRun model and ClusterBuildStrategy pattern
- **Cluster Prerequisites**: Installation and configuration of Shipwright + Tekton on each agent cluster
- **Potential Migration Risk**: If Shipwright doesn't meet needs, migration to custom operator could disrupt users
- **Limited Build Tool Control**: Constrained to Shipwright's ClusterBuildStrategy options (though extensible)

### Neutral
- Agent cluster resource consumption increases slightly (Tekton + Shipwright controllers vs single operator)
- Documentation shifts from custom CRD to Shipwright Build/BuildRun usage
- Work type identifier: 'shipwright-build' instead of 'build' in ephemeral work system
- Agent uses kube-rs with Shipwright custom resource definitions
- Build authentication follows Shipwright's secret patterns (pushSecret, cloneSecret)
- BuildRun status conditions map to broker ephemeral work status

## Review Schedule **[CONDITIONAL: Temporary Decision]**

This is a hybrid decision with a built-in evaluation checkpoint. The decision will be reviewed after initial implementation.

### Review Triggers
- **Completion of BROKKR-I-0001 Phase 6** (Testing & Evaluation phase at 8-10 weeks)
- **Critical Shipwright limitation discovered** during implementation that blocks key requirements
- **Shipwright project governance changes** (e.g., losing CNCF status, major maintainer departure)
- **Production incidents** caused by Shipwright/Tekton operational issues
- **Significant performance issues** not resolvable through configuration

### Scheduled Review
- **Next Review Date**: End of BROKKR-I-0001 Phase 6 (estimated 8-10 weeks from start)
- **Review Criteria**:
  - **Feature Coverage**: Does Shipwright support 80%+ of build requirements?
  - **Performance**: Are build times and resource usage acceptable for production workloads?
  - **Operational Complexity**: Is Tekton + Shipwright manageable with current team size/skills?
  - **Reliability**: Has Shipwright proven stable during testing and evaluation?
  - **Limitations**: What use cases does Shipwright not support? Are they critical?
- **Possible Outcomes**:
  - **Continue with Shipwright**: Mark ADR as decided/permanent, document as standard build solution
  - **Build Custom Operator**: Create new initiative for custom BuildRequest + buildah operator
  - **Hybrid Approach**: Use Shipwright for most builds, custom operator for specialized cases

### Evaluation Success Criteria
For Shipwright to be considered successful and permanent:
1. Supports git-based builds with private repository authentication
2. Successfully builds and pushes to registries with authentication
3. Build failure retry logic works with broker's ephemeral work queue
4. Performance acceptable: builds complete within reasonable time for project size
5. Operational stability: no critical incidents during evaluation period
6. Team comfortable with Shipwright troubleshooting and configuration

**If ANY of the above criteria fail critically**, initiate custom buildah operator initiative.
