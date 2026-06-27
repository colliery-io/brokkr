# Submitting a Folder of Manifests (CLI)

You have a folder of Kubernetes manifests and want it to become a stack's desired state — from your shell or a CI job, without writing any code. The `brokkr` CLI does this with one idempotent command.

## Prerequisites

- The `brokkr` binary on your `PATH` (download the Linux or macOS tarball for your architecture from the [GitHub Release](https://github.com/colliery-io/brokkr/releases), or build it from source with `cargo build --release -p brokkr-cli`).
- A reachable broker and a **generator** PAK (the stack will be owned by that generator).

## Configure the connection once

Create `~/.brokkr/config` so you don't repeat the broker URL and PAK on every call:

```toml
broker_url = "https://broker.example.com"
pak = "brokkr_BRabcd1234_GeneratorTokenExample0001"
```

The `/api/v1` suffix is added for you. Any value here can be overridden per-invocation with `--broker-url` / `--pak`, or supplied instead via `BROKKR_BROKER_URL` / `BROKKR_PAK` (handy in CI, where the PAK comes from a secret). Precedence is flag → environment → config file.

## Apply the folder

```bash
brokkr apply -f ./manifests --stack payments
```

This reads the top-level `*.yaml`/`*.yml` files in `./manifests` (sorted), concatenates them into one multi-document stream, validates that each document has `apiVersion` and `kind`, and submits the bundle as the stack's latest deployment object — creating the stack `payments` if it doesn't exist. You'll see one of:

```
created stack "payments": first revision (sequence 1)
updated stack "payments": new revision (sequence 2)
unchanged: stack "payments" already current
```

## Target specific agents

Add one or more labels so only matching agents reconcile the stack:

```bash
brokkr apply -f ./manifests --stack payments \
  --target-label env:prod \
  --target-label region:us
```

Labels are additive and applied every run; a label that already exists is left as-is.

`--target-label` is **label-based fan-out**: any agent whose labels match reconciles the stack, and label/annotation matching does **not** consult generator registration. Registration is enforced separately, only when an admin creates an *explicit* per-agent target (`POST /agents/{id}/targets`): the agent must be registered with the stack's owning generator or the request is rejected with HTTP `403` / `agent_not_registered`. So `brokkr apply --target-label` is unaffected by registration; if you instead bind a specific agent explicitly, register it first with `brokkr register --agent <id> --generator <id>` (admin PAK) or by setting `BROKKR__AGENT__GENERATOR_IDS` on the agent at startup. See [Registering agents with generators](./agent-registration.md) and the [error code reference](../reference/error-codes.md).

## Re-run safely in CI

`apply` is idempotent — it compares a checksum of the bundle against the stack's current latest deployment object and submits a new revision **only when the folder changed**. That makes it safe to run on every push:

```bash
BROKKR_BROKER_URL="$BROKER_URL" BROKKR_PAK="$BROKKR_GENERATOR_PAK" \
  brokkr apply -f ./k8s --stack "$SERVICE_NAME" --target-label env:prod
```

A successful run exits `0` (including the `unchanged` case); any failure prints `error: …` to stderr and exits `1`, so a failed apply fails the job.

## Remove a resource

A stack's desired state is the single latest bundle, and the agent prunes anything no longer present. To delete a resource, remove its file from the folder and re-apply — the next reconcile deletes it from the cluster. Object ordering within the folder doesn't matter: the agent front-loads `Namespace` and `CustomResourceDefinition` objects.

## See also

- [CLI Reference](../reference/cli.md) — every flag, config key, and exit code.
- [Rust SDK](./sdks/rust.md) / [Python SDK](./sdks/python.md) / [TypeScript SDK](./sdks/typescript.md) — the same `apply` semantics, in code.
