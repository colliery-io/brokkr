/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Internal broker↔agent WebSocket channel.
//!
//! Wire protocol lives in [`brokkr_wire`]. This module owns the **broker side**
//! of the connection: the upgrade endpoint, PAK auth, the per-agent connection
//! registry, and the priority-aware writer that ensures control-plane messages
//! are never starved by log/event traffic.
//!
//! This surface is **internal-only** — it is not part of the public OpenAPI
//! spec and is not exposed via the generated SDKs. See [[BROKKR-A-0008]] and
//! [[BROKKR-I-0019]] in `.metis/`.

pub mod handler;
pub mod registry;

pub use handler::{internal_routes, INTERNAL_WS_PATH};
pub use registry::{ConnectionHandle, ConnectionInfo, ConnectionRegistry, SendError};
