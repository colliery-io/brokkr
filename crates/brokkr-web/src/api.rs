//! Same-origin REST client to the broker. The console is served by the broker,
//! so the API is at `/api/v1/...`. Auth (BROKKR-I-0032): the broker injects an
//! ephemeral **read-only** UI PAK into the served HTML as
//! `<meta name="brokkr-ui-token">`; the console reads it on boot, so no
//! configuration is needed. Errors map to Aurora's `ApiError` for `ErrorState`.
//!
//! That injected token is the console's **only** ambient credential, and it is
//! read-only — so reaching the page grants visibility and nothing more. The one
//! privileged action (minting a generator tenant) takes an admin PAK the
//! operator supplies per request via [`post_json_with_token`]; it is never
//! stored. There is deliberately no persistent credential store here
//! (BROKKR-T-0320).

use crate::models::{
    DiagnosticRequestDto, DiagnosticResponse, ErrorBody, FleetAgentRecord, PakSummary,
    TargetStateObject,
};
use aurora_leptos::tokens::ApiError;
use gloo_net::http::Request;
use serde::de::DeserializeOwned;
use serde::Serialize;

/// The broker-injected read-only UI token, read from the `<meta>` tag once
/// and cached for the page lifetime (the token never changes within a page
/// load — a broker restart mints a new one, but also requires a reload).
fn injected_token() -> Option<String> {
    thread_local! {
        static TOKEN: std::cell::OnceCell<Option<String>> = const { std::cell::OnceCell::new() };
    }
    TOKEN.with(|cell| {
        cell.get_or_init(|| {
            let document = web_sys::window()?.document()?;
            let meta = document
                .query_selector("meta[name='brokkr-ui-token']")
                .ok()??;
            meta.get_attribute("content").filter(|s| !s.is_empty())
        })
        .clone()
    })
}

/// Bearer token for API calls: the broker-injected read-only UI token, and
/// nothing else.
///
/// There used to be an operator-pasted `localStorage["brokkr_pak"]` override
/// that took **precedence** over this, so setting it silently upgraded the
/// entire console from read-only to full admin write, for every request,
/// persistently (BROKKR-T-0320). Removed: all three of its justifications had
/// lapsed. The e2e harness does not need it (its route mocks ignore auth
/// headers), `trunk serve` cannot reach a real broker to authenticate to (no
/// proxy is configured), and the "full-write operator use" it existed for is
/// now served properly by the per-action admin PAK prompt in the tenants view,
/// which holds the credential in memory for one request instead of parking it
/// in browser storage indefinitely.
fn token() -> Option<String> {
    injected_token()
}

