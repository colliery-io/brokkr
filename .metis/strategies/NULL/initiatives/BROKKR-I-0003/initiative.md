---
id: brokkr-installation-and-deployment
level: initiative
title: "Brokkr Installation and Deployment System"
short_code: "BROKKR-I-0003"
created_at: 2025-10-16T11:26:57.706436+00:00
updated_at: 2025-10-18T14:39:44.309059+00:00
parent:
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/decompose"


exit_criteria_met: false
estimated_complexity: M
strategy_id: NULL
initiative_id: brokkr-installation-and-deployment
---

# Brokkr Installation and Deployment System Initiative

## Context **[REQUIRED]**

Currently, Brokkr components must be deployed manually or through custom deployment scripts, creating barriers to adoption and testing. Organizations need straightforward, production-ready installation methods that align with modern Kubernetes deployment practices. While the broker can be deployed anywhere (on-cluster or external), agents must run within target clusters to manage local resources.

Different organizations have varying infrastructure preferences: some prefer managed databases, others want self-contained deployments, and many need flexibility to choose based on environment (development vs production). The lack of standardized deployment mechanisms forces each user to create custom deployment solutions, duplicating effort and potentially introducing security or configuration issues.

## Goals & Non-Goals **[REQUIRED]**

**Goals:**
- Provide Helm charts for both broker and agent components with production-ready defaults
- Support multiple deployment scenarios: broker with managed PostgreSQL, broker with bundled PostgreSQL, and external broker setups
- Create standalone container images with proper configuration management and health checks
- Enable quick development/testing setups alongside production-ready configurations
- Include comprehensive configuration documentation and common deployment patterns
- Ensure security best practices are built into default configurations

**Non-Goals:**
- Support for non-Kubernetes deployment targets (focus remains on Kubernetes-native deployment)
- Database migration tools or backup/restore functionality (separate operational concerns)
- Custom resource definitions or operators (maintain simplicity with standard Kubernetes resources)
- Integration with specific CI/CD platforms (provide building blocks that work with any system)

## Requirements **[CONDITIONAL: Requirements-Heavy Initiative]**

{Delete if not a requirements-focused initiative}

### User Requirements
- **User Characteristics**: {Technical background, experience level, etc.}
- **System Functionality**: {What users expect the system to do}
- **User Interfaces**: {How users will interact with the system}

### System Requirements
- **Functional Requirements**: {What the system should do - use unique identifiers}
  - REQ-001: {Functional requirement 1}
  - REQ-002: {Functional requirement 2}
- **Non-Functional Requirements**: {How the system should behave}
  - NFR-001: {Performance requirement}
  - NFR-002: {Security requirement}

## Use Cases **[CONDITIONAL: User-Facing Initiative]**

### Use Case 1: Development Environment Setup
- **Actor**: Developer wanting to test Brokkr locally
- **Scenario**:
  1. Developer runs `helm install brokkr-broker ./charts/brokkr-broker --set postgresql.enabled=true`
  2. Broker deploys with bundled PostgreSQL in local cluster
  3. Developer installs agent with `helm install brokkr-agent ./charts/brokkr-agent --set broker.url=http://brokkr-broker:3000`
  4. Complete Brokkr setup running locally for testing
- **Expected Outcome**: Functional Brokkr installation in under 5 minutes

### Use Case 2: Production Multi-Cluster Deployment
- **Actor**: Platform engineer deploying to production
- **Scenario**:
  1. Engineer deploys broker to dedicated cluster with external PostgreSQL using Helm values
  2. Agents deployed to multiple data plane clusters using same Helm chart with different broker URL
  3. TLS certificates and PAK authentication configured through Helm values
  4. Resource limits and monitoring configured for production workloads
- **Expected Outcome**: Secure, scalable Brokkr deployment across multiple clusters

### Use Case 3: External Broker Deployment
- **Actor**: Operations team using existing container orchestration
- **Scenario**:
  1. Team deploys broker container image to existing infrastructure (Docker Swarm, plain containers, etc.)
  2. PostgreSQL connection configured through environment variables
  3. Kubernetes clusters connect via agent Helm charts pointing to external broker URL
  4. Network policies and firewalls configured for secure communication
