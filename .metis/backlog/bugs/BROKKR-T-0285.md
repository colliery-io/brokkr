---
id: broker-chart-advertises-tls
level: task
title: "Broker chart advertises TLS termination (tls.*) but the broker has no TLS code — docs promise encrypted traffic that is plaintext"
short_code: "BROKKR-T-0285"
created_at: 2026-07-27T14:27:43.911639+00:00
updated_at: 2026-07-27T18:14:28.151428+00:00
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

# Broker chart advertises TLS termination (tls.*) but the broker has no TLS code — docs promise encrypted traffic that is plaintext

## Objective

Resolve a false security promise. `grep -rni tls crates/brokkr-broker/src crates/brokkr-utils/src` returns **zero hits** (verified 2026-07-27): the broker binds plain HTTP and reads no `BROKKR__TLS__*` variables. Yet:

- `charts/brokkr-broker/values.yaml:304+` offers `tls.enabled` / `tls.existingSecret`, mounting certs and rendering env vars nothing consumes.
- `charts/brokkr-broker/README.md` instructs enabling "TLS termination at the broker" for production.
- `docs/src/how-to/network-configuration.md` claims "Direct TLS on Broker enables TLS termination at the broker itself... Enable via tls.enabled: true" (two reviewers flagged independently).

An operator following these instructions believes agent↔broker traffic (PAKs in Authorization headers included) is encrypted; it is plaintext.

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing (false security guarantee)

### Priority
- [x] P0 - Critical (credentials transit plaintext on installs believed TLS-secured)

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [x] Decision recorded: either implement broker TLS (larger change) or remove the `tls.*` chart values and rewrite chart README + network-configuration.md + security-hardening.md to prescribe ingress/mesh termination as the only supported TLS path. **Decision (Dylan, 2026-07-27): remove the dead values; TLS terminates at the ingress.**
- [x] No published doc or chart value implies broker-terminated TLS unless the binary supports it.
- [x] Interim: docs warn existing users that `tls.enabled=true` never encrypted anything (network-configuration.md, chart README, security-hardening.md — including rotate-PAKs-that-transited-plaintext guidance).

## Status Updates

**2026-07-27** — Docs side complete (agent-assisted; TLS absence re-verified: zero `tls` matches in broker+utils source, plain bind at `cli/commands.rs:212`):
- network-configuration.md: "Broker TLS Options" replaced with "The Broker Does Not Terminate TLS"; loud warning for existing `tls.enabled=true` users (never encrypted; rotate PAKs that may have transited plaintext); cert-manager example moved to the ingress-annotation flow; agent-to-broker TLS section corrected (agent validates the ingress/proxy cert).
- charts/brokkr-broker/README.md: TLS section rewritten to ingress-only methods; `tls.*` rows in the values table now state the broker reads none of them; production examples no longer set `tls.enabled`.
- security-hardening.md TLS prescription handled in the parallel security-docs workstream.

**2026-07-27 — CHART CLEANUP DONE (Dylan: "do it", TLS terminates at the ingress).** Verified first that `ingress.tls` is a real, wired path (`templates/ingress.yaml` renders hosts + secretName) and fully independent of the dead tree; also re-confirmed zero `BROKKR__TLS` references in any Rust source.

Removed:
- `templates/tls-secret.yaml` and `templates/certificate.yaml` (deleted — the latter only made a standalone cert-manager `Certificate` whose secret was useful solely to something in front of the broker; the supported flow is the ingress `cert-manager.io/cluster-issuer` annotation).
- `tls.*` block from `values.yaml`, replaced with a comment pointing at `ingress.tls`; dropped the stale "tls.* (all TLS settings)" line from the static-settings list.
- `tls:` blocks from `values/{development,staging,production}.yaml`, each replaced with environment-appropriate ingress guidance; production header now points at `ingress.annotations` + `ingress.tls` instead of `tls.certManager.issuer`.
- Cert volume/volumeMount from `deployment.yaml`, `BROKKR__TLS__*` env vars from `configmap.yaml`, and the `tlsSecretName` helper from `_helpers.tpl`.

Docs: chart README's TLS section now reads as a removal/migration note (values are gone; Helm ignores stale entries but delete them; rotate PAKs that transited untrusted networks), values table row replaced with `ingress.tls`, and `docs/src/getting-started/installation.md`'s `tls.enabled` row replaced likewise.

Verified: `helm lint` passes; chart renders with default and all three values files; production render still emits the ingress TLS block (`secretName: brokkr-tls-prod`); `angreal docs build` passes. Note this is a chart-behavior change for anyone who set `tls.*` — Helm silently ignores unknown values, so no install breaks, but the migration note in the README is the user-facing signal.