/// GET `/api/v1{path}` with `params` as the query string, and deserialize the
/// JSON body. Params go through the builder's query API rather than being baked
/// into `path` so the URL stays canonical (no stray separators — gloo-net
/// concatenates an existing query with its own and would leave a trailing `&`).
async fn get_query<T: DeserializeOwned>(
    path: &str,
    params: &[(&str, &str)],
) -> Result<T, ApiError> {
    let url = format!("/api/v1{path}");
    let mut req = Request::get(&url);
    if !params.is_empty() {
        req = req.query(params.iter().copied());
    }
    if let Some(t) = token() {
        req = req.header("Authorization", &format!("Bearer {t}"));
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

/// GET `/api/v1{path}` and deserialize the JSON body. `scope` becomes a
/// `?pak_id=` query param (tenant filter, BROKKR-I-0032).
pub async fn get_scoped<T: DeserializeOwned>(
    path: &str,
    scope: Option<String>,
) -> Result<T, ApiError> {
    match scope.filter(|s| !s.is_empty()) {
        Some(pak_id) => get_query(path, &[("pak_id", pak_id.as_str())]).await,
        None => get_query(path, &[]).await,
    }
}

/// GET `/api/v1{path}` (unscoped) and deserialize the JSON body.
pub async fn get<T: DeserializeOwned>(path: &str) -> Result<T, ApiError> {
    get_scoped(path, None).await
}

/// `GET /api/v1/fleet` — the fleet rollup (flat list of agents), optionally
/// scoped to a tenant.
pub async fn fleet(scope: Option<String>) -> Result<Vec<FleetAgentRecord>, ApiError> {
    get_scoped("/fleet", scope).await
}

/// `GET /api/v1/paks` — named PAKs (tenants) for the scope selector.
pub async fn paks() -> Result<Vec<PakSummary>, ApiError> {
    get("/paks").await
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

/// `GET /api/v1/stacks`, optionally scoped to a tenant.
pub async fn stacks(scope: Option<String>) -> Result<Vec<crate::models::Stack>, ApiError> {
    get_scoped("/stacks", scope).await
}

/// `GET /api/v1/agent-events`, optionally scoped to a tenant.
pub async fn agent_events(
    scope: Option<String>,
) -> Result<Vec<crate::models::AgentEventDto>, ApiError> {
    get_scoped("/agent-events", scope).await
}

/// POST `/api/v1{path}` with a JSON body, deserializing the response body
/// (BROKKR-T-0301 — the previous `post` discarded it, which threw away the
/// created diagnostic request's id and made the feature write-only). The
/// console's only write is diagnostic creation, which answers `201` with the
/// created record, so every caller wants the body; there is no body-discarding
/// variant left to keep in step.
pub async fn post_json<B: Serialize, T: DeserializeOwned>(
    path: &str,
    body: &B,
) -> Result<T, ApiError> {
    let url = format!("/api/v1{path}");
    let mut req = Request::post(&url);
    if let Some(t) = token() {
        req = req.header("Authorization", &format!("Bearer {t}"));
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

/// `GET /api/v1/agents/:id/target-state?mode=full` — every deployment object
/// currently targeted at an agent (`mode=full` includes already-deployed ones,
/// not just the undeployed delta). Admin-readable, so the read-only UI PAK
/// passes. Populates the Fleet modal's diagnostic picker.
pub async fn agent_target_state(agent_id: &str) -> Result<Vec<TargetStateObject>, ApiError> {
    get_query(
        &format!("/agents/{agent_id}/target-state"),
        &[("mode", "full")],
    )
    .await
}

/// `POST /api/v1/deployment-objects/:id/diagnostics` — ask `agent_id` to collect
/// diagnostics for one deployment object (the console's only write).
///
/// Diagnostics are inherently deployment-object-scoped — there is no bare
/// `POST /diagnostics` route, and this path is the one the broker's read-only
/// PAK allowlist admits (BROKKR-I-0032), so the injected UI token can drive it.
/// Returns the created request (201 body) so the caller can poll
/// [`diagnostic`] for its outcome (BROKKR-T-0301).
pub async fn create_diagnostic(
    deployment_object_id: &str,
    agent_id: &str,
) -> Result<DiagnosticRequestDto, ApiError> {
    post_json(
        &format!("/deployment-objects/{deployment_object_id}/diagnostics"),
        &serde_json::json!({ "agent_id": agent_id, "requested_by": "operator-console" }),
    )
    .await
}

/// `GET /api/v1/diagnostics/:id` — the request plus, once an agent has submitted
/// one, its result. A plain read, so the injected read-only UI token (which the
/// broker treats as a read-only *admin*) is admitted.
pub async fn diagnostic(id: &str) -> Result<DiagnosticResponse, ApiError> {
    get(&format!("/diagnostics/{id}")).await
}

/// `GET /api/v1/stacks/:id/health` — per-stack deployment-object health rollup.
pub async fn stack_health(id: &str) -> Result<crate::models::StackHealth, ApiError> {
    get(&format!("/stacks/{id}/health")).await
}

/// `GET /api/v1/webhooks/:id/deliveries` — recent delivery attempts.
pub async fn webhook_deliveries(
    id: &str,
) -> Result<Vec<crate::models::WebhookDeliveryDto>, ApiError> {
    get(&format!("/webhooks/{id}/deliveries")).await
}

/// `GET /api/v1/work-orders` — full work-order list (admin-gated).
pub async fn work_orders() -> Result<Vec<crate::models::WorkOrder>, ApiError> {
    get("/work-orders").await
}

/// `GET /api/v1/generators` — the tenant list. Admin-gated, so the injected
/// read-only UI token is admitted (it is a read-only *admin*). The broker
/// excludes the system generator from this listing, so what comes back is
/// exactly the set of real tenants.
pub async fn generators() -> Result<Vec<crate::models::Generator>, ApiError> {
    get("/generators").await
}

/// POST `/api/v1{path}` authenticating with an **explicitly supplied** bearer
/// token rather than [`token()`] (BROKKR-T-0318).
///
/// This exists so a privileged one-shot action can carry an operator-pasted
/// admin PAK **without that PAK ever being stored**. Deliberately does *not*
/// reuse a stored credential: the console deliberately has no persistent PAK
/// store (BROKKR-T-0320 removed the `localStorage` override, which survived
/// reloads and is readable by any script on the origin. An admin credential —
/// the strongest in the system — should not persist there just to create one
/// generator.
///
/// The token is borrowed for the duration of the call and never captured,
/// logged, or written anywhere. Failures return [`ApiError`] built from the
/// broker's `ErrorResponse` body, which never echoes the request's
/// `Authorization` header.
async fn post_json_with_token<B: Serialize, T: DeserializeOwned>(
    path: &str,
    body: &B,
    bearer: &str,
) -> Result<T, ApiError> {
    let url = format!("/api/v1{path}");
    let resp = Request::post(&url)
        .header("Authorization", &format!("Bearer {bearer}"))
        .json(body)
        .map_err(|_| ApiError::Network)?
        .send()
        .await
        .map_err(|_| ApiError::Network)?;
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

/// `POST /api/v1/generators` — mint a new generator tenant, authenticating with
/// the operator's admin PAK for this one request (BROKKR-T-0318).
///
/// The console's own credential cannot do this and is deliberately not used:
/// the injected UI token is read-only, so the broker would reject the write.
/// Requiring the operator to supply an admin PAK per action is what keeps
/// "network reach is the console's authentication boundary" true — reaching the
/// page grants no ability to mint anything.
///
/// The response carries the new generator's PAK in plaintext, exactly once.
pub async fn create_generator(
    name: &str,
    description: Option<&str>,
    admin_pak: &str,
) -> Result<crate::models::CreateGeneratorResponse, ApiError> {
    post_json_with_token(
        "/generators",
        &serde_json::json!({
            "name": name,
            "description": description,
        }),
        admin_pak,
    )
    .await
}
