---
id: rotate-admin-reports-success-when
level: task
title: "rotate admin reports success when it mints nothing, and hides the PAK it does mint in a file nothing reads"
short_code: "BROKKR-T-0317"
created_at: 2026-07-29T05:00:00+00:00
updated_at: 2026-07-29T04:56:02.128348+00:00
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

# rotate admin reports success when it mints nothing, and hides the PAK it does mint in a file nothing reads

## Objective

Make `brokkr-broker rotate admin` tell the truth about what it did, and surface the credential it creates the way every sibling command already does.

`upsert_admin` has two branches and `rotate admin` could not tell them apart, because both returned `()`:

1. **`broker.pak_hash` is set** — the hash is validated and re-applied verbatim. **No credential is minted.** The command logged `Admin key rotated successfully` anyway.
2. **`broker.pak_hash` is unset/empty** — a PAK is minted and written to `/tmp/brokkr-keys/key.txt`. Nothing was printed; the hash was never surfaced at all.

Branch 1 is the common case. The Helm chart always sets `broker.pakHash`, so an operator who execs into a broker pod and runs `rotate admin` — a reasonable cold-start instinct — gets a success message, no new credential, and no indication that the publicly-known default may still be live.

## Backlog Item Details

### Type
- [x] Bug - Production issue that needs fixing

### Priority
- [x] P2 - Medium (no data loss; it misreports the state of the system's strongest credential during exactly the operation meant to secure it)

## 2026-07-29 — grounding

**`rotate admin` was the only credential-minting command in the CLI that printed nothing.** Every sibling already prints:

| Command | Prints |
|---|---|
| `generate-pak` | PAK **and** hash, with the day-zero flow spelled out |
| `rotate agent` | `New PAK: <pak>` |
| `rotate generator` | `New PAK: <pak>` |
| `rotate admin` | **nothing** — file only |

So this is an inconsistency, not a deliberate security posture. The file is not safer either: `fs::write` leaves it mode 0644, readable by anything in the pod.

**`/tmp/brokkr-keys/key.txt` has no reader anywhere in the repo.** Checked across Rust, Python, YAML and shell: the constant is written in `utils/mod.rs` and deleted in `shutdown`, and the only other use of that directory is `kubeconfig.yaml` in the agent's k8s tests. The dev environment authenticates with the configured default hash, not the file. It is a write-only artifact whose sole consumer is a human who knows to `cat` it before graceful shutdown removes it.

**Admin needs the hash printed, unlike agent/generator.** Nothing configures an agent or generator by hash, so those commands only need to emit the PAK. The admin hash must land in `BROKKR__BROKER__PAK_HASH` / the chart's `broker.pakHash` / `broker.pakHashExistingSecret`. Emitting only the PAK strands the operator: the hash is `long_token_hashed` (SHA-256 of the PAK's long-token component only), so it cannot be reproduced with a plain `sha256sum` on the PAK string, and config and database then disagree with no way to reconcile them.

That divergence is not theoretical — it is exactly what `DefaultAdminPakStatus` tracks `configured` and `stored` separately for.

## Acceptance Criteria

## Acceptance Criteria

- [x] `rotate admin` distinguishes the two branches and never claims to have rotated when it minted nothing.
- [x] The no-op branch states plainly what it did, that nothing was revoked, and both routes to actually replacing the credential.
- [x] The minting branch prints the PAK **and** the hash, noting the PAK is shown once.
- [x] `serve`'s first-startup behaviour is unchanged — it must not print secrets into a long-running process's log stream.
- [x] The key file is still written, so nothing depending on it breaks.
- [x] Tests pin the branch distinction rather than the wording.
- [x] `how-to/pak-management.md` reflects the new output, and documents the exec-into-the-pod cold start it previously omitted.

## Status Updates

**2026-07-29 — DONE** on branch `docs/tenancy-review-2026-07` (commit `73bac4e`). 531 integration tests pass (2 new), 147 unit, `angreal docs build` clean.

`upsert_admin` now returns `AdminPakOutcome::{Minted { pak, hash }, ReappliedConfigured { hash }}` instead of `()`. That is the whole fix: the information existed inside the function and was thrown away at the boundary.

**`serve` is deliberately unchanged.** `first_startup` maps the outcome to `()` and keeps the key file as its channel. `serve` is a long-running process whose stdout is a log stream — a secret printed there would sit in `kubectl logs` for the pod's lifetime. The file is the right channel for that path even though it is the wrong one for an interactive CLI.

**The no-op branch is not an error.** Re-applying a configured hash is the supported way to commit a hash you minted yourself, which is the documented flow. It was only ever misleading, so the fix is the message, not the exit code.

Two integration tests pin the branches (`tests/integration/db/default_admin_pak.rs`), asserting the outcome variant rather than the printed strings so wording can change freely. The minting test also asserts the returned PAK is not the hash and that the stored hash is the minted one — i.e. the printed credential actually authenticates. Printing the hash as though it were the PAK would hand out a string that silently fails to authenticate, which is the obvious way to get this wrong.
**Docs corrected, including two claims that this change falsified:** `reference/cli.md` said the minted PAK "is never printed to stdout" and that "the previously stored hash is replaced, so the old admin PAK stops working" — the latter was already misleading before this change, since nothing is replaced or revoked on the re-apply branch. Both fixed. `how-to/pak-management.md` gained the Kubernetes cold start, which was genuinely undocumented: the page's only `kubectl` material was for *agent* Secrets.

Checked the rest of the tree for stale `key.txt` claims. The remaining references (`multi-tenant-setup.md`, `sdks/README.md`, `configuration.md`, `development.md`, `installation.md`) all describe `serve`'s first-startup path, which is unchanged and still file-only — they stay accurate.
