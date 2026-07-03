/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Ephemeral read-only UI PAK (BROKKR-T-0267).
//!
//! The broker mints one random PAK per process at startup and holds it in
//! memory only — it is never persisted, never written to the database, and
//! never logged. The served console HTML embeds the raw token (BROKKR-T-0268)
//! so the operator console authenticates with zero configuration; the auth
//! middleware recognizes its hash as a readonly admin credential.
//!
//! Scope: the token is visible to anyone who can fetch the console URL. That
//! is by design — network access is the auth boundary for the read-only
//! console (BROKKR-I-0032 non-goal), and the credential cannot mutate state.
//!
//! Multi-replica note: each replica mints its own UI PAK. A token injected by
//! one replica is rejected by another, so a load-balanced console needs sticky
//! sessions. Acceptable for v1 — the console is a single-broker ops tool.

use crate::utils::pak;
use std::sync::OnceLock;

struct UiPak {
    /// The raw PAK, retained for HTML injection at serve time.
    token: String,
    /// The PAK's hash, compared against incoming credentials by the auth
    /// middleware.
    hash: String,
}

static UI_PAK: OnceLock<UiPak> = OnceLock::new();

/// Mint the process-lifetime UI PAK. Idempotent: subsequent calls keep the
/// first key. Requires the PAK controller to be initialized.
pub fn init() -> Result<(), Box<dyn std::error::Error>> {
    if UI_PAK.get().is_some() {
        return Ok(());
    }
    let (token, hash) = pak::create_pak()?;
    let _ = UI_PAK.set(UiPak { token, hash });
    Ok(())
}

/// The raw UI PAK for injection into the served console HTML. `None` until
/// [`init`] has run (e.g. a broker built without the console flow).
pub fn token() -> Option<&'static str> {
    UI_PAK.get().map(|p| p.token.as_str())
}

/// The UI PAK's hash, for the auth middleware's in-memory comparison.
pub fn hash() -> Option<&'static str> {
    UI_PAK.get().map(|p| p.hash.as_str())
}
