---
id: helm-source-sensitive-credentials
level: task
title: "Helm: source sensitive credentials from existing Secrets (agent PAK + broker webhook key/PAK hash)"
short_code: "BROKKR-T-0266"
created_at: 2026-06-29T19:18:16.961285+00:00
updated_at: 2026-06-29T19:18:16.961285+00:00
parent: 
blocked_by: []
archived: false

tags:
  - "#task"
  - "#phase/backlog"
  - "#feature"


exit_criteria_met: false
initiative_id: NULL
---

# Helm: source sensitive credentials from existing Secrets (agent PAK + broker webhook key/PAK hash)

## Objective

Let operators source sensitive Helm credentials from a pre-existing Kubernetes Secret instead of baking them into a ConfigMap or committing them to values/git. Originated from operator feedback on the agent chart (GitOps / external-secrets-operator workflows), then extended to the broker.

Ships as a **template-only, backward-compatible change** — `appVersion` stays `0.8.3` (same container images), so it can be delivered as an out-of-band chart re-deploy/overwrite, **not** a full lockstep release. See [[project_release_versioning]].

## Backlog Item Details

### Type
- [x] Feature - New functionality or enhancement

### Priority
- [x] P1 - High (security/GitOps gap; sensitive creds were only available as ConfigMap plaintext)

### Business Justification
- **User Value**: PAK / webhook encryption key / admin PAK hash can be vended by external-secrets-operator (Vault / 1Password / AWS Secrets Manager) into a k8s Secret; the raw credential never touches a values file or git history.
- **Effort Estimate**: S

## Problem

- **Agent**: `BROKKR__AGENT__PAK` was always rendered into the ConfigMap from `broker.pak`. The README documented an `extraEnv`-based Secret workaround, but `extraEnv` was **never wired into the agent deployment template** — so there was genuinely no working way to source the PAK from a Secret.
- **Broker**: `BROKKR__BROKER__WEBHOOK_ENCRYPTION_KEY` and `BROKKR__BROKER__PAK_HASH` were rendered as ConfigMap plaintext (DB URL already had `postgresql.existingSecret`).

## Acceptance Criteria

- [x] Agent: `broker.existingSecret` + `broker.existingSecretKey` (default `BROKKR__AGENT__PAK`) added; PAK injected via `secretKeyRef`; ConfigMap emits empty PAK when set.
- [x] Broker: `broker.webhookEncryptionKeyExistingSecret`/`...Key` and `broker.pakHashExistingSecret`/`...Key` added; injected via `secretKeyRef`; plaintext keys omitted from ConfigMap when the existingSecret variant is set.
- [x] Default behavior unchanged when no existingSecret is set (backward compatible).
- [x] No plaintext credential leaks into the ConfigMap when an existingSecret is configured.
- [x] READMEs updated; the broken agent `extraEnv` workaround replaced with the first-class option.
- [x] `helm lint` clean and `helm template` verified for both charts (default + existingSecret paths, incl. agent existingSecret + namespace-scoped RBAC combined).

## Implementation Notes

### Technical Approach
Used `env:` + `secretKeyRef` rather than the originally-proposed `envFrom:` + `secretRef`:
- Makes `existingSecretKey` **functional** — `envFrom: secretRef` imports the whole Secret and ignores the key (the key would have to always equal the env var name).
- Least-privilege: imports only the one key.
- Kubernetes applies `env` after `envFrom`, so the Secret value deterministically overrides the ConfigMap value.

Agent deployment merges the new PAK entry with the existing namespace-scoped-RBAC `env` block (`WATCH_NAMESPACE`); the block now renders when either condition holds.

### Files changed
- `charts/brokkr-agent/`: `values.yaml`, `templates/configmap.yaml`, `templates/deployment.yaml`, `README.md`
- `charts/brokkr-broker/`: `values.yaml`, `templates/configmap.yaml`, `templates/deployment.yaml`, `README.md`

### Release / rollout
Out-of-band chart re-package + push (manual), reusing 0.8.3 app images. The release pipeline (`release.yml` `publish-helm-charts`) packages charts with `--version`/`--app-version` = git tag, so this intentionally bypasses the normal lockstep tag path.

## Status Updates

- 2026-06-29: Implemented across both charts; `helm lint` clean, `helm template` verified for default + existingSecret paths (agent, agent+namespaced RBAC, broker both secrets) with no plaintext ConfigMap leak. Pending: commit + manual out-of-band chart publish.