- **Expected Outcome**: Brokkr broker running outside Kubernetes with agent clusters connecting securely

## Architecture **[CONDITIONAL: Technically Complex Initiative]**

{Delete if not technically complex}

### Overview
{High-level architectural approach}

### Component Diagrams
{Describe or link to component diagrams}

### Class Diagrams
{Describe or link to class diagrams - for OOP systems}

### Sequence Diagrams
{Describe or link to sequence diagrams - for interaction flows}

### Deployment Diagrams
{Describe or link to deployment diagrams - for infrastructure}

## Detailed Design **[REQUIRED]**

### Container Images

Both broker and agent will have optimized container images with:

**Build & Distribution:**
- Multi-stage builds using cargo-chef for dependency caching
- Multi-architecture support: AMD64 + ARM64 (covers x86_64, Apple Silicon, AWS Graviton, Raspberry Pi 4+)
- Docker Buildx for multi-arch builds (can switch to native builds if performance requires)
- Published to GitHub Container Registry: `ghcr.io/colliery-io/brokkr-broker`, `ghcr.io/colliery-io/brokkr-agent`
- Image tagging strategy:
  - Release tags: `v1.0.0`, `v1.0`, `v1`, `latest`
  - Development tags: `sha-abc1234`, `main`, `develop`
- Automated image building and publishing on git tags

**Security:**
- Non-root user execution (UID/GID 10001:10001)
- Stateless containers (no local file writes, all config via env vars/ConfigMaps)
- Security context defined in both Dockerfile and Helm charts
- Vulnerability scanning in CI/CD pipeline

**Health Checks:**
Three endpoint pattern for comprehensive health monitoring:
- `/healthz` - Liveness probe (simple 200 OK if process is alive, no dependencies checked)
- `/readyz` - Readiness probe with dependency checks:
  - Broker: Database connectivity must be healthy
  - Agent: Kubernetes API connectivity must be healthy
- `/health` - Detailed JSON status for monitoring/debugging (includes component status, timestamps)

**Runtime:**
- Configuration via environment variables and config files
- Proper signal handling for graceful shutdowns
- Minimal runtime dependencies

### Broker Helm Chart Structure
```
charts/brokkr-broker/
├── Chart.yaml
├── values.yaml
├── templates/
│   ├── deployment.yaml
│   ├── service.yaml
│   ├── configmap.yaml
│   ├── secret.yaml
│   ├── postgresql.yaml (conditional)
│   └── ingress.yaml (optional)
└── values/
    ├── production.yaml
    └── development.yaml
```

**Key Configuration Options:**
- PostgreSQL: bundled, external, or managed service
- Resource limits and requests
- TLS/SSL configuration
- Authentication secrets
- Storage configuration
- Monitoring and logging

### Agent Helm Chart Structure
```
charts/brokkr-agent/
├── Chart.yaml
├── values.yaml
├── templates/
│   ├── deployment.yaml
│   ├── configmap.yaml
│   ├── secret.yaml
│   ├── rbac.yaml
│   └── serviceaccount.yaml
```

**Key Configuration Options:**
- Broker connection details (URL, authentication)
- Polling intervals and timeouts
- Resource limits
- RBAC permissions for cluster access
- Agent metadata and labels

### Helm Chart Distribution

**Repository Strategy:**
- Charts published to OCI registry via GHCR: `oci://ghcr.io/colliery-io/charts/brokkr-broker`
- Modern OCI-based distribution (requires Helm 3.8+)
- No separate index.yaml maintenance required
- Same infrastructure as container images

**Installation Pattern:**
```bash
# Install broker
helm install brokkr-broker \
  oci://ghcr.io/colliery-io/charts/brokkr-broker \
  --version 1.0.0

# Install agent
helm install brokkr-agent \
  oci://ghcr.io/colliery-io/charts/brokkr-agent \
  --version 1.0.0
```

**Versioning:**
- Chart versions match application versions (e.g., chart v1.0.0 deploys app v1.0.0)
- Simplifies version tracking for users
- Chart-only fixes increment patch version with same app version

**Release Automation:**
- Automated chart packaging and publishing on git tags
- Single git tag triggers: build images → package charts → publish both
- Charts tested in CI/CD before publishing

