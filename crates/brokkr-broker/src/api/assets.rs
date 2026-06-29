/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Static web-UI serving + SPA fallback for the operator console
//! (BROKKR-I-0031, BROKKR-T-0253; mirrors Skadi's `skadi-api`).
//!
//! The Leptos front-end (`crates/brokkr-web`) is built by `trunk` into `dist/`
//! and baked into this binary via `rust-embed` when the **`embed-ui`** feature is
//! on. [`attach`] mounts a fallback on the **outer** router (outside the
//! `/api/v1` nest, where auth lives), so:
//!
//!   * real API routes always win — they match before the fallback runs;
//!   * any other GET serves the matching embedded asset, or `index.html` for
//!     unknown client-side routes (the SPA fallback);
//!   * `/api/*` and `/internal/*` that fall through are honest 404s, never the SPA.
//!
//! With `embed-ui` **off** (the default for the host workspace / tests), a small
//! placeholder is served instead and the binary needs no `dist/` to compile.

use axum::Router;
use axum::http::{StatusCode, Uri, header};
use axum::response::{IntoResponse, Response};

/// Mount the static/SPA fallback on the outer app router. Call this LAST, after
/// the `/api/v1` nest and the other route groups are in place, so API routes take
/// precedence. Generic over the router state — the fallback extracts none.
pub fn attach<S>(app: Router<S>) -> Router<S>
where
    S: Clone + Send + Sync + 'static,
{
    app.fallback(static_handler)
}

/// Paths the API owns; these must 404 rather than fall back to the SPA shell.
fn is_api_path(path: &str) -> bool {
    path == "/api"
        || path.starts_with("/api/")
        || path == "/internal"
        || path.starts_with("/internal/")
}

async fn static_handler(uri: Uri) -> Response {
    let path = uri.path();
    if is_api_path(path) {
        return (StatusCode::NOT_FOUND, "not found").into_response();
    }
    serve_asset(path.trim_start_matches('/'))
}

#[cfg(feature = "embed-ui")]
mod embedded {
    use rust_embed::RustEmbed;

    /// The `trunk build` output, baked in at compile time.
    #[derive(RustEmbed)]
    #[folder = "../brokkr-web/dist"]
    pub struct Assets;
}

/// Serve `path` from the embedded bundle, falling back to `index.html` for
/// unknown (client-side) routes.
#[cfg(feature = "embed-ui")]
fn serve_asset(path: &str) -> Response {
    let lookup = if path.is_empty() { "index.html" } else { path };

    if let Some(file) = embedded::Assets::get(lookup) {
        let mime = mime_guess::from_path(lookup).first_or_octet_stream();
        return (
            [(header::CONTENT_TYPE, mime.as_ref().to_string())],
            file.data.into_owned(),
        )
            .into_response();
    }

    // Unknown path → hand the SPA its shell so client-side routing can resolve.
    match embedded::Assets::get("index.html") {
        Some(index) => (
            [(header::CONTENT_TYPE, "text/html")],
            index.data.into_owned(),
        )
            .into_response(),
        None => (StatusCode::NOT_FOUND, "ui not built").into_response(),
    }
}

/// Placeholder served when the binary was built without `embed-ui`: the broker
/// and its API work fully; only the bundled console is absent.
#[cfg(not(feature = "embed-ui"))]
fn serve_asset(_path: &str) -> Response {
    const PLACEHOLDER: &str = concat!(
        "<!doctype html><meta charset=utf-8><title>Brokkr</title>",
        "<body style=\"font-family:system-ui;background:#0e1116;color:#e6e9ee;",
        "padding:2rem\"><h1>Brokkr</h1><p>The broker API is running. This build ",
        "was compiled without the bundled operator console (the ",
        "<code>embed-ui</code> feature). The JSON API is available under ",
        "<code>/api/v1</code>.</p>"
    );
    ([(header::CONTENT_TYPE, "text/html")], PLACEHOLDER).into_response()
}
