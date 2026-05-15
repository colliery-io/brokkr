/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Broker communication module for agent-broker interaction.
//!
//! All v1 API calls go through [`brokkr_client::BrokkrClient`] (T-D1
//! migration). The only remaining `reqwest::Client` usage is in
//! [`wait_for_broker_ready`] for the `/readyz` health probe, which lives
//! outside the v1 surface and is therefore not part of the generated SDK.
//!
//! Public function signatures continue to exchange `brokkr-models` types
//! (`Agent`, `DeploymentObject`, …) so callers (`cli/commands.rs`) didn't
//! have to change. Inside each function we JSON-round-trip between the
//! SDK's generated types and the workspace's `brokkr-models` types — the
//! wire form is byte-identical and the conversion is cheap on the
//! frequencies we operate at (seconds-scale).

use std::time::{Duration, Instant};

use brokkr_client::{BrokkrClient, BrokkrError};
use brokkr_models::models::agent_events::NewAgentEvent;
use brokkr_models::models::agents::Agent;
use brokkr_models::models::deployment_objects::DeploymentObject;
use brokkr_utils::Settings;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tokio::time::sleep;
use tracing::{debug, error, info, instrument, trace, warn};
use uuid::Uuid;

use crate::deployment_health::{DeploymentObjectHealthUpdate, HealthStatusUpdate};
use crate::diagnostics::{DiagnosticRequest, SubmitDiagnosticResult};
use crate::metrics;

/// HTTP status helper. The agent and `brokkr-client` link different reqwest
/// majors (0.11 vs 0.13), so we never compare `StatusCode` values directly —
/// always go through `as_u16()`.
fn status_u16(err: &BrokkrError) -> Option<u16> {
    err.status().map(|s| s.as_u16())
}

/// JSON-round-trip between two `serde`-compatible types. Used to bridge the
/// SDK's `brokkr_client::types::*` and the workspace's `brokkr_models` types,
/// which have identical wire formats but distinct Rust identities.
fn convert<From: Serialize, To: DeserializeOwned>(value: From) -> Result<To, serde_json::Error> {
    let v = serde_json::to_value(value)?;
    serde_json::from_value(v)
}

/// Map a `BrokkrError` into the agent's historical `Box<dyn Error>` shape with
/// a stable prefix. Status-aware mapping happens at the call site.
fn boxed(prefix: &str, err: BrokkrError) -> Box<dyn std::error::Error> {
    let msg = match status_u16(&err) {
        Some(s) => format!("{prefix}. Status: {s}, Error: {err}"),
        None => format!("{prefix}: {err}"),
    };
    msg.into()
}

/// Waits for the broker service to become ready.
///
/// Not migrated to the SDK: `/readyz` is outside the v1 spec surface.
pub async fn wait_for_broker_ready(config: &Settings) {
    let client = reqwest::Client::new();
    let readyz_url = format!("{}/readyz", config.agent.broker_url);

    for attempt in 1..=config.agent.max_retries {
        match client.get(&readyz_url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    info!("Successfully connected to broker at {}", readyz_url);
                    return;
                }
                warn!(
                    "Broker at {} returned non-success status: {}",
                    readyz_url,
                    response.status()
                );
            }
            Err(e) => {
                warn!(
                    "Failed to connect to broker at {} (attempt {}/{}): {:?}",
                    readyz_url, attempt, config.agent.max_retries, e
                );
            }
        }
        if attempt < config.agent.max_retries {
            info!(
                "Waiting for broker to be ready at {} (attempt {}/{})",
                readyz_url, attempt, config.agent.max_retries
            );
            sleep(Duration::from_secs(1)).await;
        }
    }
    error!(
        "Failed to connect to broker at {} after {} attempts. Exiting.",
        readyz_url, config.agent.max_retries
    );
    std::process::exit(1);
}