### Security Considerations
- Default RBAC with minimal required permissions
- Secret management for PAK tokens
- TLS configuration with certificate management
- Network policies for pod-to-pod communication
- Security contexts with non-root execution

## UI/UX Design **[CONDITIONAL: Frontend Initiative]**

{Delete if no UI components}

### User Interface Mockups
{Describe or link to UI mockups}

### User Flows
{Describe key user interaction flows}

### Design System Integration
{How this fits with existing design patterns}

## Testing Strategy **[CONDITIONAL: Separate Testing Initiative]**

{Delete if covered by separate testing initiative}

### Unit Testing
- **Strategy**: {Approach to unit testing}
- **Coverage Target**: {Expected coverage percentage}
- **Tools**: {Testing frameworks and tools}

### Integration Testing
- **Strategy**: {Approach to integration testing}
- **Test Environment**: {Where integration tests run}
- **Data Management**: {Test data strategy}

### System Testing
- **Strategy**: {End-to-end testing approach}
- **User Acceptance**: {How UAT will be conducted}
- **Performance Testing**: {Load and stress testing}

### Test Selection
{Criteria for determining what to test}

### Bug Tracking
{How defects will be managed and prioritized}

## Alternatives Considered **[REQUIRED]**

**1. Kubernetes Operator Pattern**
- **Pros**: Native Kubernetes integration, automatic lifecycle management, custom resource definitions
- **Cons**: Increased complexity, steeper learning curve, additional operational overhead
- **Decision**: Rejected in favor of simpler Helm-based approach that's more universally understood

**2. Single Monolithic Chart**
- **Pros**: Simpler to manage, single deployment command
- **Cons**: Less flexibility for different deployment scenarios, harder to manage component versions independently
- **Decision**: Rejected in favor of separate charts that can be composed as needed

**3. Raw Kubernetes Manifests Only**
- **Pros**: No Helm dependency, maximum transparency
- **Cons**: No templating, difficult to customize, poor user experience for configuration
- **Decision**: Rejected but will provide as generated output from Helm charts

**4. Third-Party Installation Tools (Kustomize, etc.)**
- **Pros**: Alternative to Helm, different templating approaches
- **Cons**: Fragments the installation ecosystem, Helm is more widely adopted for application deployment
- **Decision**: Focus on Helm as primary method, but design to be tool-agnostic where possible

## Implementation Plan **[REQUIRED]**

### Phase 1: Container Images and Base Charts (3 weeks)
- Update Dockerfiles with non-root user (UID 10001:10001)
- Implement health check endpoints (/healthz, /readyz, /health) in both services
- Set up Docker Buildx for multi-arch builds (AMD64 + ARM64)
- Configure GHCR publishing pipeline (ghcr.io/colliery-io)
- Implement image tagging strategy (semver + SHA + branch tags)
- Build basic Helm charts with essential templates
- Test basic deployment scenarios locally on both architectures

### Phase 2: Production-Ready Features (3 weeks)
- Add comprehensive configuration options to Helm charts
- Implement PostgreSQL bundling with persistence options
- Add TLS/SSL support and certificate management
- Create RBAC templates with security best practices
- Add security contexts to pod specs (non-root enforcement)
- Develop values files for different deployment scenarios
- Test multi-arch images in CI/CD (AMD64 always, ARM64 for releases)

### Phase 3: Helm Chart Distribution (2 weeks)
- Set up OCI-based chart publishing to GHCR
- Implement chart versioning matching app versions
- Create automated chart packaging on git tags
- Add Helm linting and security validation to CI/CD
- Test chart installation from OCI registry
- Document chart installation patterns

### Phase 4: Documentation and Examples (2 weeks)
- Write comprehensive installation guides
- Document multi-architecture deployment
- Create example deployment scenarios and tutorials
- Document health check endpoint usage
- Add troubleshooting guides
- Test installation procedures with fresh environments

### Phase 5: Release Automation and Testing (2 weeks)
- Implement end-to-end release automation (git tag → images + charts)
- Add vulnerability scanning for container images
- Automated testing of installation procedures
- Performance testing of different deployment scenarios
- Add monitoring and observability configurations
- Final multi-arch testing on both architectures

