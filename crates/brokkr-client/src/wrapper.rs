/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Ergonomic wrapper around the progenitor-generated [`crate::Client`].
//!
//! The generated [`Client`] is a faithful 1:1 of the OpenAPI spec — every
//! operation is a per-call builder, auth is supplied via the underlying
//! `reqwest::Client`, and errors come back as `progenitor_client::Error<E>`.
//! That surface is correct but verbose. This module adds:
//!
//! - A single-credential constructor that injects the `Authorization` header
//!   on every request. Hides the fact that the spec declares three security
//!   schemes (`admin_pak`, `agent_pak`, `generator_pak`) — they all map to
//!   the same header and the broker disambiguates at runtime.
//! - A typed [`BrokkrError`] that wraps the generated error enum and exposes
//!   the `code` string from [`crate::types::ErrorResponse`] for pattern
//!   matching.
//! - A [`BrokkrClient::retry`] helper that re-invokes a fallible operation
//!   with exponential backoff on transient failures. Retry is opt-in —
//!   callers wrap individual ops they consider safe to retry.
//!
//! Pagination iterators are intentionally not provided: the v1 broker API
//! returns full collections without cursor tokens (per the audit). If
//! pagination is introduced later, add `Stream` adapters here.
//!
//! The module is intentionally small. If it grows past a few hundred lines,
//! that is a signal to push complexity back into the spec rather than the
//! wrapper.

use std::time::Duration;

use progenitor_client::Error as RawError;
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};

use crate::Client;
use crate::types::{
    AgentGeneratorRegistration, AgentRegistrationBody, CreateDeploymentObjectRequest,
    DeploymentObject, ErrorResponse, K8sEventHistoryResponse, NewStack, PodLogHistoryResponse,
    Stack, WsConnectionsResponse,
};
use chrono::{DateTime, Utc};
use std::path::Path;
use uuid::Uuid;

/// Top-level error returned by every wrapper method. Mirrors
/// [`progenitor_client::Error`] but specialises `E` to [`ErrorResponse`] so
/// callers can match on [`ErrorResponse::code`] directly.
#[derive(Debug)]
pub enum BrokkrError {
    /// A documented 4xx/5xx response body. Match on `.code` for stable
    /// machine-readable error categorisation (e.g. `agent_not_found`).
    Api(ErrorResponse, reqwest::StatusCode),
    /// Local or transport error (DNS, TLS, timeout, connection reset, etc).
    Transport(reqwest::Error),
    /// Server returned a response shape that did not match the spec. Usually
    /// a sign of spec drift; investigate with the raw bytes attached.
    UnexpectedResponse {
        status: Option<reqwest::StatusCode>,
        detail: String,
    },
    /// Request rejected before transmission (bad input).
    InvalidRequest(String),
}

impl BrokkrError {
    /// HTTP status, when known.
    pub fn status(&self) -> Option<reqwest::StatusCode> {
        match self {
            Self::Api(_, status) => Some(*status),
            Self::Transport(e) => e.status(),
            Self::UnexpectedResponse { status, .. } => *status,
            Self::InvalidRequest(_) => None,
        }
    }

    /// Stable, machine-readable error code from the wire response, if any.
    /// Pattern-match on this rather than the human-readable message.
    pub fn code(&self) -> Option<&str> {
        match self {
            Self::Api(body, _) => Some(&body.code),
            _ => None,
        }
    }

    /// Whether this error is appropriate to retry. Mirrors
    /// [`progenitor_client::Error::is_retryable`]: transport errors and
    /// 408/429/502/503/504 responses qualify.
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Transport(_) => true,
            Self::Api(_, status) => is_retryable_status(*status),
            Self::UnexpectedResponse {
                status: Some(status),
                ..
            } => is_retryable_status(*status),
            _ => false,
        }
    }
}

