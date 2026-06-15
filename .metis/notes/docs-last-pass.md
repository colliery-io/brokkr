# Docs last-pass — correctness + concision sweep (follow-up to I-0029)

Tree-wide QA over 74 publishable docs (excl. auto-generated rustdoc), two axes:
1. Correctness — claims traceable to code.
2. Concision — tighten; sentence over paragraph; cut hedging/redundancy.

6 parallel reviewers by batch: onboarding | how-to deploy/manage | how-to observe+sdks
| explanation | reference A | reference B. Findings collected here, then applied on
branch docs/last-pass-concision → single PR.

(Metis MCP DB was unavailable this session; tracking here instead.)

## Findings
_pending reviewer returns_

### Batch: Reference B (DONE)
CORRECTNESS:
- reference/health-endpoints.md: version examples "0.6.0" → 0.8.0 (CARGO_PKG_VERSION; health.rs:179). [minor]
- reference/deployment-health.md: line 17 "carry the label" is wrong — attribution is label OR annotation OR ownerReference walk; delete (dupes line 13). line 80 "discovered by the label query" → "discovered/attributed". [minor]
- reference/ws-protocol.md: heartbeat body row (line 59) missing optional k8s_reachable / k8s_api_latency_ms (wire Heartbeat lib.rs:44-54) → add. [minor]
- fleet.md, agent-annotations.md, network-ports.md, container-images.md, monitoring.md, api/README, api/sdks: correctness CLEAN.
CONCISION:
- audit-logs.md: drop redundant intro (line 3); trim 6 curl examples (lines 144-179) → 2-3 or move to how-to.
- health-endpoints.md: cut "Performance Considerations" (228-253, unsourced); drop restate line 13.
- agent-annotations.md: line 56 paragraph repeats row-25 → 1 sentence.
- container-images.md: trim "Image Layer Structure" (117-124) + UI size hedging (128-145).
- monitoring.md: cut "Performance Impact" (284-292, unsourced arithmetic); trim intro restatements (3,13,19).
- deployment-health.md, fleet.md, ws-protocol.md, network-ports.md: tight.

### Batch: Explanation (DONE)
CORRECTNESS:
- explanation/architecture.md: "five background task workers" (72) & "runs five concurrent background tasks" (121) STALE — now 7 maintenance (incl agent-metrics-refresh, agent-events-cleanup) + fleet-sweep + WS-eviction (commands.rs:131-181, api/mod.rs:212-224). → genericize/relist. [major]
- explanation/architecture.md: "all twenty-two entity types" (105) → 24 DAL accessors (dal/mod.rs:257-460) → "two dozen"/drop number. [major]
- explanation/architecture.md: "Performance Characteristics" (291-295) unsourced numbers → soften to qualitative. [minor]
- explanation/components.md: "Broker Startup Sequence" step 7 (331-340) + "Background Tasks Module" (119-131) omit newer tasks — same stale set → relist/genericize. [major]
- explanation/core-concepts.md: agent_targets framing (64,92) softer than newer docs' read-time-resolution; optional align. [minor]
- data-model, network-flows, data-flows, security-model, publishing-strategy, work-orders, template-system, reconciliation, internal-ws-channel, fleet-legibility, README: correctness CLEAN.
CONCISION (prime targets — long docs):
- architecture.md: cut repeated pull-vs-push rationale (56-58); compress init-sequence (62-72, dup of components.md); compress bg-task paragraphs (123-131); read-time-resolution stated 3× (244,265,277)→1.
- data-flows.md: deployment-object states defined in diagram AND prose (91-117)→pick one; read-time-union 4×→1; trim "several advantages" (150-151); immutability dup (481-483).
- security-model.md: trim 4 trust-zone paragraphs (40-46, dup diagram); tighten "Security Principles" (50-58); PAK structure 3×→keep diagram+example; drop generation-process gloss (99); Compliance bullets dup table (388-407)→keep table.
- core-concepts.md: cut analogy (3); collapse broker triple-statement (32-34); cut "Deployment Journey" restatement (109-115).
- network-flows.md: cut throat-clearing (2-4); trim topology prose dup of diagram (40).
- fleet-legibility.md: drop 3rd thesis restatement (62-65); trim log-gap contrast dup of internal-ws (182-194)→pointer.
- components.md "Performance Considerations" (360-380) mild dup; others tight.