**Total Timeline: 12 weeks**

## Task Breakdown **[IMPLEMENTATION REFERENCE]**

This section provides a detailed decomposition of the implementation plan into actionable tasks, organized by phase. Tasks will be created incrementally in Metis as phases progress.

### Current State Analysis

**What Exists:**
- ✅ Dockerfiles for broker and agent with cargo-chef and multi-stage builds
- ✅ Basic `/healthz` and `/readyz` endpoints in broker (no dependency checks)
- ✅ Basic CI/CD pipeline (unit/integration tests)
- ✅ Development on Apple Silicon (ARM64 native)

**What's Missing:**
- ❌ Non-root user setup in Dockerfiles (currently running as root)
- ❌ Readyz endpoints don't check dependencies (DB for broker, K8s for agent)
- ❌ No `/health` endpoint with detailed JSON status
- ❌ Agent health endpoints not implemented (agent is CLI-only currently)
- ❌ Multi-arch build support (kubectl download hardcoded to amd64)
- ❌ Helm charts don't exist
- ❌ Container image building/publishing not in CI/CD
- ❌ GHCR publishing pipeline
- ❌ Chart distribution via OCI registry

### Phase 1: Container Images and Base Charts (Weeks 1-3) - 8 Tasks

**BROKKR-T-0001: Update Dockerfiles for non-root execution**
- Add non-root user (UID/GID 10001:10001) to broker and agent Dockerfiles
- Update WORKDIR ownership and permissions
- Add USER directive before ENTRYPOINT
- Test both containers run correctly as non-root
- Verify no file write permissions issues

**BROKKR-T-0002: Enhance broker health check endpoints**
- Upgrade `/readyz` to validate PostgreSQL connectivity (test DB connection)
- Implement `/health` endpoint with detailed JSON (DB status, uptime, version, timestamp)
- Add comprehensive error handling and appropriate status codes
- Ensure `/healthz` remains simple (no dependency checks)

**BROKKR-T-0003: Implement agent health check endpoints**
- Add HTTP server capability to agent (currently CLI-only application)
- Create `/healthz` endpoint (simple liveness check)
- Create `/readyz` endpoint with Kubernetes API connectivity validation
- Create `/health` endpoint with detailed JSON status
- Match broker endpoint patterns for consistency

**BROKKR-T-0004: Add multi-architecture build support**
- Update agent Dockerfile kubectl download to detect architecture (amd64/arm64)
- Configure Docker Buildx for local development environment
- Create build script for multi-arch images (AMD64 + ARM64)
- Test builds on Apple Silicon locally
- Verify CI/CD pipeline can build both architectures

**BROKKR-T-0005: Set up GHCR publishing infrastructure**
- Configure GHCR repository structure: `ghcr.io/colliery-io/brokkr-broker` and `brokkr-agent`
- Set up GitHub Actions secrets for GHCR authentication (GITHUB_TOKEN)
- Document image naming convention
- Create initial image tagging strategy (semver, SHA, branch tags)
- Test manual push to GHCR

**BROKKR-T-0006: Create broker Helm chart foundation**
- Initialize chart structure with proper Chart.yaml metadata
- Create deployment.yaml template with container spec
- Create service.yaml template
- Create configmap.yaml template for configuration
- Create secret.yaml template for sensitive data
- Add conditional PostgreSQL deployment (bundled option)
- Create basic values.yaml with essential configuration options

**BROKKR-T-0007: Create agent Helm chart foundation**
- Initialize chart structure with proper Chart.yaml metadata
- Create deployment.yaml template
- Create serviceaccount.yaml template
- Create basic rbac.yaml templates (ClusterRole, ClusterRoleBinding)
- Create configmap.yaml for agent configuration
- Create basic values.yaml with broker connection settings

**BROKKR-T-0008: Validate Phase 1 deliverables**
- Deploy broker locally using Helm chart with bundled PostgreSQL
- Deploy agent locally pointing to local broker
- Verify all health endpoints respond correctly (/healthz, /readyz, /health)
- Test multi-arch images on both AMD64 (CI/CD) and ARM64 (local)
- Document any issues or gaps for Phase 2
- Verify non-root execution in running containers