/// Verifies the agent's Personal Access Key (PAK) with the broker.
#[instrument(skip(config), fields(broker_url = %config.agent.broker_url))]
pub async fn verify_agent_pak(config: &Settings) -> Result<(), Box<dyn std::error::Error>> {
    debug!("Verifying agent PAK at {}", config.agent.broker_url);
    let sdk = crate::broker_sdk::build_client(config)
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error>)?;

    match sdk.api().verify_pak().send().await {
        Ok(_) => {
            info!("Successfully verified agent PAK");
            Ok(())
        }
        Err(raw) => {
            let wrapped = BrokkrError::from(raw);
            if status_u16(&wrapped) == Some(401) {
                error!("Agent PAK verification failed: unauthorized");
                Err("Invalid agent PAK".into())
            } else {
                error!("PAK verification failed: {}", wrapped);
                Err(boxed("PAK verification failed", wrapped))
            }
        }
    }
}

/// Fetches the details of the agent from the broker.
pub async fn fetch_agent_details(
    config: &Settings,
    client: &BrokkrClient,
) -> Result<Agent, Box<dyn std::error::Error>> {
    debug!(
        "Fetching agent details for name={} cluster={}",
        config.agent.agent_name, config.agent.cluster_name
    );

    let result = client
        .api()
        .search_agent()
        .name(&config.agent.agent_name)
        .cluster_name(&config.agent.cluster_name)
        .send()
        .await;

    match result {
        Ok(rv) => {
            let agent: Agent = convert(rv.into_inner()).map_err(|e| {
                error!("Failed to convert agent details: {}", e);
                Box::new(e) as Box<dyn std::error::Error>
            })?;
            info!(
                "Successfully fetched details for agent {} in cluster {}",
                agent.name, agent.cluster_name
            );
            Ok(agent)
        }
        Err(raw) => {
            let wrapped = BrokkrError::from(raw);
            if status_u16(&wrapped) == Some(404) {
                error!(
                    "Agent not found: name={}, cluster={}",
                    config.agent.agent_name, config.agent.cluster_name
                );
                Err("Agent not found".into())
            } else {
                error!("Failed to fetch agent details: {}", wrapped);
                Err(boxed("Failed to fetch agent details", wrapped))
            }
        }
    }
}

/// Fetches deployment objects to apply from the broker's target-state view.
pub async fn fetch_and_process_deployment_objects(
    _config: &Settings,
    client: &BrokkrClient,
    agent: &Agent,
) -> Result<Vec<DeploymentObject>, Box<dyn std::error::Error>> {
    debug!("Fetching target-state deployment objects for agent {}", agent.name);

    let start = Instant::now();
    let result = client.api().get_target_state().id(agent.id).send().await;
    let duration = start.elapsed().as_secs_f64();

    match result {
        Ok(rv) => {
            let objects: Vec<DeploymentObject> = convert(rv.into_inner()).map_err(|e| {
                error!("Failed to convert deployment objects: {}", e);
                metrics::poll_requests_total()
                    .with_label_values(&["error"])
                    .inc();
                metrics::poll_duration_seconds()
                    .with_label_values(&[])
                    .observe(duration);
                Box::new(e) as Box<dyn std::error::Error>
            })?;
            info!(
                "Successfully fetched {} deployment objects for agent {}",
                objects.len(),
                agent.name
            );
            metrics::poll_requests_total()
                .with_label_values(&["success"])
                .inc();
            metrics::poll_duration_seconds()
                .with_label_values(&[])
                .observe(duration);
            Ok(objects)
        }
        Err(raw) => {
            let wrapped = BrokkrError::from(raw);
            error!("Failed to fetch deployment objects: {}", wrapped);
            metrics::poll_requests_total()
                .with_label_values(&["error"])
                .inc();
            metrics::poll_duration_seconds()
                .with_label_values(&[])
                .observe(duration);
            Err(boxed("Failed to fetch deployment objects", wrapped))
        }
    }
}