impl std::fmt::Display for BrokkrError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Api(body, status) => {
                write!(f, "{} {}: {}", status.as_u16(), body.code, body.message)
            }
            Self::Transport(e) => write!(f, "transport error: {e}"),
            Self::UnexpectedResponse { status, detail } => match status {
                Some(s) => write!(f, "unexpected response ({}): {}", s.as_u16(), detail),
                None => write!(f, "unexpected response: {detail}"),
            },
            Self::InvalidRequest(msg) => write!(f, "invalid request: {msg}"),
        }
    }
}

impl std::error::Error for BrokkrError {}

impl From<RawError<ErrorResponse>> for BrokkrError {
    fn from(err: RawError<ErrorResponse>) -> Self {
        match err {
            RawError::ErrorResponse(rv) => {
                let status = rv.status();
                Self::Api(rv.into_inner(), status)
            }
            RawError::CommunicationError(e)
            | RawError::InvalidUpgrade(e)
            | RawError::ResponseBodyError(e) => Self::Transport(e),
            RawError::InvalidRequest(msg) => Self::InvalidRequest(msg),
            RawError::InvalidResponsePayload(bytes, e) => Self::UnexpectedResponse {
                status: None,
                detail: format!(
                    "payload deserialization failed: {e} ({} bytes)",
                    bytes.len()
                ),
            },
            RawError::UnexpectedResponse(resp) => Self::UnexpectedResponse {
                status: Some(resp.status()),
                detail: "response not described in OpenAPI spec".to_string(),
            },
            RawError::Custom(s) => Self::InvalidRequest(s),
        }
    }
}

fn is_retryable_status(status: reqwest::StatusCode) -> bool {
    matches!(status.as_u16(), 408 | 429 | 502 | 503 | 504)
}

/// Builder for [`BrokkrClient`]. Use [`BrokkrClient::builder`] to start.
#[derive(Debug)]
pub struct BrokkrClientBuilder {
    base_url: String,
    token: Option<String>,
    request_timeout: Duration,
    connect_timeout: Duration,
    max_retries: u32,
    initial_backoff: Duration,
}

impl BrokkrClientBuilder {
    fn new(base_url: impl Into<String>) -> Self {
        Self {
            base_url: base_url.into(),
            token: None,
            request_timeout: Duration::from_secs(30),
            connect_timeout: Duration::from_secs(10),
            max_retries: 3,
            initial_backoff: Duration::from_millis(200),
        }
    }

    /// PAK credential (admin, agent, or generator). The wrapper sends this as
    /// the `Authorization` header on every request; the broker inspects the
    /// PAK prefix to determine which security scheme applies.
    pub fn token(mut self, token: impl Into<String>) -> Self {
        self.token = Some(token.into());
        self
    }

    /// Total per-request timeout. Default: 30 seconds.
    pub fn request_timeout(mut self, timeout: Duration) -> Self {
        self.request_timeout = timeout;
        self
    }

    /// TCP connect timeout. Default: 10 seconds.
    pub fn connect_timeout(mut self, timeout: Duration) -> Self {
        self.connect_timeout = timeout;
        self
    }

    /// Maximum retry attempts for [`BrokkrClient::retry`]. Default: 3.
    /// A value of 0 disables retry; the first attempt always runs.
    pub fn max_retries(mut self, max: u32) -> Self {
        self.max_retries = max;
        self
    }

    /// Initial backoff between retry attempts. Doubles on each subsequent
    /// failure (capped at 10s). Default: 200ms.
    pub fn initial_backoff(mut self, initial: Duration) -> Self {
        self.initial_backoff = initial;
        self
    }

    pub fn build(self) -> Result<BrokkrClient, BrokkrError> {
        let mut headers = HeaderMap::new();
        if let Some(token) = &self.token {
            let value = HeaderValue::from_str(token).map_err(|e| {
                BrokkrError::InvalidRequest(format!("invalid token header value: {e}"))
            })?;
            headers.insert(AUTHORIZATION, value);
        }

        let reqwest_client = reqwest::Client::builder()
            .default_headers(headers)
            .connect_timeout(self.connect_timeout)
            .timeout(self.request_timeout)
            .build()
            .map_err(BrokkrError::Transport)?;

        let inner = Client::new_with_client(&self.base_url, reqwest_client);
        Ok(BrokkrClient {
            inner,
            max_retries: self.max_retries,
            initial_backoff: self.initial_backoff,
        })
    }
}