### Phase 2: Production-Ready Features (Weeks 4-6) - 7 Tasks

**BROKKR-T-0009: Add comprehensive configuration to broker chart**
- External PostgreSQL connection configuration options
- Resource requests and limits configuration
- Environment variable management and templating
- Secrets management for database credentials
- Replicas and scaling configuration

**BROKKR-T-0010: Implement PostgreSQL bundling option**
- Add PostgreSQL subchart dependency (bitnami/postgresql or similar)
- Configure conditional deployment based on values.yaml
- Add persistence configuration (PVC, storage class options)
- Configure PostgreSQL connection strings for both scenarios
- Test both bundled and external DB deployment scenarios

**BROKKR-T-0011: Add TLS/SSL support to broker chart**
- Certificate secret templates for TLS termination
- Ingress configuration with TLS support
- Environment variables for TLS enablement in broker
- Document certificate requirements and generation
- Test with self-signed and proper certificates

**BROKKR-T-0012: Create comprehensive RBAC for agent**
- Define minimal required permissions (list/watch pods, namespaces, etc.)
- Create detailed ClusterRole template with specific API groups
- Create ClusterRoleBinding and Role templates
- Add configurable RBAC scope (cluster-wide vs namespace-scoped)
- Document all permission requirements and why they're needed

**BROKKR-T-0013: Add security contexts to all pod specs**
- Add non-root enforcement to Helm templates (runAsNonRoot: true, runAsUser: 10001)
- Configure read-only root filesystem where possible
- Drop all capabilities, add back only required ones
- Configure seccomp profiles (RuntimeDefault)
- Add fsGroup for volume permissions
- Test security constraints in restricted clusters

**BROKKR-T-0014: Create values files for deployment scenarios**
- values/production.yaml (external DB, high resources, strict security, monitoring)
- values/development.yaml (bundled DB, minimal resources, relaxed security)
- values/staging.yaml (middle ground configuration)
- Document when to use each values file
- Add inline comments explaining each major setting

**BROKKR-T-0015: Set up multi-arch CI/CD builds**
- Create GitHub Actions workflow for multi-arch image building
- Configure matrix strategy for AMD64 + ARM64 builds
- Add Docker layer caching strategy for faster builds
- Test workflow on feature branch
- Ensure workflow triggers on appropriate events (PR, push to main/develop)

### Phase 3: Helm Chart Distribution (Weeks 7-8) - 5 Tasks

**BROKKR-T-0016: Implement image tagging strategy**
- Create script/workflow for semantic version tags (v1.0.0, v1.0, v1, latest)
- Add git SHA tags for development builds (sha-abc1234)
- Add branch name tags (main, develop)
- Implement tag propagation logic (v1.0.0 also gets v1.0, v1, latest)
- Test tagging logic with various git scenarios

**BROKKR-T-0017: Create GitHub Actions workflow for image publishing**
- Build multi-arch images on git tag push
- Automated multi-arch manifest creation
- GHCR authentication and image pushing
- Tag propagation implementation
- Test complete workflow from git tag to published images

**BROKKR-T-0018: Set up OCI-based Helm chart publishing**
- Create workflow to package Helm charts (`helm package`)
- Configure `helm push` to GHCR OCI registry
- Implement chart versioning matching app versions
- Test chart publishing to `oci://ghcr.io/colliery-io/charts/`
- Verify chart installation from OCI registry

**BROKKR-T-0019: Add Helm chart linting and validation**
- Add `helm lint` to CI/CD pipeline
- Add chart validation with kubeconform or kubeval
- Add security scanning with Trivy or Checkov
- Configure linting rules and standards
- Fail builds on critical issues, warn on medium issues

**BROKKR-T-0020: Create chart installation testing**
- Automated testing of chart installation in CI/CD (kind or k3s cluster)
- Test both broker and agent chart installation
- Verify connectivity between components
- Test upgrade scenarios (chart version changes)
- Test rollback scenarios

### Phase 4: Documentation and Examples (Weeks 9-10) - 5 Tasks

