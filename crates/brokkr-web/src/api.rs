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
pub(crate) fn token() -> Option<String> {
    injected_token()
}

/// Whether the injected token still works, as the responses show it.
///
/// The token is per broker process (`brokkr-broker/src/utils/ui_pak.rs`), so a
/// broker restart, or a request that a load balancer sends to a different
/// replica, makes every later request fail with 401 or 403. Each view would
/// then show its own "Not authorized"; the shell instead reads this state and
/// shows one banner that says a reload fixes it. A token refused from the
/// first request (`Refused`) gets its own banner: the broker answered, so the
/// shell must not say "broker unreachable".
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Session {
    /// No request with the injected token has succeeded yet.
    #[default]
    Unknown,
    /// A request with the injected token succeeded.
    Valid,
    /// The first answers refused the token (401 or 403) and none succeeded.
    /// The broker answered, so this is not "unreachable".
    Refused,
    /// A request was refused (401 or 403) after an earlier one succeeded.
    Expired,
}

impl Session {
    /// Whether the broker refused the injected token: it answered, so the
    /// shell must not say "broker unreachable".
    pub fn refused(self) -> bool {
        matches!(self, Session::Refused | Session::Expired)
    }

    /// The state after a response with `status` to a request that carried the
    /// injected token.
    ///
    /// A refusal before any success is `Refused`, not `Expired`: a token that
    /// never worked is not evidence of a restart. `Refused` becomes `Valid`
    /// when a later request succeeds: the token works after all (for example,
    /// a load balancer sent the first requests to a different replica), and
    /// the banner must not ask for a reload that is not necessary. Once
    /// `Expired`, the state stays there until a reload: behind a load balancer
    /// without session affinity some later requests can still succeed, and the
    /// page is broken all the same.
    pub fn after(self, status: u16) -> Session {
        match (self, status) {
            (Session::Expired, _) => Session::Expired,
            (_, 200..=299) => Session::Valid,
            (Session::Valid, 401 | 403) => Session::Expired,
            (Session::Unknown, 401 | 403) => Session::Refused,
            (state, _) => state,
        }
    }

    /// Whether a panel shows `error` as a quiet wait, not as an error.
    ///
    /// While the broker refuses the token (`Refused` or `Expired`), the shell
    /// banner already says what happened and that a reload is the fix. A 401
    /// or 403 in a panel is the same refusal, so the panel must not repeat it
    /// as a red "Not authorized" with the raw broker body. Other errors (404,
    /// 5xx, network) are not caused by the token and keep the error state.
    pub fn quiets(self, error: &ApiError) -> bool {
        match error {
            ApiError::Http { status, .. } => self.refused() && matches!(status, 401 | 403),
            ApiError::Network | ApiError::Unknown(_) => false,
        }
    }
}

thread_local! {
    /// The shell's session signal, set once by [`watch_session`]. A thread
    /// local, not a Leptos context: a fetch continues after an `.await`, where
    /// there is no reactive owner to read a context from.
    static SESSION: std::cell::Cell<Option<leptos::prelude::RwSignal<Session>>> =
        const { std::cell::Cell::new(None) };
}

/// Make every later request with the injected token update `signal`.
pub fn watch_session(signal: leptos::prelude::RwSignal<Session>) {
    SESSION.with(|cell| cell.set(Some(signal)));
}

/// The shell's session signal, as [`watch_session`] registered it. Before the
/// shell registers it, the session is always `Unknown`.
///
/// A panel reads this, not a Leptos context: the shell registers the signal
/// once for the fetch path, and the panels read the same registration, so the
/// session state has one source.
pub fn session() -> leptos::prelude::Signal<Session> {
    SESSION.with(|cell| match cell.get() {
        Some(signal) => signal.into(),
        None => leptos::prelude::Signal::stored(Session::Unknown),
    })
}

