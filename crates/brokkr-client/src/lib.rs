/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Auto-generated Rust client for the Brokkr broker API.
//!
//! Every type and method in this crate is generated at compile time by the
//! `progenitor::generate_api!` macro from `spec/brokkr-v1.json`. That file
//! is kept byte-identical to the workspace-canonical `openapi/brokkr-v1.json`
//! by `angreal openapi export` and asserted by `angreal openapi check` —
//! the in-crate copy exists so the spec ships with the published crate.
//! Do not edit the generated surface; regenerate the spec via
//! `angreal openapi export` after broker changes.
//!
//! This crate is intentionally a thin wire-level client. The ergonomic
//! wrapper (auth, retries, pagination, typed errors) lives in a separate
//! layer added by task BROKKR-T-0137 (C1).

progenitor::generate_api!(spec = "spec/brokkr-v1.json", interface = Builder,);

mod wrapper;
pub use wrapper::{ApplyOutcome, BrokkrClient, BrokkrClientBuilder, BrokkrError};