/// Ergonomic client for the Brokkr broker API.
///
/// Construct via [`BrokkrClient::builder`]. The wrapper holds a configured
/// [`Client`] (the generated low-level client) and a retry policy. Access the
/// generated builders through [`BrokkrClient::api`].
#[derive(Debug, Clone)]
pub struct BrokkrClient {
    inner: Client,
    max_retries: u32,
    initial_backoff: Duration,
}

impl BrokkrClient {
    /// Start building a client. `base_url` should include the version prefix
    /// (e.g. `https://broker.example.com/api/v1`).
    pub fn builder(base_url: impl Into<String>) -> BrokkrClientBuilder {
        BrokkrClientBuilder::new(base_url)
    }

    /// Access the underlying generated client. Every spec operation is
    /// available as a builder method on it: e.g.
    /// `client.api().list_agents().send().await`.
    pub fn api(&self) -> &Client {
        &self.inner
    }

    // -------------------------------------------------------------------
    // Ergonomic methods for the internal-WS-channel surface
    // (BROKKR-I-0019). These wrap the generated builders so callers can
    // skip the `.api().<op>().<param>().<param>().send().await` chain
    // for the most common cases. The retention metadata is part of the
    // typed response — callers should surface it (or at least not hide
    // it) per ADR-0008 / project_log_retention_stance.
    // -------------------------------------------------------------------

    /// Paginated kube-event history for a stack, scoped to the 6h
    /// retention window. The response carries the `retention` block —
    /// surface it in any UI built on this SDK so users aren't surprised
    /// by missing rows.
    pub async fn list_telemetry_events(
        &self,
        stack_id: Uuid,
        since: Option<DateTime<Utc>>,
        limit: Option<i64>,
    ) -> Result<K8sEventHistoryResponse, BrokkrError> {
        let mut req = self.inner.list_telemetry_events().id(stack_id);
        if let Some(since) = since {
            req = req.since(since);
        }
        if let Some(limit) = limit {
            req = req.limit(limit);
        }
        let resp = req.send().await?;
        Ok(resp.into_inner())
    }

    /// Paginated pod-log history for a stack within the 6h retention
    /// window. See [`Self::list_telemetry_events`] for retention
    /// semantics — same response shape modulo the row type.
    pub async fn list_telemetry_logs(
        &self,
        stack_id: Uuid,
        since: Option<DateTime<Utc>>,
        limit: Option<i64>,
    ) -> Result<PodLogHistoryResponse, BrokkrError> {
        let mut req = self.inner.list_telemetry_logs().id(stack_id);
        if let Some(since) = since {
            req = req.since(since);
        }
        if let Some(limit) = limit {
            req = req.limit(limit);
        }
        let resp = req.send().await?;
        Ok(resp.into_inner())
    }

    /// Snapshot of currently-connected agents on the internal WS
    /// channel (admin-only). Useful for fleet diagnostics — for
    /// continuous monitoring prefer scraping the
    /// `brokkr_ws_connected_agents` Prometheus gauge.
    pub async fn list_ws_connections(&self) -> Result<WsConnectionsResponse, BrokkrError> {
        let resp = self.inner.list_ws_connections().send().await?;
        Ok(resp.into_inner())
    }

    // -------------------------------------------------------------------
    // Manifest submission helpers (BROKKR-I-0021). The control-plane
    // friction is taking a folder of Kubernetes manifests and getting it
    // submitted as a stack's desired state. These read the folder,
    // concatenate it into one multi-document YAML stream, validate it, and
    // submit — so callers hand over a directory, not a hand-built blob.
    // "1 stack = 1 rendered bundle"; the latest deployment object is the
    // stack's desired state, and the agent reconciles + prunes.
    // -------------------------------------------------------------------