/// Sends a success event to the broker for the given deployment object.
pub async fn send_success_event(
    _config: &Settings,
    client: &BrokkrClient,
    agent: &Agent,
    deployment_object_id: Uuid,
    message: Option<String>,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!(
        "Sending success event for deployment {} for agent {}",
        deployment_object_id, agent.name
    );

    let event = NewAgentEvent {
        agent_id: agent.id,
        deployment_object_id,
        event_type: "DEPLOY".to_string(),
        status: "SUCCESS".to_string(),
        message,
    };
    let sdk_event: brokkr_client::types::NewAgentEvent = convert(event).map_err(|e| {
        error!("Failed to convert NewAgentEvent: {}", e);
        Box::new(e) as Box<dyn std::error::Error>
    })?;

    match client
        .api()
        .create_event()
        .id(agent.id)
        .body(sdk_event)
        .send()
        .await
    {
        Ok(_) => {
            info!(
                "Successfully reported deployment success for object {}",
                deployment_object_id
            );
            Ok(())
        }
        Err(raw) => {
            let wrapped = BrokkrError::from(raw);
            error!("Failed to send success event: {}", wrapped);
            Err(boxed("Failed to send success event", wrapped))
        }
    }
}

/// Sends a failure event to the broker for the given deployment object.
pub async fn send_failure_event(
    _config: &Settings,
    client: &BrokkrClient,
    agent: &Agent,
    deployment_object_id: Uuid,
    error_message: String,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!(
        "Sending failure event for deployment {} for agent {}",
        deployment_object_id, agent.name
    );

    let event = NewAgentEvent {
        agent_id: agent.id,
        deployment_object_id,
        event_type: "DEPLOY".to_string(),
        status: "FAILURE".to_string(),
        message: Some(error_message),
    };
    let sdk_event: brokkr_client::types::NewAgentEvent = convert(event).map_err(|e| {
        error!("Failed to convert NewAgentEvent: {}", e);
        Box::new(e) as Box<dyn std::error::Error>
    })?;

    match client
        .api()
        .create_event()
        .id(agent.id)
        .body(sdk_event)
        .send()
        .await
    {
        Ok(_) => {
            info!(
                "Successfully reported deployment failure for object {}",
                deployment_object_id
            );
            Ok(())
        }
        Err(raw) => {
            let wrapped = BrokkrError::from(raw);
            error!(
                "Failed to send failure event for deployment {}: {}",
                deployment_object_id, wrapped
            );
            Err(boxed("Failed to send failure event", wrapped))
        }
    }
}

/// Sends a heartbeat to the broker for the given agent.
pub async fn send_heartbeat(
    _config: &Settings,
    client: &BrokkrClient,
    agent: &Agent,
) -> Result<(), Box<dyn std::error::Error>> {
    match client.api().record_heartbeat().id(agent.id).send().await {
        Ok(_) => {
            trace!("Heartbeat sent successfully for agent {}", agent.name);
            metrics::heartbeat_sent_total().inc();
            metrics::last_successful_poll_timestamp().set(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs_f64(),
            );
            Ok(())
        }
        Err(raw) => {
            let wrapped = BrokkrError::from(raw);
            metrics::heartbeat_sent_total().inc();
            if status_u16(&wrapped) == Some(401) {
                error!("Heartbeat unauthorized for agent {}", agent.name);
                Err("Unauthorized: Invalid agent PAK".into())
            } else {
                error!(
                    "Heartbeat failed for agent {}: {}",
                    agent.name, wrapped
                );
                Err(boxed("Heartbeat failed", wrapped))
            }
        }
    }
}

/// Sends health status updates for deployment objects to the broker.
pub async fn send_health_status(
    _config: &Settings,
    client: &BrokkrClient,
    agent: &Agent,
    health_updates: Vec<DeploymentObjectHealthUpdate>,
) -> Result<(), Box<dyn std::error::Error>> {
    if health_updates.is_empty() {
        return Ok(());
    }

    debug!(
        "Sending health status update for {} deployment objects for agent {}",
        health_updates.len(),
        agent.name
    );

    let update = HealthStatusUpdate {
        deployment_objects: health_updates,
    };
    let count = update.deployment_objects.len();
    let sdk_update: brokkr_client::types::HealthStatusUpdate = convert(update).map_err(|e| {
        error!("Failed to convert HealthStatusUpdate: {}", e);
        Box::new(e) as Box<dyn std::error::Error>
    })?;

    match client
        .api()
        .update_health_status()
        .id(agent.id)
        .body(sdk_update)
        .send()
        .await
    {
        Ok(_) => {
            debug!(
                "Successfully sent health status for {} deployment objects",
                count
            );
            Ok(())
        }
        Err(raw) => {
            let wrapped = BrokkrError::from(raw);
            if status_u16(&wrapped) == Some(401) {
                error!("Health status update unauthorized for agent {}", agent.name);
                Err("Unauthorized: Invalid agent PAK".into())
            } else {
                error!(
                    "Health status update failed for agent {}: {}",
                    agent.name, wrapped
                );
                Err(boxed("Health status update failed", wrapped))
            }
        }
    }
}

