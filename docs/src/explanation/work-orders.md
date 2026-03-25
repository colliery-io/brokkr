# Work Orders & Build System

This document explains the design rationale behind Brokkr's work order system and its integration with Shipwright for container builds. Work orders extend Brokkr beyond Kubernetes manifest distribution into task orchestration across clusters.

## Why Work Orders Exist

Brokkr's core workflow — pushing YAML deployment objects to agents — solves the "distribute manifests" problem well. But real-world operations need more:

- **Build container images** on specific clusters (near source registries, with GPU access, etc.)
- **Run one-off tasks** like database migrations, cleanup scripts, or certificate rotations
- **Execute maintenance operations** that need to happen on specific clusters

Work orders address these needs without changing the pull-based deployment model. They're a parallel system: deployment objects are continuous desired state; work orders are discrete, one-time tasks.

## Design Decisions

### Pull-Based, Not Push-Based

Like deployment objects, work orders use a pull model. The broker doesn't push commands to agents. Instead:

1. Admin creates a work order in the broker
2. Broker determines eligible agents via targeting rules
3. Agents poll for pending work orders they're authorized to claim
4. One agent claims and executes the work order
5. Agent reports completion (success or failure)

This preserves Brokkr's key property: clusters behind firewalls work without inbound connections.

### Single-Claim Semantics

A work order is claimed by exactly one agent. This prevents duplicate execution — you don't want two clusters running the same database migration. The claiming process is atomic: the work order transitions from `PENDING` to `CLAIMED` in a single database transaction.

### Retry with Exponential Backoff

When a work order fails, the system supports automatic retries:

```
PENDING → CLAIMED → (failure) → RETRY_PENDING → (backoff expires) → PENDING → CLAIMED
```

The backoff follows the formula: `2^retry_count × backoff_seconds`

| Retry | Backoff (60s base) | Wait |
|-------|-------------------|------|
| 1st | 2¹ × 60 | 2 minutes |
| 2nd | 2² × 60 | 4 minutes |
| 3rd | 2³ × 60 | 8 minutes |

After `max_retries` failures, the work order moves to the log with `success: false`.

### Why Retryability Is Caller-Declared

The `retryable` flag on completion is set by the agent, not the broker. This is intentional: only the agent knows whether the failure was transient (network timeout, resource contention) or permanent (invalid YAML, missing permissions). Non-retryable failures skip the retry loop entirely.

### Stale Claim Detection

If an agent claims a work order but crashes before completing it, the work order would be stuck in `CLAIMED` forever. The broker's maintenance task detects stale claims:

- Each work order has a `claim_timeout_seconds` (default: 3600 = 1 hour)
- If a claimed work order exceeds its timeout, it's released back to `PENDING`
- The maintenance task runs every 10 seconds

## State Machine

```
                    ┌──────────┐
                    │ PENDING  │◄─────────────────────┐
                    └────┬─────┘                      │
                         │ claim()                    │ backoff expires
                         ▼                            │
                    ┌──────────┐              ┌───────┴──────┐
                    │ CLAIMED  │              │RETRY_PENDING │
                    └────┬─────┘              └──────────────┘
                         │                            ▲
              ┌──────────┴──────────┐                 │
              ▼                     ▼                  │
     ┌────────────────┐    ┌────────────────┐         │
     │ complete_success│    │complete_failure │────────┘
     └────────┬───────┘    └────────┬───────┘  (retryable + retries left)
              │                     │
              ▼                     ▼
     ┌────────────────┐    ┌────────────────┐
     │  WORK_ORDER_LOG│    │  WORK_ORDER_LOG│
     │  success=true  │    │  success=false │
     └────────────────┘    └────────────────┘
```

## Targeting

Work orders support three targeting mechanisms: **hard targets** (explicit agent UUIDs), **label matching**, and **annotation matching**. At least one must be specified — the API rejects work orders with no targeting.

All three mechanisms use **OR logic**: an agent is eligible if it matches *any* of the specified targets, labels, or annotations. When multiple types are combined, they're also OR'd together — agent UUID-1 OR any agent with label `builder:true` can claim the work order.

> **Design note:** This differs from template matching, which uses **AND logic** (a template's labels must *all* be present on the stack). The rationale: work orders need to reach *at least one capable agent* (OR is permissive), while template matching needs to ensure *full compatibility* with a stack (AND is restrictive). See [Template Matching & Rendering](./template-system.md) for comparison.

See the [Work Orders Reference](../reference/work-orders.md) for the complete targeting API with request body examples.

## Shipwright Build Integration

The primary built-in work order type is `build`, which integrates with [Shipwright](https://shipwright.io/) for container image builds.

The agent claims a build work order, applies the Shipwright `Build` resource, creates a `BuildRun`, watches it until completion, and reports back (including the image digest on success). See [Container Builds with Shipwright](../how-to/shipwright-builds.md) for the complete operational guide.

### Why Shipwright?

Shipwright provides a Kubernetes-native build abstraction:

- **No privileged containers** — uses unprivileged build strategies (Buildah, Kaniko, etc.)
- **Cluster-native** — builds run as Kubernetes resources, leverage cluster scheduling
- **Strategy flexibility** — swap between Buildah, Kaniko, ko, S2I without changing build definitions
- **Build caching** — strategies can cache layers for faster rebuilds

Builds have a 15-minute timeout — if the BuildRun doesn't complete in time, it's reported as failed. These timeouts are compile-time constants in the agent, not configurable at runtime.

## Work Order Log

Completed work orders (success or failure) are moved from the active `work_orders` table to the `work_order_log` table. This is an immutable audit trail:

| Active Table | Log Table |
|-------------|-----------|
| `work_orders` | `work_order_log` |
| Mutable (status changes) | Immutable (write-once) |
| Current/pending work | Historical record |
| Cleaned up on completion | Retained indefinitely |

The log records: original ID, work type, timing, claiming agent, success/failure, retry count, and result message.

## Custom Work Orders

Beyond builds, work orders support arbitrary YAML:

```json
{
  "work_type": "custom",
  "yaml_content": "apiVersion: batch/v1\nkind: Job\nmetadata:\n  name: db-migration\nspec:\n  template:\n    spec:\n      containers:\n      - name: migrate\n        image: myapp/migrate:v1\n      restartPolicy: Never"
}
```

Custom work orders apply the YAML to the cluster and monitor completion. This enables arbitrary Kubernetes jobs, CronJobs, or any other resource to be orchestrated through Brokkr.

## Related Documentation

- [Work Orders Reference](../reference/work-orders.md) — API endpoints and data model
- [Container Builds with Shipwright](../how-to/shipwright-builds.md) — setup and usage guide
- [Data Flows](./data-flows.md) — work order lifecycle in context
- [Architecture](./architecture.md) — system-level view