    /// Read a folder (or file/list of files) of `*.yaml`/`*.yml` manifests,
    /// concatenate them into one multi-document stream, validate that each
    /// document parses and carries `apiVersion`+`kind`, and submit it as a
    /// new deployment object on an existing stack.
    ///
    /// Files in a directory are concatenated in sorted-name order. Ordering
    /// is forgiving: the agent front-loads `Namespace`/`CustomResourceDefinition`
    /// objects during apply.
    pub async fn submit_manifests(
        &self,
        stack_id: Uuid,
        path: impl AsRef<Path>,
    ) -> Result<DeploymentObject, BrokkrError> {
        let yaml_content = read_manifests(path.as_ref())?;
        let resp = self
            .inner
            .create_deployment_object()
            .id(stack_id)
            .body(CreateDeploymentObjectRequest {
                yaml_content,
                is_deletion_marker: Some(false),
            })
            .send()
            .await?;
        Ok(resp.into_inner())
    }

    /// Idempotently make a folder of manifests the desired state of the stack
    /// named `stack_name`, creating the stack if it does not exist and
    /// applying any `targeting` labels (for fan-out). A new revision is
    /// submitted only when the bundle differs from the stack's current latest
    /// deployment object, so this drops straight into a reconcile loop.
    ///
    /// Requires a generator PAK (the new stack is owned by that generator);
    /// admin callers should create the stack explicitly and use
    /// [`Self::submit_manifests`].
    pub async fn apply(
        &self,
        stack_name: &str,
        path: impl AsRef<Path>,
        targeting: &[String],
    ) -> Result<ApplyOutcome, BrokkrError> {
        let yaml_content = read_manifests(path.as_ref())?;
        let checksum = sha256_hex(&yaml_content);

        // Resolve the caller's generator identity (needed to own a new stack).
        let auth = self.inner.verify_pak().send().await?.into_inner();
        let generator_id = auth
            .generator
            .ok_or_else(|| {
                BrokkrError::InvalidRequest(
                    "apply by name requires a generator PAK; admin callers should create the \
                     stack explicitly and use submit_manifests"
                        .to_string(),
                )
            })
            .and_then(|g| {
                Uuid::parse_str(&g).map_err(|e| BrokkrError::UnexpectedResponse {
                    status: None,
                    detail: format!("auth response generator id is not a UUID: {e}"),
                })
            })?;

        // Find-or-create the stack by name.
        let stacks: Vec<Stack> = self.inner.list_stacks().send().await?.into_inner();
        let stack = match stacks.into_iter().find(|s| s.name == stack_name) {
            Some(s) => s,
            None => self
                .inner
                .create_stack()
                .body(NewStack {
                    name: stack_name.to_string(),
                    generator_id,
                    description: None,
                })
                .send()
                .await?
                .into_inner(),
        };

        // Apply targeting labels; a label that already exists is not an error.
        for label in targeting {
            if let Err(e) = self
                .inner
                .stacks_add_label()
                .id(stack.id)
                .body(label.clone())
                .send()
                .await
            {
                let err = BrokkrError::from(e);
                if err.status() != Some(reqwest::StatusCode::CONFLICT) {
                    return Err(err);
                }
            }
        }

        // Idempotency: skip submission when the latest bundle already matches.
        let objects: Vec<DeploymentObject> = self
            .inner
            .list_deployment_objects()
            .id(stack.id)
            .send()
            .await?
            .into_inner();
        let had_prior = !objects.is_empty();
        let already_current = objects
            .iter()
            .max_by_key(|o| o.sequence_id)
            .map(|latest| latest.yaml_checksum == checksum)
            .unwrap_or(false);
        if already_current {
            return Ok(ApplyOutcome::Unchanged);
        }

        let object = self
            .inner
            .create_deployment_object()
            .id(stack.id)
            .body(CreateDeploymentObjectRequest {
                yaml_content,
                is_deletion_marker: Some(false),
            })
            .send()
            .await?
            .into_inner();

        Ok(if had_prior {
            ApplyOutcome::Updated(object)
        } else {
            ApplyOutcome::Created(object)
        })
    }