/// Fetches pending diagnostic requests for the agent.
pub async fn fetch_pending_diagnostics(
    _config: &Settings,
    client: &BrokkrClient,
    agent: &Agent,
) -> Result<Vec<DiagnosticRequest>, Box<dyn std::error::Error>> {
    debug!("Fetching pending diagnostics for agent {}", agent.name);

    match client
        .api()
        .get_pending_diagnostics()
        .id(agent.id)
        .send()
        .await
    {
        Ok(rv) => {
            let requests: Vec<DiagnosticRequest> = convert(rv.into_inner()).map_err(|e| {
                error!("Failed to convert diagnostic requests: {}", e);
                Box::new(e) as Box<dyn std::error::Error>
            })?;
            if !requests.is_empty() {
                debug!(
                    "Found {} pending diagnostic requests for agent {}",
                    requests.len(),
                    agent.name
                );
            }
            Ok(requests)
        }
        Err(raw) => {
            let wrapped = BrokkrError::from(raw);
            error!("Failed to fetch pending diagnostics: {}", wrapped);
            Err(boxed("Failed to fetch pending diagnostics", wrapped))
        }
    }
}

/// Claims a diagnostic request for processing.
pub async fn claim_diagnostic_request(
    _config: &Settings,
    client: &BrokkrClient,
    request_id: Uuid,
) -> Result<DiagnosticRequest, Box<dyn std::error::Error>> {
    debug!("Claiming diagnostic request {}", request_id);

    match client.api().claim_diagnostic().id(request_id).send().await {
        Ok(rv) => {
            let request: DiagnosticRequest = convert(rv.into_inner()).map_err(|e| {
                error!("Failed to convert claimed diagnostic request: {}", e);
                Box::new(e) as Box<dyn std::error::Error>
            })?;
            info!("Successfully claimed diagnostic request {}", request_id);
            Ok(request)
        }
        Err(raw) => {
            let wrapped = BrokkrError::from(raw);
            if status_u16(&wrapped) == Some(409) {
                warn!(
                    "Diagnostic request {} already claimed or completed",
                    request_id
                );
                Err(format!(
                    "Diagnostic request {request_id} already claimed or completed"
                )
                .into())
            } else {
                error!(
                    "Failed to claim diagnostic request {}: {}",
                    request_id, wrapped
                );
                Err(boxed("Failed to claim diagnostic request", wrapped))
            }
        }
    }
}

/// Submits diagnostic results for a request.
pub async fn submit_diagnostic_result(
    _config: &Settings,
    client: &BrokkrClient,
    request_id: Uuid,
    result: SubmitDiagnosticResult,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("Submitting diagnostic result for request {}", request_id);

    let sdk_result: brokkr_client::types::SubmitDiagnosticResult =
        convert(result).map_err(|e| {
            error!("Failed to convert SubmitDiagnosticResult: {}", e);
            Box::new(e) as Box<dyn std::error::Error>
        })?;

    match client
        .api()
        .submit_diagnostic_result()
        .id(request_id)
        .body(sdk_result)
        .send()
        .await
    {
        Ok(_) => {
            info!(
                "Successfully submitted diagnostic result for request {}",
                request_id
            );
            Ok(())
        }
        Err(raw) => {
            let wrapped = BrokkrError::from(raw);
            error!(
                "Failed to submit diagnostic result for request {}: {}",
                request_id, wrapped
            );
            Err(boxed("Failed to submit diagnostic result", wrapped))
        }
    }
}