/// Feed one response status to the session signal, if the shell set one.
fn note_status(status: u16) {
    use leptos::prelude::{GetUntracked, Set};
    SESSION.with(|cell| {
        if let Some(signal) = cell.get() {
            let now = signal.get_untracked();
            let next = now.after(status);
            if next != now {
                signal.set(next);
            }
        }
    });
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
    note_status(status);
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

/// `GET /api/v1/stacks/:id/events`: the Kubernetes events an agent reported
/// for the stack, newest first, in the retention window (BROKKR-T-0338).
pub async fn stack_events(id: &str) -> Result<crate::models::K8sEventHistory, ApiError> {
    get(&format!("/stacks/{id}/events")).await
}

/// `GET /api/v1/stacks/:id/logs`: the pod log lines an agent streamed for
/// the stack, in the retention window.
pub async fn stack_logs(id: &str) -> Result<crate::models::PodLogHistory, ApiError> {
    get(&format!("/stacks/{id}/logs")).await
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
    note_status(status);
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

/// PUT `/api/v1{path}` authenticating with an **explicitly supplied** bearer
/// token, mirroring [`post_json_with_token`] (BROKKR-T-0322).
///
/// Same rule: the token is borrowed for one request and never captured,
/// logged, or stored. The console has no persistent credential store.
async fn put_json_with_token<B: Serialize, T: DeserializeOwned>(
    path: &str,
    body: &B,
    bearer: &str,
) -> Result<T, ApiError> {
    let url = format!("/api/v1{path}");
    let resp = Request::put(&url)
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

/// `PUT /api/v1/agents/{id}` — set an agent's `status`, pausing or resuming the
/// work it picks up (BROKKR-T-0322).
///
/// **This is what "paused" actually means.** The broker does not withhold
/// anything: the *agent* skips deployment-object fetches and work-order
/// processing while its status is not `ACTIVE`
/// (`brokkr-agent/src/cli/commands.rs:427` and `:545`), and re-reads its own
/// record each heartbeat, so a change here lands within a poll cycle. It is
/// agent-side self-restraint rather than an enforced boundary — see
/// BROKKR-T-0321, which tracks that distinction and whether the broker should
/// enforce it too.
///
/// The endpoint is `require_admin_or_agent`, and the console's injected token
/// is read-only, so the middleware rejects this before the handler runs. Hence
/// the operator-supplied admin PAK, per action.
///
/// The body is a partial update: the handler applies only the fields present,
/// so sending `status` alone cannot clobber the agent's name or cluster.
pub async fn set_agent_status(
    agent_id: &str,
    status: &str,
    admin_pak: &str,
) -> Result<crate::models::AgentRecord, ApiError> {
    put_json_with_token(
        &format!("/agents/{agent_id}"),
        &serde_json::json!({ "status": status }),
        admin_pak,
    )
    .await
}

#[cfg(test)]
mod tests {
    use super::Session;
    use aurora_leptos::tokens::ApiError;

    fn http(status: u16) -> ApiError {
        ApiError::Http {
            status,
            message: r#"{"code":"unauthorized","message":""}"#.to_string(),
            code: Some("unauthorized".to_string()),
        }
    }

    #[test]
    fn a_refused_session_quiets_401_and_403_only() {
        for session in [Session::Expired, Session::Refused] {
            assert!(session.quiets(&http(401)));
            assert!(session.quiets(&http(403)));
            assert!(!session.quiets(&http(404)));
            assert!(!session.quiets(&http(500)));
            assert!(!session.quiets(&ApiError::Network));
        }
    }

    #[test]
    fn a_working_session_quiets_nothing() {
        for session in [Session::Unknown, Session::Valid] {
            assert!(!session.quiets(&http(401)));
            assert!(!session.quiets(&http(403)));
            assert!(!session.quiets(&http(500)));
        }
    }

    #[test]
    fn a_refusal_after_a_success_expires_the_session() {
        let ok = Session::Unknown.after(200);
        assert_eq!(ok, Session::Valid);
        assert_eq!(ok.after(401), Session::Expired);
        assert_eq!(ok.after(403), Session::Expired);
    }

    #[test]
    fn a_refusal_before_any_success_is_a_refused_token() {
        assert_eq!(Session::Unknown.after(401), Session::Refused);
        assert_eq!(Session::Unknown.after(403), Session::Refused);
        assert_eq!(Session::Refused.after(401), Session::Refused);
        assert_eq!(Session::Refused.after(403), Session::Refused);
    }

    #[test]
    fn a_refused_token_that_later_works_is_valid() {
        let ok = Session::Unknown.after(401).after(200);
        assert_eq!(ok, Session::Valid);
        // A refusal after that success is a normal expiry.
        assert_eq!(ok.after(401), Session::Expired);
    }

    #[test]
    fn other_failures_before_any_success_are_not_a_refusal() {
        // A network failure has no status and never reaches `after`; a 5xx or
        // 404 at load is not a refused token either.
        for status in [404, 409, 500, 502, 503] {
            assert_eq!(Session::Unknown.after(status), Session::Unknown, "{status}");
            assert_eq!(Session::Refused.after(status), Session::Refused, "{status}");
        }
    }

    #[test]
    fn other_failures_do_not_expire_the_session() {
        for status in [404, 409, 500, 502, 503] {
            assert_eq!(Session::Valid.after(status), Session::Valid, "{status}");
        }
    }

    #[test]
    fn an_expired_session_stays_expired_until_a_reload() {
        assert_eq!(Session::Expired.after(200), Session::Expired);
        assert_eq!(Session::Expired.after(401), Session::Expired);
    }
}