    // -------------------------------------------------------------------
    // Generator registration helpers (BROKKR-I-0030). An agent must be
    // registered with a generator before that generator's stacks can be
    // targeted at it. The agent self-registers on startup; these methods are
    // the admin-side path for bootstrapping and inspection.
    // -------------------------------------------------------------------

    /// Register an agent with a generator scope. Pass `agent_id = None` to
    /// register the calling agent (implied by its PAK); an admin supplies the
    /// target agent's id. Registering an already-registered pair returns the
    /// existing registration. Returns the registration record.
    pub async fn register_agent(
        &self,
        generator_id: Uuid,
        agent_id: Option<Uuid>,
    ) -> Result<AgentGeneratorRegistration, BrokkrError> {
        let resp = self
            .inner
            .register_agent()
            .id(generator_id)
            .body(AgentRegistrationBody { agent_id })
            .send()
            .await?;
        Ok(resp.into_inner())
    }

    /// Remove an agent's registration from a generator scope. Pass
    /// `agent_id = None` for the calling agent, or an admin supplies the target.
    ///
    /// Destructive: the broker cascades the agent's `agent_targets` for that
    /// generator and pushes a `TargetChanged` frame to the departing agent so it
    /// prunes the corresponding Kubernetes resources on its next reconcile.
    pub async fn deregister_agent(
        &self,
        generator_id: Uuid,
        agent_id: Option<Uuid>,
    ) -> Result<(), BrokkrError> {
        self.inner
            .deregister_agent()
            .id(generator_id)
            .body(AgentRegistrationBody { agent_id })
            .send()
            .await?;
        Ok(())
    }

    /// List the generator scopes an agent is registered with.
    pub async fn list_agent_registrations(
        &self,
        agent_id: Uuid,
    ) -> Result<Vec<AgentGeneratorRegistration>, BrokkrError> {
        let resp = self
            .inner
            .list_agent_registrations()
            .id(agent_id)
            .send()
            .await?;
        Ok(resp.into_inner())
    }

    /// List the agents registered with a generator scope.
    pub async fn list_generator_registered_agents(
        &self,
        generator_id: Uuid,
    ) -> Result<Vec<AgentGeneratorRegistration>, BrokkrError> {
        let resp = self
            .inner
            .list_generator_registered_agents()
            .id(generator_id)
            .send()
            .await?;
        Ok(resp.into_inner())
    }

    /// Run `op` with exponential backoff on retryable errors.
    ///
    /// The closure is invoked at most `max_retries + 1` times (configured via
    /// [`BrokkrClientBuilder::max_retries`]). Between attempts, the wrapper
    /// sleeps for `initial_backoff * 2^(attempt - 1)`, capped at 10 seconds.
    /// Non-retryable errors return immediately on the first attempt.
    ///
    /// Callers are responsible for choosing safe operations to retry. POSTs
    /// that are not idempotent should generally not be wrapped.
    pub async fn retry<F, Fut, T>(&self, mut op: F) -> Result<T, BrokkrError>
    where
        F: FnMut(&Client) -> Fut,
        Fut: std::future::Future<Output = Result<T, BrokkrError>>,
    {
        let mut attempt: u32 = 0;
        loop {
            match op(&self.inner).await {
                Ok(value) => return Ok(value),
                Err(err) if !err.is_retryable() || attempt >= self.max_retries => {
                    return Err(err);
                }
                Err(_) => {
                    let backoff = self
                        .initial_backoff
                        .saturating_mul(1u32 << attempt)
                        .min(Duration::from_secs(10));
                    tokio::time::sleep(backoff).await;
                    attempt += 1;
                }
            }
        }
    }
}