### Batch: How-to deploy/manage (DONE)
CORRECTNESS:
- how-to/install-operations.md: `--version 0.6.0` (14,39,44) STALE → 0.8.0 (charts/*/Chart.yaml). [major]
- how-to/multi-tenant-setup.md: schema-name rules (171) omit "must start with a letter"; "max 63" not validated by Brokkr (db.rs:78-97 only checks non-empty/first-alpha/alnum_underscore) → add letter rule, drop/attribute 63-cap. [minor]
- cli-apply, README, templates, generators, pak-management, webhooks, shipwright-builds, build-and-publish-images, managing-stacks: correctness CLEAN.
CONCISION:
- managing-stacks.md: trim intro (3); cut "Stack Naming Conventions" padding (49-55)→constraint+example; tighten line 159.
- generators.md: collapse triple-stated intro (3,6-9)→1; trim "Best Practices" padding (241-259).
- all others in batch: tight.

### Batch: Onboarding (DONE)
CORRECTNESS:
- getting-started/development.md: "Rust 1.85 or later" (7) → 1.90 (Cargo.toml rust-version=1.90). [major]
- tutorials/first-deployment.md:181 "deletion marker still requires a non-empty yaml_content" likely WRONG — deployment_objects.rs:125-128 / stacks.rs:393-396 allow empty for deletion markers. **VERIFY then fix.** [major]
- tutorials/first-deployment.md:213 Note "soft-deleting a stack also soft-deletes its deployment objects and auto-inserts a deletion marker" — reviewer says dal/stacks.rs:170-186 only sets deleted_at + stack.deleted event, no cascade/marker. **VERIFY actual cleanup path then fix.** [major]
- README, getting-started/{README,evaluate,installation,configuration}, tutorials/{README,multi-cluster-targeting,cicd-generators,templates}: correctness CLEAN.
CONCISION:
- README.md: Use Cases (7-17) vs What-Makes-Different (30-40) duplicate ~40 lines → merge; cut marketing line 9; drop "In short" (55, dup of 28).
- getting-started/README.md:3 throat-clearing → "This section covers installing and configuring Brokkr."
- evaluate.md: "5-15 minutes" stated verbatim at 14 AND 23 → drop one (keep 23); trim line 93 parenthetical.
- installation.md: health-check/verify blocks 3× (132-141, 252-275, 412-419) → one canonical; tighten line 28; collapse values-files repetition (162-197).
- others tight.

### Batch: How-to observe + SDKs (DONE)
CORRECTNESS (all minor):
- how-to/sdks/rust.md:15 `brokkr-client = "0.6"` → "0.8" (Cargo.toml:3). [minor]
- how-to/sdks/python.md:43 "exports four names" → five; add ApplyResult (__init__.py __all__). [minor]
- how-to/monitoring-setup.md:228 implies broker /health route — broker only has /healthz,/readyz,/metrics; /health is agent-only (api/mod.rs:242-244, agent/health.rs:84) → scope sentence to agent. [minor]
- how-to/sdks/README.md:27 "0.3.x" lockstep example stale vs 0.8.0 → optional bump.
- deployment-health, log-streaming, fleet-monitoring, diagnostics, troubleshoot, audit-logs, network-configuration, security-hardening, sdks/{typescript,regeneration}: CLEAN. No brokkr_database_* anywhere.
CONCISION:
- deployment-health.md: cut intro dup (3); tighten line 66.
- monitoring-setup.md: trim probe-math restatement (304); tighten line 310.
- network-configuration.md: drop "each suited to different deployment scenarios" (9).
- others tight.

### Batch: Reference A (DONE)
CORRECTNESS:
- reference/environment-variables.md:51-52 PHANTOM vars BROKKR__AGENT__MAX_EVENT_MESSAGE_RETRIES & EVENT_MESSAGE_RETRY_DELAY — no such fields (config.rs Agent 185-245), zero hits in crates/** → DELETE both rows. [BLOCKER]
- reference/generators.md: all JSON response examples (49-57,96-107,139-148,190-198,258-268) omit last_active_at & is_active which the model serializes (generator.rs:74-79; only pak_hash skipped) → add to examples. [major]
- reference/error-codes.md: missing config_reload_disabled (503, admin.rs:234, from PR #75) → add row. [minor]
- reference/soft-deletion.md:153-157 trigger-function table omits cascade_soft_delete_agents (prose+entity table cover it) → add row. [minor]
- reference/templates.md:71 auth line "Admin only" then "Generators can also" → reword "Admin (system templates) or generator (owns it)". [minor]
- reference/work-orders.md: targeting Required nuance (78); backoff exponent uses post-increment (illustrative) — minor, optional.
- README, api/README, cli, diagnostics, multi-tenancy, webhooks: correctness CLEAN.
CONCISION:
- reference/README.md: heavy padding (9-49) → collapse to API pointer + Rust-docs pointer; cut dup link (49).
- api/README.md: trim Swagger bullet overlap (9-16).
- webhooks.md: trim duplicate perf/delivery-mode batch facts (255-291,474-493).
- others tight.

## ALL 6 BATCHES COMPLETE. Apply phase: correctness first, then concision. Verify first-deployment.md:181/213 vs code before editing. Grep docs tree for stale 0.6.0/0.7.0.