**BROKKR-T-0021: Write comprehensive installation guide**
- Quick start guide (5-minute local setup)
- Production installation guide with best practices
- External broker deployment guide (non-Kubernetes broker)
- Multi-cluster deployment guide
- Troubleshooting common installation issues

**BROKKR-T-0022: Document multi-architecture deployment**
- Architecture-specific considerations and differences
- ARM64 deployment guide (Apple Silicon, AWS Graviton, Raspberry Pi 4+)
- Performance characteristics comparison by architecture
- Known limitations or issues per architecture
- Build and deployment instructions for each architecture

**BROKKR-T-0023: Create deployment example scenarios**
- Development environment setup tutorial (step-by-step)
- Production multi-cluster setup example with diagrams
- External broker with multiple agents example
- GitOps integration example (ArgoCD/Flux patterns)
- Air-gapped deployment scenario

**BROKKR-T-0024: Document health check endpoint usage**
- Complete endpoint specifications and expected responses
- Kubernetes liveness/readiness probe configuration examples
- Monitoring integration examples (Prometheus, Datadog, etc.)
- Debugging guide using /health endpoint
- Performance impact and best practices

**BROKKR-T-0025: Create troubleshooting guide**
- Common deployment issues and solutions (with symptoms)
- Network connectivity debugging (broker-agent, agent-K8s)
- Authentication and authorization failures
- Resource constraint handling (OOMKilled, CPU throttling)
- Log analysis guide and common error patterns
- Debug mode and verbose logging configuration

### Phase 5: Release Automation and Testing (Weeks 11-12) - 5 Tasks

**BROKKR-T-0026: Implement end-to-end release automation**
- Single git tag triggers full release (images + charts)
- Automated testing before publishing (lint, build, deploy tests)
- Release notes generation from git history
- GitHub release creation with artifacts
- Notification system for successful/failed releases

**BROKKR-T-0027: Add container image vulnerability scanning**
- Integrate Trivy or similar scanner into CI/CD
- Scan on every build (PR and main branches)
- Fail on high/critical vulnerabilities in PRs
- Generate vulnerability reports and summaries
- Configure vulnerability database updates

**BROKKR-T-0028: Create automated installation testing suite**
- E2E tests for chart installation in clean clusters
- Multi-cluster test scenarios (broker in one, agents in others)
- Upgrade path testing (v1.0.0 → v1.1.0)
- Rollback testing and verification
- Chaos testing (pod deletion, network partition)

**BROKKR-T-0029: Add monitoring and observability configuration**
- Prometheus metrics endpoints implementation
- ServiceMonitor CRDs for Prometheus Operator
- Grafana dashboard examples (JSON exports)
- Common metrics documentation (latency, errors, saturation)
- Log aggregation configuration examples (Loki, ELK)
- Tracing integration examples (Jaeger, Tempo)

**BROKKR-T-0030: Perform final multi-arch validation**
- Full deployment testing on AMD64 (x86_64 servers)
- Full deployment testing on ARM64 (Apple Silicon, AWS Graviton)
- Performance benchmarking both architectures (throughput, latency)
- Load testing with different configurations and scales
- Resource usage comparison (CPU, memory)
- Final security audit of images and charts
- Sign-off for production readiness

### Task Creation Strategy

- **Phase 1 tasks (BROKKR-T-0001 to BROKKR-T-0008)**: Create immediately as BROKKR-I-0003 enters decompose phase
- **Phase 2-5 tasks**: Create as Phase 1 nears completion or when new learning requires task adjustment
- **Task dependencies**: Later phase tasks depend on earlier phase completions
- **Iteration**: Task details may be refined based on implementation learnings

### Success Criteria
- Both components can be deployed via Helm charts in under 5 minutes
- Multi-architecture images (AMD64 + ARM64) build and deploy successfully
- All health check endpoints (/healthz, /readyz, /health) functional
- Containers run as non-root user (UID 10001) in both Dockerfile and Helm
- Images published to GHCR with correct tagging strategy
- Charts distributed via OCI registry and installable with Helm 3.8+
- Support for development, staging, and production deployment scenarios
- Comprehensive documentation enables users to deploy without additional support
- Charts pass Helm linting and security validation
- Container images follow security best practices and pass vulnerability scanning
- Automated release pipeline publishes images + charts from single git tag