/// Outcome of [`BrokkrClient::apply`].
#[derive(Debug)]
pub enum ApplyOutcome {
    /// The stack had no prior deployment object; this bundle is its first.
    Created(DeploymentObject),
    /// A new revision was submitted (the bundle differed from the latest).
    Updated(DeploymentObject),
    /// The stack's latest bundle already matched; nothing was submitted.
    Unchanged,
}

/// Read a manifest path into one validated multi-document YAML stream.
///
/// `path` may be a single file or a directory; for a directory, top-level
/// `*.yaml`/`*.yml` files are concatenated in sorted-name order. Each
/// document must parse and carry `apiVersion` and `kind`.
fn read_manifests(path: &Path) -> Result<String, BrokkrError> {
    let files = collect_manifest_files(path)?;
    if files.is_empty() {
        return Err(BrokkrError::InvalidRequest(format!(
            "no .yaml/.yml manifests found in {}",
            path.display()
        )));
    }
    let mut parts: Vec<String> = Vec::with_capacity(files.len());
    for file in &files {
        let content = std::fs::read_to_string(file).map_err(|e| {
            BrokkrError::InvalidRequest(format!("cannot read {}: {e}", file.display()))
        })?;
        validate_manifest_documents(&content, file)?;
        parts.push(content.trim_end().to_string());
    }
    Ok(format!("{}\n", parts.join("\n---\n")))
}

/// Resolve a manifest path to the concrete list of files to read.
fn collect_manifest_files(path: &Path) -> Result<Vec<std::path::PathBuf>, BrokkrError> {
    if path.is_file() {
        return Ok(vec![path.to_path_buf()]);
    }
    if !path.is_dir() {
        return Err(BrokkrError::InvalidRequest(format!(
            "path not found: {}",
            path.display()
        )));
    }
    let mut files: Vec<std::path::PathBuf> = std::fs::read_dir(path)
        .map_err(|e| {
            BrokkrError::InvalidRequest(format!("cannot read directory {}: {e}", path.display()))
        })?
        .filter_map(|entry| entry.ok().map(|e| e.path()))
        .filter(|p| {
            p.is_file()
                && matches!(
                    p.extension().and_then(|s| s.to_str()),
                    Some("yaml") | Some("yml")
                )
        })
        .collect();
    files.sort();
    Ok(files)
}

/// Validate that every non-empty document in `content` parses and carries
/// `apiVersion` and `kind`.
fn validate_manifest_documents(content: &str, file: &Path) -> Result<(), BrokkrError> {
    use serde::Deserialize;
    for doc in serde_yaml::Deserializer::from_str(content) {
        let value = serde_yaml::Value::deserialize(doc).map_err(|e| {
            BrokkrError::InvalidRequest(format!("{}: invalid YAML: {e}", file.display()))
        })?;
        if value.is_null() {
            continue;
        }
        let has = |key: &str| value.get(key).and_then(|v| v.as_str()).is_some();
        if !has("apiVersion") || !has("kind") {
            return Err(BrokkrError::InvalidRequest(format!(
                "{}: every manifest document must have apiVersion and kind",
                file.display()
            )));
        }
    }
    Ok(())
}

