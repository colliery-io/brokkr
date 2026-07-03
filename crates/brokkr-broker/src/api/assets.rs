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

    if lookup == "index.html" {
        return serve_index();
    }

    if let Some(file) = embedded::Assets::get(lookup) {
        let mime = mime_guess::from_path(lookup).first_or_octet_stream();
        return (
            [(header::CONTENT_TYPE, mime.as_ref().to_string())],
            file.data.into_owned(),
        )
            .into_response();
    }

    // Unknown path → hand the SPA its shell so client-side routing can resolve.
    serve_index()
}

/// Serve the SPA shell with the ephemeral UI PAK injected (BROKKR-T-0268).
///
/// The token is minted per broker process (see `utils::ui_pak`), so the HTML
/// must not outlive it in any cache — hence `Cache-Control: no-store`. Other
/// assets are content-hashed by trunk and stay cacheable.
#[cfg(feature = "embed-ui")]
fn serve_index() -> Response {
    let Some(index) = embedded::Assets::get("index.html") else {
        return (StatusCode::NOT_FOUND, "ui not built").into_response();
    };
    let html = inject_ui_token(
        String::from_utf8_lossy(&index.data).into_owned(),
        crate::utils::ui_pak::token(),
    );
    (
        [
            (header::CONTENT_TYPE, "text/html".to_string()),
            (header::CACHE_CONTROL, "no-store".to_string()),
        ],
        html,
    )
        .into_response()
}

/// Inject the ephemeral UI PAK as a `<meta>` tag before `</head>` so the WASM
/// console can authenticate with zero configuration (BROKKR-T-0268). `None`
/// (UI PAK not minted — e.g. a broker embedding the UI but started via a
/// code path that skips `ui_pak::init`) leaves the document untouched.
#[cfg(any(feature = "embed-ui", test))]
fn inject_ui_token(html: String, token: Option<&str>) -> String {
    match token {
        // PAKs are `<prefix>_<short>_<long>` over [A-Za-z0-9_], so the token
        // needs no HTML-attribute escaping.
        Some(token) => html.replacen(
            "</head>",
            &format!("<meta name=\"brokkr-ui-token\" content=\"{token}\"></head>"),
            1,
        ),
        None => html,
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

#[cfg(test)]
mod tests {
    use super::inject_ui_token;

    #[test]
    fn injects_meta_tag_before_head_close() {
        let html = "<html><head><title>x</title></head><body></body></html>".to_string();
        let out = inject_ui_token(html, Some("brokkr_abc_def"));
        assert_eq!(
            out,
            "<html><head><title>x</title>\
             <meta name=\"brokkr-ui-token\" content=\"brokkr_abc_def\">\
             </head><body></body></html>"
        );
    }

    #[test]
    fn injects_only_once() {
        let html = "<head></head><head></head>".to_string();
        let out = inject_ui_token(html, Some("t"));
        assert_eq!(out.matches("brokkr-ui-token").count(), 1);
    }

    #[test]
    fn no_token_leaves_document_untouched() {
        let html = "<html><head></head></html>".to_string();
        assert_eq!(inject_ui_token(html.clone(), None), html);
    }
}
