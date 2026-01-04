---
id: 001-operator-sidecar-pattern-for-agent
level: adr
title: "Operator Sidecar Pattern for Agent Capabilities"
number: 2
short_code: "BROKKR-A-0002"
created_at: 2025-10-17T09:50:14.075256+00:00
updated_at: 2025-10-17T09:50:14.075256+00:00
decision_date: 2025-10-17
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

# ADR-2: Operator Sidecar Pattern for Agent Capabilities

> **Implementation Status: Superseded for Builds**
> The sidecar pattern described here was superseded by ADR-5 (Shipwright Integration) for container builds. Rather than a custom buildah-operator sidecar, Brokkr now uses Shipwright as the build execution engine, with the agent creating Shipwright CRDs directly. The sidecar pattern remains valid for future capabilities that don't have existing Kubernetes-native solutions.

## Context **[REQUIRED]**

Brokkr agents need to support various optional capabilities beyond core deployment management. The first such capability is container image building via buildah, with future possibilities including:

- Test execution
- Backup operations
- Database migrations
- Security scanning
- Policy enforcement

These capabilities share common characteristics that create architectural challenges:

**Resource consumption**: Operations like container builds can consume significant CPU, memory, and I/O, potentially starving the agent's core responsibilities (heartbeats, deployment polling, status reporting).

**Failure isolation**: Build failures, infinite loops, or crashes in capability-specific code could destabilize the entire agent, disrupting deployment management across the cluster.

**Security boundaries**: Some operations (like builds) may require different security contexts or privileges than core agent functions. For example, rootless buildah vs standard agent permissions.

**Independent lifecycle**: Capabilities should be updateable without redeploying the entire agent. A buildah operator update shouldn't require agent downtime.

**Optional deployment**: Not all agents need all capabilities. GPU-less nodes don't need GPU-aware build support; non-builder agents don't need buildah at all.

Without clear architectural patterns, each new capability risks:
- Bloating the agent binary with optional dependencies
- Creating tight coupling between unrelated concerns
- Complicating the agent's control loop with capability-specific logic
- Increasing blast radius of capability failures

## Decision **[REQUIRED]**

Implement optional agent capabilities as independent operator sidecar containers within agent pods, rather than embedding capability logic in the agent binary or control loop.

**Pattern structure:**
1. Agent container remains focused: poll broker, claim work, apply CRDs, report status
2. Operator sidecar containers: independent Kubernetes controllers watching specific CRD types
3. Communication: Both containers use Kubernetes API (no inter-container communication needed)
4. Isolation: Separate containers, separate resource limits, separate security contexts

**Responsibilities:**

*Agent container:*
- Poll broker for ephemeral work
- Claim work items
- Deserialize CRD specs from broker
- Apply CRDs to Kubernetes cluster
- Watch CRD status via k8s API
- Report completion to broker

*Operator sidecar (e.g., buildah-operator):*
- Watch for specific CRD types (BuildRequest)
- Execute capability-specific operations
- Update CRD status directly
- No broker or agent interaction

## Alternatives Analysis **[CONDITIONAL: Complex Decision]**

| Option | Pros | Cons | Risk Level | Implementation Cost |
|--------|------|------|------------|-------------------|
| **Operator sidecar** (chosen) | Fault isolation; independent scaling; different security contexts; optional deployment; clean separation | More containers per pod; slightly more complex deployment | Low | 1-2 weeks per capability |
| **Embedded in agent** | Simple deployment; single container; shared dependencies | Tight coupling; shared resource limits; capability crashes affect agent; bloated binary | High | 1 week per capability |
| **Agent + HTTP API sidecar** | Some isolation; explicit interface | Over-engineered; need API contract; additional network calls; serialization overhead | Medium | 2-3 weeks per capability |
| **Separate pod deployment** | Maximum isolation; independent scaling | Complex pod coordination; duplicate k8s client setup; work routing complexity | High | 3-4 weeks per capability |

## Rationale **[REQUIRED]**

The operator sidecar pattern was chosen because:

1. **Fault isolation**: Build crashes, infinite loops, or resource exhaustion in the operator sidecar cannot destabilize the agent. Agent continues deployment management even if builds fail.

2. **Resource boundaries**: Each container has independent resource requests/limits. A runaway build hitting memory limits won't affect agent heartbeats or deployment polling.

3. **Security separation**: Operators can run with different security contexts. Rootless buildah can run in a restricted sidecar while agent maintains its permissions.

4. **Independent lifecycle**: Update buildah-operator image without restarting agent. Add new operator sidecars without changing agent code.

5. **Optional deployment**: Agents without builder capability simply omit the sidecar. No code changes, no binary bloat, no unnecessary dependencies.

6. **Kubernetes-native**: Leverages pod-level resource management and security contexts. Operators are standard Kubernetes controllers watching CRDs.

7. **No inter-container coupling**: Both containers independently interact with k8s API. No shared volumes, HTTP APIs, or message queues needed. Simple, decoupled design.

8. **Established pattern**: Sidecar pattern is well-understood in Kubernetes (Istio, log collectors, service mesh). Clear operational model.

## Consequences **[REQUIRED]**

### Positive
- Agent stability protected from capability failures
- Container-level resource isolation and limits
- Independent security contexts per capability
- Optional capabilities via deployment configuration (not code changes)
- Clean separation of concerns (agent orchestrates, operators execute)
- Operators are reusable Kubernetes controllers (can run standalone if needed)
- Easy to add new capabilities without modifying agent
- Update capabilities independently from agent releases
- Standard Kubernetes patterns (sidecars, CRD controllers)

### Negative
- More containers per pod (increases scheduling complexity slightly)
- Each capability needs separate Dockerfile and binary
- Pod configuration more complex (multiple containers, shared service account)
- Capability developers must understand CRD controller patterns
- Resource overhead of multiple containers (though small per-container overhead)

### Neutral
- Agent pods become multi-container (standard Kubernetes pattern)
- Each capability becomes a separate Rust crate (e.g., `brokkr-buildah-operator`)
- Operators watch CRDs independently of agent polling loops
- Communication between agent and operator is implicit (via CRD status)
- Deployment manifests need to specify which sidecars to include