/// Lowercase hex SHA-256, matching the broker's deployment-object checksum so
/// [`BrokkrClient::apply`] can detect an unchanged bundle.
fn sha256_hex(content: &str) -> String {
    use sha2::{Digest, Sha256};
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    format!("{:x}", hasher.finalize())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builder_constructs_without_token() {
        use progenitor_client::ClientInfo;
        let c = BrokkrClient::builder("http://localhost:3000/api/v1")
            .build()
            .expect("builder should succeed");
        assert_eq!(c.api().baseurl(), "http://localhost:3000/api/v1");
    }

    #[test]
    fn builder_accepts_token_and_timeouts() {
        let c = BrokkrClient::builder("http://localhost:3000/api/v1")
            .token("bk_admin_test_token")
            .request_timeout(Duration::from_secs(5))
            .connect_timeout(Duration::from_secs(2))
            .max_retries(5)
            .initial_backoff(Duration::from_millis(50))
            .build()
            .expect("builder should succeed");
        assert_eq!(c.max_retries, 5);
        assert_eq!(c.initial_backoff, Duration::from_millis(50));
    }

    #[test]
    fn invalid_token_header_is_rejected() {
        let result = BrokkrClient::builder("http://localhost:3000/api/v1")
            .token("invalid\nheader\rvalue")
            .build();
        assert!(matches!(result, Err(BrokkrError::InvalidRequest(_))));
    }

    #[test]
    fn error_code_extracted_from_api_response() {
        let err = BrokkrError::Api(
            ErrorResponse {
                code: "agent_not_found".to_string(),
                message: "agent not found".to_string(),
                details: None,
            },
            reqwest::StatusCode::NOT_FOUND,
        );
        assert_eq!(err.code(), Some("agent_not_found"));
        assert_eq!(err.status(), Some(reqwest::StatusCode::NOT_FOUND));
        assert!(!err.is_retryable());
    }

    #[test]
    fn retryable_classification() {
        for status in [408u16, 429, 502, 503, 504] {
            let err = BrokkrError::Api(
                ErrorResponse {
                    code: "transient".to_string(),
                    message: "x".to_string(),
                    details: None,
                },
                reqwest::StatusCode::from_u16(status).unwrap(),
            );
            assert!(err.is_retryable(), "{status} should be retryable");
        }
        for status in [400u16, 401, 403, 404, 409, 422, 500, 501] {
            let err = BrokkrError::Api(
                ErrorResponse {
                    code: "non_transient".to_string(),
                    message: "x".to_string(),
                    details: None,
                },
                reqwest::StatusCode::from_u16(status).unwrap(),
            );
            assert!(!err.is_retryable(), "{status} should NOT be retryable");
        }
    }

    #[tokio::test(start_paused = true)]
    async fn retry_stops_after_max_attempts() {
        let client = BrokkrClient::builder("http://localhost:3000/api/v1")
            .max_retries(2)
            .initial_backoff(Duration::from_millis(1))
            .build()
            .unwrap();
        let calls = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let calls_clone = calls.clone();
        let result: Result<(), BrokkrError> = client
            .retry(|_| {
                let calls = calls_clone.clone();
                async move {
                    calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    // A retryable error per is_retryable_status (503).
                    Err(BrokkrError::Api(
                        ErrorResponse {
                            code: "transient".to_string(),
                            message: "service unavailable".to_string(),
                            details: None,
                        },
                        reqwest::StatusCode::SERVICE_UNAVAILABLE,
                    ))
                }
            })
            .await;
        assert!(result.is_err());
        // Initial attempt + 2 retries = 3 calls total.
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 3);
    }

    // -----------------------------------------------------------------
    // WS-10 / WS-13 ergonomic-method surface (BROKKR-I-0019).
    //
    // We don't run a real broker here; the contract under test is that
    // the method exists with the right signature and returns the
    // declared response type. End-to-end coverage is in
    // `crates/brokkr-broker/tests/integration/api/ws.rs`.
    // -----------------------------------------------------------------
    #[test]
    fn ws_wrapper_methods_compile_with_expected_signatures() {
        fn _assert_signatures() {
            async fn _types_check() {
                let c = BrokkrClient::builder("http://localhost:3000/api/v1")
                    .build()
                    .unwrap();
                let id = uuid::Uuid::nil();
                let _ev: K8sEventHistoryResponse =
                    c.list_telemetry_events(id, None, None).await.unwrap();
                let _lo: PodLogHistoryResponse =
                    c.list_telemetry_logs(id, None, Some(100)).await.unwrap();
                let _co: WsConnectionsResponse = c.list_ws_connections().await.unwrap();
            }
            let _ = _types_check;
        }
    }

    #[tokio::test(start_paused = true)]
    async fn retry_returns_immediately_on_non_retryable() {
        let client = BrokkrClient::builder("http://localhost:3000/api/v1")
            .max_retries(5)
            .build()
            .unwrap();
        let calls = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
        let calls_clone = calls.clone();
        let result: Result<(), BrokkrError> = client
            .retry(|_| {
                let calls = calls_clone.clone();
                async move {
                    calls.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                    Err(BrokkrError::Api(
                        ErrorResponse {
                            code: "agent_not_found".to_string(),
                            message: "x".to_string(),
                            details: None,
                        },
                        reqwest::StatusCode::NOT_FOUND,
                    ))
                }
            })
            .await;
        assert!(result.is_err());
        assert_eq!(calls.load(std::sync::atomic::Ordering::SeqCst), 1);
    }

    // --- BROKKR-T-0195: manifest folder helpers ---

    fn write(dir: &std::path::Path, name: &str, content: &str) {
        std::fs::write(dir.join(name), content).unwrap();
    }

    #[test]
    fn read_manifests_concatenates_folder_in_sorted_order() {
        let dir = tempfile::tempdir().unwrap();
        // intentionally out of order on disk; sorted by name on read
        write(dir.path(), "02-deploy.yaml", "apiVersion: apps/v1\nkind: Deployment\nmetadata:\n  name: d\n");
        write(dir.path(), "01-namespace.yaml", "apiVersion: v1\nkind: Namespace\nmetadata:\n  name: ns\n");
        write(dir.path(), "notes.txt", "ignored");
        let stream = read_manifests(dir.path()).unwrap();
        let ns_at = stream.find("kind: Namespace").unwrap();
        let dep_at = stream.find("kind: Deployment").unwrap();
        assert!(ns_at < dep_at, "01-namespace should come before 02-deploy");
        assert!(stream.contains("\n---\n"), "documents joined with a separator");
        assert!(!stream.contains("ignored"), "non-yaml files are skipped");
    }

    #[test]
    fn read_manifests_accepts_single_file_and_multidoc() {
        let dir = tempfile::tempdir().unwrap();
        write(dir.path(), "all.yaml", "apiVersion: v1\nkind: Namespace\nmetadata:\n  name: a\n---\napiVersion: v1\nkind: ConfigMap\nmetadata:\n  name: b\n");
        let stream = read_manifests(&dir.path().join("all.yaml")).unwrap();
        assert!(stream.contains("kind: Namespace") && stream.contains("kind: ConfigMap"));
    }

    #[test]
    fn read_manifests_rejects_missing_apiversion_or_kind() {
        let dir = tempfile::tempdir().unwrap();
        write(dir.path(), "bad.yaml", "kind: ConfigMap\nmetadata:\n  name: x\n");
        let err = read_manifests(dir.path()).unwrap_err();
        assert!(matches!(err, BrokkrError::InvalidRequest(_)), "got {err:?}");
    }

    #[test]
    fn read_manifests_rejects_malformed_yaml() {
        let dir = tempfile::tempdir().unwrap();
        write(dir.path(), "bad.yaml", "kind: : : [unbalanced");
        assert!(read_manifests(dir.path()).is_err());
    }

    #[test]
    fn read_manifests_errors_on_empty_dir_and_missing_path() {
        let dir = tempfile::tempdir().unwrap();
        assert!(read_manifests(dir.path()).is_err(), "empty dir");
        assert!(read_manifests(&dir.path().join("nope")).is_err(), "missing path");
    }

    #[test]
    fn sha256_hex_is_stable_and_matches_known_vector() {
        // Matches the broker's `format!("{:x}", Sha256::digest(...))`.
        assert_eq!(
            sha256_hex(""),
            "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
        );
        let a = "apiVersion: v1\nkind: ConfigMap\n";
        assert_eq!(sha256_hex(a), sha256_hex(a));
        assert_ne!(sha256_hex(a), sha256_hex("apiVersion: v1\nkind: Secret\n"));
    }

}
