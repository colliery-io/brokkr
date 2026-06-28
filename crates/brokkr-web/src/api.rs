//! Same-origin REST client to the broker. The console is served by the broker,
//! so the API is at `/api/v1/...`. Auth: the broker requires a PAK; until the
//! console's read-access auth model is decided (ADR-0010, deferred), we read a
//! PAK the operator pastes into `localStorage["brokkr_pak"]` and send it as a
//! Bearer token. Errors map to Aurora's `ApiError` for `ErrorState`.

use crate::models::{ErrorBody, FleetAgentRecord};
use aurora_leptos::tokens::ApiError;
use gloo_net::http::Request;
use serde::de::DeserializeOwned;

/// Operator-pasted PAK, if any (interim auth — see module docs).
pub fn pak() -> Option<String> {
    let ls = web_sys::window()?.local_storage().ok()??;
    match ls.get_item("brokkr_pak").ok()? {
        Some(s) if !s.is_empty() => Some(s),
        _ => None,
    }
}

/// GET `/api/v1{path}` and deserialize the JSON body.
pub async fn get<T: DeserializeOwned>(path: &str) -> Result<T, ApiError> {
    let url = format!("/api/v1{path}");
    let mut req = Request::get(&url);
    if let Some(p) = pak() {
        req = req.header("Authorization", &format!("Bearer {p}"));
    }
    let resp = req.send().await.map_err(|_| ApiError::Network)?;
    let status = resp.status();
    if !(200..300).contains(&status) {
        let message = resp.text().await.unwrap_or_default();
        let code = serde_json::from_str::<ErrorBody>(&message)
            .ok()
            .map(|b| b.code);
        return Err(ApiError::Http {
            status,
            message,
            code,
        });
    }
    resp.json::<T>().await.map_err(|e| ApiError::Http {
        status,
        message: e.to_string(),
        code: None,
    })
}

/// `GET /api/v1/fleet` — the fleet rollup (flat list of agents).
pub async fn fleet() -> Result<Vec<FleetAgentRecord>, ApiError> {
    get("/fleet").await
}
