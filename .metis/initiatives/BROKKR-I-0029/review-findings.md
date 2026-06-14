# BROKKR-I-0029 — Phase 4 Review Findings (to address, then re-review)

## Diátaxis compliance — 0 blockers, 4 majors, 6 minors
- M1 evaluate.md: "pick one" two-path choice + "Best for…" comparative framing crosses tutorial boundary. FIX: treat evaluate.md as a getting-started ON-RAMP (not a pure tutorial); keep BOTH paths (user-required) but make each a clean linear path and cut comparative editorializing to a crisp one-line chooser.
- M2 fleet-monitoring.md (Step 1 "Read the key signals" bullets ~53-60): teaches FleetAgentRecord field semantics = reference's job. FIX: replace with short task pointer to reference/fleet.md; keep only what to act on.
- M3 reference/fleet.md (Slow-Subscriber ~136-139 + lines 5,143): editorializes ("because… needs no gap concept", panic-safety rationale). FIX: state behavior factually; move the why to explanation (already linked).
- M4 SUMMARY filing: evaluate.md identity. FIX: keep in Getting Started as intentional on-ramp; ensure page doesn't claim to be a tutorial.
- minors: evaluate rationale creep (PAK "never for production", demo-UI para); fleet-monitoring live-update + alert philosophy creep; fleet.md instructional phrasing (lines 9,79); fleet-legibility literal "20-second" (defer to reference).

## Completeness — 0 blockers/majors, 3 minors
- #1 reference/api/README.md:206-208 WS section says "two" endpoints, OMITS /fleet/live. FIX: "two"→"three", add /api/v1/fleet/live (admin-only) — mirror ws-protocol.md.
- #2 ws-protocol.md:103-105 fleet_update trigger prose omits the 20s computed-signal sweep. FIX: add sweep clause + link to reference/fleet.md trigger section.
- #3 monitoring-setup.md has no reciprocal back-link to fleet-monitoring.md. FIX: add a back-link (fleet-monitoring already links out to it).
- VERIFIED PASS: all 16 FleetAgentRecord fields documented; 3 endpoints+frame discoverable; how-to covers 4 tasks; no dangling quick-start; onboarding present.

## Clarity — 0 blockers, 3 majors (evaluate.md) + minors
- MAJ k3s (Path A, bundled) vs kind/k3d (Path B, BYO) never contrasted in chooser. FIX: state in chooser that A bundles k3s (nothing to install), B needs a local cluster.
- MAJ `.[0].id` agent selection (Path A ~60, Path B ~212) can grab wrong agent. FIX: select by name (jq select(.name==...)).
- MAJ redundant agent_id in targets POST body (~83, ~228) unexplained. NOTE: handler requires body agent_id to match path (mismatch→400). FIX: add one-line note it must match the path.
- minors: state expected status ACTIVE before pushing; gloss "target the agent to the stack"; port-forward must stay running + add to teardown; PAK placeholder mismatch — fleet-monitoring uses `pak_...` but real PAKs use `brokkr_...` (default.toml agent.pak) → fix fleet-monitoring to `brokkr_...`; drop `AuthPayload.admin` jargon from how-to; define $AGENT_ID in how-to Step 2; reference/fleet.md H1 "Fleet Observability" vs inbound "Fleet Reference" — align; fleet-legibility drop bare Metis IDs / broaden "operators only" scope / gloss internal channel.

## Accuracy — PENDING (await before fix pass; may change content)

## Accuracy — 1 major, 1 minor (everything else VERIFIED CORRECT against source)
- MAJ ws-protocol.md "Broker → consumer (fleet live-push)" (~104-105): omits the 20s computed-signal sweep trigger (same as completeness #2). FIX: add the sweep to the trigger list.
- min container-images.md table (line 13): lists UI at `ghcr.io/colliery-io/brokkr-ui` but it's never built/published by CI (invented pullable ref). FIX: replace repo cell with "n/a — local build only (docker/Dockerfile.ui-slim)" or drop the row; align with the correct UI subsection below.
- VERIFIED CORRECT: all 16 FleetAgentRecord fields/types/nullability (k8s_api_latency_ms i32→i64 via i64::from, correct), admin-only auth on all 3 surfaces, heartbeat push = REST POST /agents/{id}/heartbeat (not WS uplink), no /fleet/{id} route, chart names/versions/OCI refs/ports in evaluate.md, sweep 20s, capacity 1024, metric names, UI-as-demo framing.

## GATE DECISION on M1/M4 (evaluate.md "pick one"): KEEP BOTH PATHS per explicit user decision (both trial vehicles requested). Resolve by positioning evaluate.md as a Getting-Started ON-RAMP (not a pure tutorial), tightening each path to a clean linear sequence and cutting comparative editorializing to a one-line chooser. Splitting Path B into a separate how-to is REJECTED (contradicts user requirement). Waiver reason recorded.
