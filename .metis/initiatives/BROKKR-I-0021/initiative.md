---
id: ergonomic-manifest-submission
level: initiative
title: "Ergonomic manifest submission: folder-of-objects on-ramp"
short_code: "BROKKR-I-0021"
created_at: 2026-06-11T02:18:37.762221+00:00
updated_at: 2026-06-11T02:18:37.762221+00:00
parent: BROKKR-V-0001
blocked_by: []
archived: false

tags:
  - "#initiative"
  - "#phase/discovery"


exit_criteria_met: false
estimated_complexity: L
initiative_id: ergonomic-manifest-submission
---

# Ergonomic manifest submission: folder-of-objects on-ramp Initiative

## Context

Submitting a set of Kubernetes objects today means taking a multi-document YAML stream and cramming it into a JSON string field (`CreateDeploymentObjectRequest.yaml_content`), hand-escaping every newline. The actual authoring unit people use — especially while testing — is a **folder of manifest files** edited one at a time (`./manifests/namespace.yaml`, `deployment.yaml`, …), not a single hand-written multi-doc file. There is no client-side helper and no user-facing CLI, so the people who feel this most are the **control-plane authors**: programs that launch many differently-configured application instances across the fleet (the generator is exactly that programmatic actor).

Crucially, the backend model is already correct. A stack's desired state is the single latest deployment object (`get_latest_for_stack` returns one bundle; `get_target_state_for_agent` serves it; the agent applies and prunes by the `k8s.brokkr.io/stack` annotation + checksum). So **1 stack = 1 rendered bundle = the continuously-reconciled target state** — each submit is a new full revision, drift is pruned. The semantics are right; only the front door is rough. This initiative fixes the front door without touching the model.

Two axes of "all over the place": *across clusters* (same bundle, many targets) is already solved by label/annotation targeting. *Across configurations* (many instances, each parameterized) is the rough path — N stacks, each a differently-rendered bundle, created programmatically. This initiative makes the per-instance "folder → submitted bundle" path a breeze.

## Goals & Non-Goals

**Goals**
- A folder of manifests becomes a stack's desired state in **one call**, from every SDK and from a CLI.
- Idempotent submission: create-or-reuse the stack by name, submit a new revision, set targeting — drops straight into a control-plane reconcile loop and into the `kubectl apply -f ./dir` testing loop.
- A raw-YAML submission primitive so the request body is a YAML stream, not a JSON-escaped string (immediate relief for curl users; simpler SDK/CLI submit code).
- Keep the API clean: the server accepts one YAML stream; "folder → stream" is a client-side affordance.

**Non-Goals**
- Server-side rendering or consuming Flux sources (GitRepository/OCIRepository). Explicitly out — the control plane owns rendering; consuming sources undermines the point.
- Changing the 1-stack-=-1-bundle model, or supporting multiple independently-reconciled bundles per stack (need a few bundles → use a few stacks).
- Replacing Brokkr's Tera/JSON-Schema template system — that remains the batteries-included path for standardized, generator-provisioned stacks; this initiative is the bring-your-own-rendered-output path.
- A git-watching GitOps controller.

## Detailed Design

**API — raw-YAML submission primitive.** Make `POST /api/v1/stacks/{id}/deployment-objects` content-type aware:
- `application/json` → existing `CreateDeploymentObjectRequest` path (unchanged, backward compatible).
- `application/yaml` (and `text/yaml`) → the raw request body **is** `yaml_content`; `is_deletion_marker` comes from a `?deletion_marker=` query param.
- Validate-on-ingest: parse the multi-doc YAML and reject malformed input with a clear 400 (`invalid_deployment_object`) instead of a late agent-apply failure.
- Fix the deletion-marker wart: allow an empty body when `deletion_marker=true` (drop the non-empty requirement for markers).
- Optional round-trip: honor `Accept: application/yaml` on `GET /deployment-objects/{id}` to return `yaml_content` as a raw YAML file.

The checksum is already computed server-side, so nothing downstream changes — this is an input-format addition.

**SDK helpers (each language).** Wrapper-layer (hand-written) helpers, no OpenAPI/codegen change:
- `submit_manifests(stack_id, paths | globs | dir)` — walk the directory (`*.yaml`/`*.yml`), concatenate documents with `---`, validate each parses and carries `apiVersion`+`kind`, POST the stream (via the raw-YAML endpoint when available, otherwise the JSON envelope).
- `apply(stack_name, dir, targeting?)` — idempotent: resolve the stack by name, create it if missing (owner = the PAK's generator identity), submit a new revision only when the bundle's checksum changed, set targeting labels. Returns changed/unchanged.
- Ordering is forgiving — the agent already front-loads `Namespace`/`CRD` objects — so naive sorted-filename concatenation is safe. Deleting a file and re-applying prunes the removed object on the next reconcile.

**CLI (`brokkr`).** A new user-facing binary (distinct from the broker/agent admin binaries) over `brokkr-client`. v1:
- `brokkr apply -f <dir|file> --stack <name> [--target-label k=v]` — the `kubectl apply -f ./dir` muscle memory, but it lands in a stack and reconciles across the fleet. Backed directly by the Rust SDK `apply` helper.
- `~/.brokkr/config` (kubeconfig-shaped: broker URL + PAK), overridable by flags/env.
- Idempotent stack create-or-reuse; pruning is automatic via the engine.
- Follow-ons (not v1): `brokkr diff`, `brokkr get -o yaml`, `brokkr stack {create,delete,prune}`.

## Decomposition

| Task | Scope | Depends on |
|---|---|---|
| BROKKR-T-0194 | Raw-YAML submission primitive (broker) + deletion-marker wart fix + `Accept: application/yaml` round-trip | — |
| BROKKR-T-0195 | Rust SDK `submit_manifests`/`apply` folder helper | T-0194 (preferred, not blocking) |
| BROKKR-T-0196 | Python SDK `submit_manifests`/`apply` folder helper | T-0194 (preferred) |
| BROKKR-T-0197 | TypeScript SDK `submit_manifests`/`apply` folder helper | T-0194 (preferred) |
| BROKKR-T-0198 | `brokkr` user CLI: `apply -f <dir> --stack <name>` + config file | T-0195 |

T-0195/0196/0197 are independent of each other and can run in parallel. Each SDK task includes a how-to doc snippet for its language; T-0194 updates the API reference and the manifest-submission how-to.

## Alternatives Considered

- **Multipart / directory-aware API** (server reads a folder): rejected — couples the API to filesystem layout; "folder → stream" belongs client-side, mirroring `kubectl -f`.
- **Structured JSON array of objects** instead of a YAML stream: still viable for programmatic callers and may come later, but YAML-stream + folder-read matches how people actually author (kustomize/helm/kubectl all emit streams) and is the lower-friction on-ramp.
- **Server-side render / Flux source consumption**: explicitly out of scope per the control-plane (not GitOps-replacement) framing.
