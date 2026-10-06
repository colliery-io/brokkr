# Rust SDK

The `brokkr-client` crate is generated from the broker's OpenAPI spec by `progenitor` at compile time. An ergonomic wrapper (`BrokkrClient`) sits on top to handle auth, retries, and typed errors.

## Install

```bash
cargo add brokkr-client tokio uuid --features tokio/macros,tokio/rt-multi-thread
```

Or by hand in `Cargo.toml`:

```toml
[dependencies]
brokkr-client = "X.Y"  # the version of your broker
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
uuid = "1"
```

The crate version matches the broker version. Use the release that has the same version as your broker. See the [releases page](https://github.com/colliery-io/brokkr/releases).

`uuid` is not required by the client itself, but resource IDs are UUIDs, so you will almost always want it.

For in-tree workspace consumers, swap the dependency for a path dep:

```toml
brokkr-client = { path = "../brokkr-client" }
```

## Construct a client

```rust
use brokkr_client::BrokkrClient;

let client = BrokkrClient::builder("https://broker.example.com") // the client adds /api/v1
    .token("brokkr_BRabcd1234_GeneratorLongTokenExample01") // generator PAK
    .build()?;
```

The constructor takes a base URL and one PAK. The base URL can be the broker root (`https://broker.example.com`) or end in `/api/v1`. The client adds `/api/v1` when it is absent and never adds it twice. The `brokkr` CLI uses the same rule. The wrapper attaches the PAK to every request as a bare `Authorization: <pak>` header, with no `Bearer` prefix. The broker accepts both forms, but expect the raw value when you inspect traffic through a proxy. You do not need to know which of the three `*_pak` security schemes your role maps to.

This page uses a **generator PAK**, because a generator owns stacks and deployment objects. An admin creates a generator with `brokkr-broker create generator --name <name>` or `POST /api/v1/generators`. Each operation needs one of these PAKs:

| Operation | PAK |
|-----------|-----|
| `apply`, `submit_manifests`, stack, label and deployment-object calls, telemetry history, `list_agents` | Generator (its own stacks only) or admin |
| `apply_for_generator`, WebSocket connection list, create agents and generators, stack health | Admin |
| Heartbeat, target state, agent events and health reports (see the [worked example](#worked-example-agent-heartbeat--fetch-target-state)) | Agent |

The `security` field of each operation in `openapi/brokkr-v1.json` gives the full list.

## Call one endpoint

The raw generated client is reachable via `client.api()`. Each operation is a builder; call `.send().await` to execute.

```rust
let response = client.api().list_agents().send().await?;
let agents = response.into_inner();
println!("{} agents", agents.len());
```

## Submit a folder of manifests

You usually have a *folder* of Kubernetes manifests, not a hand-built YAML blob. `submit_manifests` reads the folder (top-level `*.yaml`/`*.yml`, sorted), concatenates it into one multi-document stream, validates each document has `apiVersion`+`kind`, and submits it as a new deployment object on an existing stack:

```rust
let object = client.submit_manifests(stack_id, "./manifests").await?;
println!("submitted revision {}", object.sequence_id);
```

For the control-plane loop, `apply` is idempotent: it creates the stack by name if it doesn't exist, applies targeting labels for fan-out, and submits a new revision **only when the bundle changed** (so re-running with an unchanged folder is a no-op). With a generator PAK, the stack is owned by that generator:

```rust
use brokkr_client::ApplyOutcome;

match client.apply("payments", "./manifests", &["env:prod".into(), "region:us".into()]).await? {
    ApplyOutcome::Created(_) => println!("first revision"),
    ApplyOutcome::Updated(_) => println!("new revision submitted"),
    ApplyOutcome::Unchanged  => println!("already current"),
}
```

An admin PAK has no generator of its own, so it names the owner (by name or id) with `apply_for_generator`:

```rust
let outcome = admin.apply_for_generator("acme-payments", "payments", "./manifests", &[]).await?;
```

`apply` with an admin PAK returns `BrokkrError::InvalidRequest`; so does `apply_for_generator` with a generator PAK that names a different generator.

A stack's desired state is the single latest deployment object, and the agent reconciles + prunes — so removing a file from the folder and re-applying deletes that resource on the next reconcile. Ordering is forgiving: the agent front-loads `Namespace`/`CustomResourceDefinition` objects.

## Handle one error

Errors come back as `BrokkrError`. Match on `.code()` for the stable wire code:

```rust
use brokkr_client::BrokkrError;

match client.api().get_stack().id(stack_id).send().await {
    Ok(response) => println!("{:?}", response.into_inner()),
    Err(raw) => {
        let err = BrokkrError::from(raw);
        match err.code() {
            Some("stack_not_found") => eprintln!("no such stack"),
            Some("unauthorized") => eprintln!("PAK rejected"),
            _ => eprintln!("{err}"),
        }
    }
}
```

See [stable error codes](../../reference/error-codes.md) for the full list.

## Retry on transient failures

`BrokkrClient::retry` re-runs a closure with exponential backoff (200 ms, doubling, capped at 10 s; 3 retries by default, so 4 attempts in total). Transport errors and HTTP `408/429/502/503/504` retry; everything else returns immediately.

The closure receives the raw client, but the future it returns must not borrow from that argument — clone the client once up front and move a clone into each attempt instead:

```rust
use brokkr_client::BrokkrError;

let api = client.api().clone();

let response = client
    .retry(move |_| {
        let api = api.clone();
        async move { api.list_agents().send().await.map_err(BrokkrError::from) }
    })
    .await?;
```

Wrap only operations you consider safe to repeat — typically idempotent GETs.

## Worked example: agent heartbeat + fetch target state

Brokkr agents do two things on every tick: send a heartbeat, then fetch their target state. These calls need an **agent PAK**. Here is the same pattern, condensed:

```rust
use brokkr_client::{BrokkrClient, BrokkrError};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // e.g. BROKKR_BROKER_URL=https://broker.example.com (the client adds /api/v1)
    let client = BrokkrClient::builder(std::env::var("BROKKR_BROKER_URL")?)
        .token(std::env::var("BROKKR_AGENT_PAK")?)
        .build()?;

    let agent_id: Uuid = std::env::var("BROKKR_AGENT_ID")?.parse()?;

    // 1. Heartbeat.
    client.api().record_heartbeat().id(agent_id).send().await
        .map_err(BrokkrError::from)?;

    // 2. Fetch target state and print one summary line.
    //    get_target_state returns a Vec<DeploymentObject>.
    let state = client.api().get_target_state().id(agent_id).send().await
        .map_err(BrokkrError::from)?
        .into_inner();

    println!("target state: {} deployment objects", state.len());
    Ok(())
}
```

The real Brokkr agent layers on metrics, logging, and `401 → "rotate PAK"` handling, but the call shape is the same.

## When you need to drop to the raw client

The wrapper intentionally does not expose every endpoint with hand-written methods — it would defeat the point of generation. For anything the wrapper doesn't cover, `client.api()` gives you the full progenitor surface; the auth header is still injected.
