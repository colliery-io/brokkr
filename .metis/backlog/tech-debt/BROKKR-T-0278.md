---
id: docs-helm-existing-secret
level: task
title: "Docs: helm existing-Secret credential sourcing (PR #83) absent from docs/src install pages"
short_code: "BROKKR-T-0278"
created_at: 2026-07-27T14:13:07.610371+00:00
updated_at: 2026-07-27T14:13:07.610371+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#tech-debt"


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

- [ ] All six existingSecret-family values documented in docs/src with their default key names and ConfigMap-omission behavior.
- [ ] installation.md shows the Secret-based install as the recommended path.
- [ ] docs/src and chart READMEs agree on secret names/keys and commands.

## Status Updates

*To be added during implementation*
