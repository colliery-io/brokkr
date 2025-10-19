---
id: validate-phase-1-deliverables
level: task
title: "Validate Phase 1 deliverables"
short_code: "BROKKR-T-0008"
created_at: 2025-10-18T14:47:36.700012+00:00
updated_at: 2025-10-18T14:47:36.700012+00:00
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

# Validate Phase 1 deliverables

*This template includes sections for various types of tasks. Delete sections that don't apply to your specific use case.*

## Parent Initiative **[CONDITIONAL: Assigned Task]**

[[BROKKR-I-0003]]

## Objective **[REQUIRED]**

Comprehensive end-to-end validation of all Phase 1 deliverables: non-root containers, health endpoints, multi-arch images, and Helm charts working together in a real Kubernetes environment.

## Acceptance Criteria **[REQUIRED]**

- [ ] Broker deploys via Helm chart with bundled PostgreSQL
- [ ] Agent deploys via Helm chart and connects to broker
- [ ] All health endpoints respond correctly (/healthz, /readyz, /health)
- [ ] Containers verified running as UID 10001 (non-root)
- [ ] Multi-arch images tested on both AMD64 (CI/CD) and ARM64 (local)
- [ ] Agent successfully registers with broker and sends heartbeats
- [ ] No permission errors in container logs
- [ ] All Phase 1 issues documented for Phase 2 planning

## Implementation Notes **[CONDITIONAL: Technical Task]**

### Technical Approach

**Test Environment:**
- Local: kind or k3s cluster (ARM64 on Apple Silicon)
- CI/CD: GitHub Actions with kind cluster (AMD64)

**Validation Steps:**

1. **Deploy broker with Helm**:
   ```bash
   helm install brokkr-broker ./charts/brokkr-broker \
     --set postgresql.enabled=true \
     --wait --timeout=5m
   ```

2. **Verify broker health**:
   ```bash
   kubectl port-forward svc/brokkr-broker 3000:3000
   curl http://localhost:3000/healthz  # Should return 200 OK
   curl http://localhost:3000/readyz   # Should return 200 OK (DB connected)
   curl http://localhost:3000/health   # Should return JSON with DB status
   ```

3. **Verify non-root execution**:
   ```bash
   kubectl exec -it deploy/brokkr-broker -- id
   # Should show uid=10001(brokkr) gid=10001(brokkr)
   ```

4. **Create agent PAK**:
   ```bash
   kubectl exec -it deploy/brokkr-broker -- \
     ./brokkr-broker create agent \
       --name test-agent \
       --cluster-name local-cluster
   # Save PAK for agent deployment
   ```

5. **Deploy agent with Helm**:
   ```bash
   helm install brokkr-agent ./charts/brokkr-agent \
     --set broker.url=http://brokkr-broker:3000 \
     --set broker.clusterName=local-cluster \
     --set broker.agentName=test-agent \
     --set broker.pak=<PAK_FROM_STEP_4> \
     --wait --timeout=5m
   ```

6. **Verify agent health**:
   ```bash
   kubectl port-forward deploy/brokkr-agent 8080:8080
   curl http://localhost:8080/healthz
   curl http://localhost:8080/readyz  # K8s API + broker connectivity
   curl http://localhost:8080/health  # JSON status
   ```

7. **Verify agent-broker connectivity**:
   ```bash
   kubectl logs deploy/brokkr-agent  # Check for heartbeat logs
   kubectl logs deploy/brokkr-broker # Check for agent registration
   ```

8. **Multi-arch testing**:
   - AMD64: Run in GitHub Actions
   - ARM64: Run locally on Apple Silicon
   - Verify both architectures complete all tests

**Documentation:**
Create `docs/phase1-validation-results.md` with:
- Test results for each step
- Screenshots/logs of successful deployment
- List of issues found and categorized:
  - Blocking (must fix before Phase 2)
  - Improvements (defer to Phase 2)
  - Documentation gaps

### Dependencies

- Depends on ALL previous Phase 1 tasks (BROKKR-T-0001 through T-0007)
- This is the final validation gate for Phase 1

### Risk Considerations

**Risk: Deployment fails due to missing dependencies**
- Mitigation: Test on clean clusters, not dev environments
- Document all prerequisites clearly

**Risk: Health endpoints not accessible (networking)**
- Mitigation: Use kubectl port-forward for initial testing
- Phase 2: Add proper Service/Ingress configuration

**Risk: Multi-arch testing limited by infrastructure**
- Mitigation: At minimum, test ARM64 locally and AMD64 in CI
- Document any architecture-specific issues found

## Status Updates **[REQUIRED]**

*To be added during implementation*
