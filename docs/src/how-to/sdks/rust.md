# Rust SDK

The `brokkr-client` crate is generated from `openapi/brokkr-v1.json` by `progenitor` at compile time. An ergonomic wrapper (`BrokkrClient`) sits on top to handle auth, retries, and typed errors.

## Install

The crate lives in this workspace. Depend on it from a sibling crate:

```toml
[dependencies]
brokkr-client = { path = "../brokkr-client" }
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

It is not yet published to crates.io.

## Construct a client

```rust
use brokkr_client::BrokkrClient;

let client = BrokkrClient::builder("https://broker.example.com")
    .token("brokkr_BA...")  // agent PAK
    .build()?;
```

The constructor takes a base URL and one PAK. The wrapper attaches `Authorization: Bearer <pak>` on every request — you do not need to know which of the three `*_pak` security schemes your role maps to.

## Call one endpoint

The raw generated client is reachable via `client.api()`. Each operation is a builder; call `.send().await` to execute.

```rust
let response = client.api().list_agents().send().await?;
let agents = response.into_inner();
println!("{} agents", agents.len());
```

## Handle one error

Errors come back as `BrokkrError`. Match on `.code()` for the stable wire code:

```rust
use brokkr_client::BrokkrError;

match client.api().get_agent().id(agent_id).send().await {
    Ok(response) => println!("{:?}", response.into_inner()),
    Err(raw) => {
        let err = BrokkrError::from(raw);
        match err.code() {
            Some("agent_not_found") => eprintln!("no such agent"),
            Some("unauthorized") => eprintln!("PAK rejected"),
            _ => eprintln!("{err}"),
        }
    }
}
```

See [stable error codes](./errors.md) for the full list.

## Retry on transient failures

`BrokkrClient::retry` re-runs a closure with exponential backoff (200 ms, doubling, capped at 10 s; 3 attempts by default). Transport errors and HTTP `408/429/502/503/504` retry; everything else returns immediately.

```rust
let response = client
    .retry(|api| Box::pin(async move {
        api.list_agents().send().await
    }))
    .await?;
```

Wrap only operations you consider safe to repeat — typically idempotent GETs.

## Worked example: agent heartbeat + fetch target state

Brokkr agents do two things on every tick: send a heartbeat, then fetch their target state. Here is the same pattern, condensed:

```rust
use brokkr_client::{BrokkrClient, BrokkrError};
use uuid::Uuid;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = BrokkrClient::builder(std::env::var("BROKKR_BROKER_URL")?)
        .token(std::env::var("BROKKR_AGENT_PAK")?)
        .build()?;

    let agent_id: Uuid = std::env::var("BROKKR_AGENT_ID")?.parse()?;

    // 1. Heartbeat.
    client.api().record_heartbeat().id(agent_id).send().await
        .map_err(BrokkrError::from)?;

    // 2. Fetch target state and print one summary line.
    let state = client.api().get_target_state().id(agent_id).send().await
        .map_err(BrokkrError::from)?
        .into_inner();

    println!("target state: {} deployment objects", state.deployment_objects.len());
    Ok(())
}
```

The real agent (`crates/brokkr-agent/src/broker.rs`) layers on metrics, logging, and `401 → "rotate PAK"` handling, but the call shape is the same.

## When you need to drop to the raw client

The wrapper intentionally does not expose every endpoint with hand-written methods — it would defeat the point of generation. For anything the wrapper doesn't cover, `client.api()` gives you the full progenitor surface; the auth header is still injected.
