---
id: docs-security-model-claims-only
level: task
title: "Docs: security-model claims only health/metrics are anonymous — console at '/' hands out a read-only admin credential; auth-flow narratives stale across explanation pages"
short_code: "BROKKR-T-0280"
created_at: 2026-07-27T14:19:52.764593+00:00
updated_at: 2026-07-27T17:50:53.060886+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#bug"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: NULL
---

# Docs: security-model claims only health/metrics are anonymous — console at '/' hands out a read-only admin credential; auth-flow narratives stale across explanation pages

## Objective

Correct the security documentation so it describes the post-PR#87 auth reality. This is the highest-severity doc defect from the 2026-07-27 review: the docs currently give a **false security guarantee**.

### Findings (code-verified)

1. **Blocker — security-model.md "Zero Trust by Default" (~line 49)** claims the only anonymous endpoints are `/healthz`, `/readyz`, `/metrics`. In the shipped image (`docker/Dockerfile.broker:23`, built `--features embed-ui`) the console is served anonymously at `/` on the outer router outside the auth nest (`crates/brokkr-broker/src/api/mod.rs:276-279`), and `serve_index_html` injects a live read-only admin UI PAK into the HTML (`api/assets.rs:86-98`; `utils/ui_pak.rs:15-17`: "network access is the auth boundary"). Anyone who can reach port 3000 gets a working read-only admin credential. Operators sizing their network exposure off this page are materially misled.
2. **Major — security-model.md role/endpoint tables (~lines 59-201)** document only three credential classes; the readonly UI PAK class (`AuthPayload.readonly`, `middleware.rs:41-57`), its GET/HEAD + `POST /auth/pak` + `POST /deployment-objects/:id/diagnostics` allowlist (`middleware.rs:150-177`), and admin-gated `GET /api/v1/paks` are absent.
3. **Major — data-flows.md "Authentication Flows" (~lines 306-344)** says the middleware "checks the PAK against three tables in order"; it now checks the in-memory UI PAK first (constant-time, `middleware.rs:205-218`), then the auth cache (`:220-227`), before any DB lookup.
4. **Minor — architecture.md (~line 80)** same stale "three possible identity types" narrative.
5. **Minor — core-concepts.md (~line 148)** "the system supports three types of PAKs."

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing (incorrect security documentation)

### Priority
- [x] P0 - Critical (false security guarantee for exactly the standalone-consumer audience the docs target)

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] security-model.md's anonymous-surface list includes the console root/SPA routes and explains the injected read-only credential + network-boundary rationale and mitigation guidance (network policy / ingress auth), mirroring the existing /metrics note.
- [x] The credential-class documentation covers all four classes with the readonly allowlist.
- [x] data-flows.md, architecture.md, core-concepts.md auth narratives include the UI-PAK pre-check and auth cache.
- [ ] Coordinated with BROKKR-T-0276 (console page) so the two land coherently.

## Status Updates

**2026-07-27** — All four explanation-page fixes landed (agent-assisted, claims re-verified in middleware.rs/ui_pak.rs/assets.rs/paks.rs; docs build passes):
- security-model.md: Zero-Trust anonymous surface corrected; new "Read-Only Console Authentication (the UI PAK)" section; verification-order diagram + prose now UI-PAK → auth cache → admin/agents/generators, with the CLI-rotation-vs-cache-TTL caveat; role table gains the read-only admin row; endpoint table gains GET /api/v1/paks + console routes with the pak_id view-filter note; regulatory mapping clarifies ingress-terminated TLS.
- data-flows.md / architecture.md / core-concepts.md: four-class narratives; architecture gains the per-replica UI PAK sticky-sessions caveat in both scaling sections.
- security-hardening.md additionally gained a "Restrict the Console Surface" checklist item (T-0280 finding, done here since the file was in this workstream).

Open: final coordination pass when the console page (BROKKR-T-0276) is written, so cross-links resolve.