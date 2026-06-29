//! Same-origin REST client to the broker. The console is served by the broker,
//! so the API is at `/api/v1/...`. Auth: the broker requires a PAK; until the
//! console's read-access auth model is decided (ADR-0010, deferred), we read a
//! PAK the operator pastes into `localStorage["brokkr_pak"]` and send it as a
//! Bearer token. Errors map to Aurora's `ApiError` for `ErrorState`.

use crate::models::{ErrorBody, FleetAgentRecord};
use aurora_leptos::tokens::ApiError;
use gloo_net::http::Request;
use serde::Serialize;
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

/// `GET /metrics` (Prometheus text; top-level, public — no `/api/v1` prefix).
pub async fn metrics_text() -> Result<String, ApiError> {
    let resp = Request::get("/metrics")
        .send()
        .await
        .map_err(|_| ApiError::Network)?;
    let status = resp.status();
    if !(200..300).contains(&status) {
        return Err(ApiError::Http {
            status,
            message: resp.text().await.unwrap_or_default(),
            code: None,
        });
    }
    resp.text().await.map_err(|_| ApiError::Network)
}

/// `GET /api/v1/admin/ws/connections`.
pub async fn ws_connections() -> Result<crate::models::WsConnectionsResponse, ApiError> {
    get("/admin/ws/connections").await
}

/// Sum all samples of a Prometheus metric `name` (handles labeled counters).
pub fn metric_sum(text: &str, name: &str) -> Option<f64> {
    let mut total = 0.0;
    let mut found = false;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with('#') || !line.starts_with(name) {
            continue;
        }
        let rest = &line[name.len()..];
        // boundary: the metric name is followed by a space or a `{labels}` block.
        if !(rest.starts_with(' ') || rest.starts_with('{')) {
            continue;
        }
        if let Some(val) = rest.split_whitespace().last() {
            if let Ok(v) = val.parse::<f64>() {
                total += v;
                found = true;
            }
        }
    }
    found.then_some(total)
}

/// `GET /api/v1/webhooks` — subscription summaries.
pub async fn webhooks() -> Result<Vec<crate::models::WebhookSummary>, ApiError> {
    get("/webhooks").await
}

/// `GET /api/v1/work-order-log` — completed work-order history.
pub async fn work_order_log() -> Result<Vec<crate::models::WorkOrderLogEntry>, ApiError> {
    get("/work-order-log").await
}

/// `GET /api/v1/stacks`.
pub async fn stacks() -> Result<Vec<crate::models::Stack>, ApiError> {
    get("/stacks").await
}

/// `GET /api/v1/agent-events`.
pub async fn agent_events() -> Result<Vec<crate::models::AgentEventDto>, ApiError> {
    get("/agent-events").await
}

/// POST `/api/v1{path}` with a JSON body; discards the response on success.
pub async fn post<B: Serialize>(path: &str, body: &B) -> Result<(), ApiError> {
    let url = format!("/api/v1{path}");
    let mut req = Request::post(&url);
    if let Some(p) = pak() {
        req = req.header("Authorization", &format!("Bearer {p}"));
    }
    let resp = req
        .json(body)
        .map_err(|_| ApiError::Network)?
        .send()
        .await
        .map_err(|_| ApiError::Network)?;
    let status = resp.status();
    if !(200..300).contains(&status) {
        let message = resp.text().await.unwrap_or_default();
        let code = serde_json::from_str::<ErrorBody>(&message).ok().map(|b| b.code);
        return Err(ApiError::Http { status, message, code });
    }
    Ok(())
}

/// `POST /api/v1/diagnostics` — request a diagnostic for an agent (the v1 write).
pub async fn create_diagnostic(agent_id: &str) -> Result<(), ApiError> {
    post("/diagnostics", &serde_json::json!({ "agent_id": agent_id })).await
}

/// `GET /api/v1/stacks/:id/health` — per-stack deployment-object health rollup.
pub async fn stack_health(id: &str) -> Result<crate::models::StackHealth, ApiError> {
    get(&format!("/stacks/{id}/health")).await
}

/// `GET /api/v1/webhooks/:id/deliveries` — recent delivery attempts.
pub async fn webhook_deliveries(id: &str) -> Result<Vec<crate::models::WebhookDeliveryDto>, ApiError> {
    get(&format!("/webhooks/{id}/deliveries")).await
}
