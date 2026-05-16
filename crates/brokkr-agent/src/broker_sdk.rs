/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Adapter that constructs the `brokkr-client` ergonomic wrapper from the
//! agent's `Settings`.
//!
//! The agent migration to `brokkr-client` happens incrementally — this module
//! is the single place that knows how to translate the agent's configuration
//! (broker URL + PAK + retry knobs) into a configured `BrokkrClient`. Per-call
//! sites in `broker.rs`, `webhooks.rs`, and `work_orders/broker.rs` consume
//! the constructed client.

use std::time::Duration;

use brokkr_client::{BrokkrClient, BrokkrError};
use brokkr_utils::Settings;

/// Bearer-token form expected by the broker's auth middleware. The wrapper
/// will inject this verbatim into the `Authorization` header on every
/// request.
fn bearer_token(pak: &str) -> String {
    format!("Bearer {pak}")
}

/// Build a `BrokkrClient` from agent `Settings`. Returns the wrapper's typed
/// `BrokkrError` if header construction or transport setup fails.
///
/// The agent's configured `broker_url` is the root of the broker service
/// (e.g. `http://broker:3000`). The v1 API lives under `/api/v1`; we append
/// it here so generated operation paths (which are resource-relative after
/// the T-A3 path cleanup) resolve correctly.
pub fn build_client(config: &Settings) -> Result<BrokkrClient, BrokkrError> {
    let base_url = format!("{}/api/v1", config.agent.broker_url);
    BrokkrClient::builder(base_url)
        .token(bearer_token(&config.agent.pak))
        .request_timeout(Duration::from_secs(30))
        .connect_timeout(Duration::from_secs(10))
        .max_retries(config.agent.max_retries)
        .build()
}
