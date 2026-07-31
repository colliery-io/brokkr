---
id: docs-helm-existing-secret
level: task
title: "Docs: helm existing-Secret credential sourcing (PR #83) absent from docs/src install pages"
short_code: "BROKKR-T-0278"
created_at: 2026-07-27T14:13:07.610371+00:00
updated_at: 2026-07-28T15:21:07.906966+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#tech-debt"
  - "#phase/completed"


exit_criteria_met: false
initiative_id: NULL
---

# Docs: helm existing-Secret credential sourcing (PR #83) absent from docs/src install pages

## Objective

Bring the existing-Secret credential flows from PR #83 into the mdbook install/ops pages. PR #83 documented them only in the chart READMEs; as of 2026-07-27 the only `existingSecret` hits in docs/src are unrelated ingress/TLS contexts (network-configuration.md, network-flows.md).

## Backlog Item Details

### Type
- [x] Tech Debt - Code improvement or refactoring (documentation debt)

### Priority
- [x] P1 - High (important for user experience)

### Technical Debt Impact
- **Current Problems**: The production-grade way to install Brokkr — credentials in pre-created Secrets instead of plaintext helm values rendered into ConfigMaps — is invisible to mdbook readers. The values shipped in PR #83 (verified against chart templates):
  - Broker: `postgresql.existingSecret` + `postgresql.existingSecretKey` (default `database-url`) → injects `BROKKR__DATABASE__URL`; `broker.pakHashExistingSecret` + `broker.pakHashExistingSecretKey` (default `BROKKR__BROKER__PAK_HASH`); `broker.webhookEncryptionKeyExistingSecret` + key (default `BROKKR__BROKER__WEBHOOK_ENCRYPTION_KEY`). When set, the value is omitted from the ConfigMap and injected via `secretKeyRef` (`charts/brokkr-broker/templates/configmap.yaml:16-18,63-67`, `deployment.yaml:59-82`).
  - Agent: `broker.existingSecret` + `broker.existingSecretKey` (default `BROKKR__AGENT__PAK`) → injects the agent PAK; ConfigMap renders it empty (`charts/brokkr-agent/templates/configmap.yaml:17-20`, `deployment.yaml:69-77`).
- **Benefits of Fixing**: docs/src installation and install-operations pages can show the secure default path (create Secrets → set existingSecret values), matching what the chart READMEs already say, instead of implicitly teaching plaintext-values installs.
- **Risk Assessment**: Consumers following only the mdbook will put the admin PAK hash, DB URL, and agent PAK in plaintext values files/ConfigMaps — precisely the exposure PR #83 exists to prevent.

### Content to write (suggested placement)
- getting-started/installation.md: secure install path with `kubectl create secret` + existingSecret values (mirroring chart README commands).
- how-to/install-operations.md: rotation/ops interplay (rotating a Secret vs `rotate admin`, restart requirements).
- how-to/security-hardening.md: promote existingSecret as the hardening default.
- Cross-check docs/src pages against chart READMEs so the two never contradict (chart READMEs are currently the more accurate source).

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

## Acceptance Criteria

- [ ] All six existingSecret-family values documented in docs/src with their default key names and ConfigMap-omission behavior.
- [ ] installation.md shows the Secret-based install as the recommended path.
- [ ] docs/src and chart READMEs agree on secret names/keys and commands.

## Status Updates

**2026-07-28 — COMPLETE** on branch `docs/tenancy-review-2026-07`. `angreal docs build` passes.

`installation.md` gained a "Production Install: Credentials from Kubernetes Secrets" section between Quick Start and Detailed Installation, an existing-Secret values reference covering all six values with default keys and the precedence/ConfigMap-omission rule, post-install verification that the credential is *absent* from the ConfigMap and *present* in the pod env, and a `CreateContainerConfigError` troubleshooting entry — which is precisely the failure mode this feature introduces. Plaintext paths demoted to dev/test with links rather than deleted.

`install-operations.md` gained two sections. **"Move an Existing Release onto Secrets"** documents the `--set <plaintext>=""` requirement, because `--reuse-values` otherwise carries the plaintext forward into the release's stored values. **"Rotating a Credential Held in a Secret"** records verified restart semantics rather than assumed ones: `secretKeyRef` and `envFrom` env vars resolve once at container start, neither chart watches Secrets, and neither puts a content hash on the *pod template* (the broker's `checksum/config` annotation is on the ConfigMap itself, not the pod spec), so `helm upgrade` will not roll pods on a Secret-content change — an explicit `kubectl rollout restart` is required.

**Subtle correctness finding worth keeping:** `rotate admin` reads `BROKKR__BROKER__PAK_HASH` from its *own process environment*, so running it before the pod restart silently re-applies the **old** hash. The documented order is therefore update Secret → `kubectl rollout restart` → exec `rotate admin`, plus the auth-cache TTL tail. Getting this backwards looks like a successful rotation that did nothing.

`security-hardening.md` gained "Keep Credentials Out of ConfigMaps", written to continue from the existing default-PAK section rather than bolt onto it, with three honest qualifications: Secrets are base64, not encrypted (this narrows the boundary, it does not eliminate it); the admin PAK hash is the least sensitive of the four since the PAK cannot be recovered from it; and the webhook key is also an *availability* measure, because an unset key means a random per-process key and, once any subscription exists, a broker that refuses to start.

Cross-check against the chart READMEs found **no contradictions** — key names, precedence, ConfigMap-omission, and the generate-pak → Secret → rotate sequence all agree. Example Secret *names* differ between sources, but the READMEs already vary their own placeholders, so they read as illustrative rather than normative.

**Defect found in a page outside this ticket's ownership, fixed here:** `how-to/pak-management.md` created an agent Secret with key `pak`, which does not match the agent chart's default `broker.existingSecretKey` of `BROKKR__AGENT__PAK`, and never mentioned `broker.existingSecret` — a reader wiring the two together would land in exactly the `CreateContainerConfigError` this ticket documents.