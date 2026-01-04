/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Webhook delivery module for agent-side webhook processing.
//!
//! This module provides functionality for agents to poll for pending webhooks
//! assigned to them, deliver them via HTTP, and report results back to the broker.

use brokkr_models::models::agents::Agent;
use brokkr_utils::Settings;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};
use uuid::Uuid;

// =============================================================================
// Types matching broker API
// =============================================================================

/// Pending webhook delivery from the broker.
/// Contains decrypted URL and auth header for delivery.
#[derive(Debug, Clone, Deserialize)]
pub struct PendingWebhookDelivery {
    /// Delivery ID.
    pub id: Uuid,
    /// Subscription ID.
    pub subscription_id: Uuid,
    /// Event type being delivered.
    pub event_type: String,
    /// JSON-encoded event payload.
    pub payload: String,
    /// Decrypted webhook URL.
    pub url: String,
    /// Decrypted Authorization header (if configured).
    pub auth_header: Option<String>,
    /// HTTP timeout in seconds.
    pub timeout_seconds: i32,
    /// Maximum retries for this subscription.
    pub max_retries: i32,
    /// Current attempt number.
    pub attempts: i32,
}

/// Request body for reporting delivery result to broker.
#[derive(Debug, Clone, Serialize)]
pub struct DeliveryResultRequest {
    /// Whether delivery succeeded.
    pub success: bool,
    /// HTTP status code (if available).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status_code: Option<i32>,
    /// Error message (if failed).
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    /// Delivery duration in milliseconds.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duration_ms: Option<i64>,
}

/// Result of a webhook delivery attempt.
#[derive(Debug)]
pub struct DeliveryResult {
    /// Whether delivery succeeded.
    pub success: bool,
    /// HTTP status code (if available).
    pub status_code: Option<i32>,
    /// Error message (if failed).
    pub error: Option<String>,
    /// Delivery duration in milliseconds.
    pub duration_ms: i64,
}

// =============================================================================
// Broker Communication
// =============================================================================

/// Fetches pending webhook deliveries for this agent from the broker.
///
/// # Arguments
/// * `config` - Application settings containing broker configuration
/// * `client` - HTTP client for making requests
/// * `agent` - Agent details
///
/// # Returns
/// Pending webhook deliveries or error
pub async fn fetch_pending_webhooks(
    config: &Settings,
    client: &Client,
    agent: &Agent,
) -> Result<Vec<PendingWebhookDelivery>, Box<dyn std::error::Error>> {
    let url = format!(
        "{}/api/v1/agents/{}/webhooks/pending",
        config.agent.broker_url, agent.id
    );

    debug!("Fetching pending webhooks from {}", url);

    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", config.agent.pak))
        .send()
        .await
        .map_err(|e| {
            error!("Failed to fetch pending webhooks: {}", e);
            Box::new(e) as Box<dyn std::error::Error>
        })?;

    match response.status() {
        StatusCode::OK => {
            let deliveries: Vec<PendingWebhookDelivery> = response.json().await.map_err(|e| {
                error!("Failed to deserialize pending webhooks: {}", e);
                Box::new(e) as Box<dyn std::error::Error>
            })?;

            if !deliveries.is_empty() {
                debug!(
                    "Fetched {} pending webhook deliveries for agent {}",
                    deliveries.len(),
                    agent.name
                );
            }

            Ok(deliveries)
        }
        status => {
            let error_body = response.text().await.unwrap_or_default();
            error!(
                "Failed to fetch pending webhooks. Status {}: {}",
                status, error_body
            );
            Err(format!(
                "Failed to fetch pending webhooks. Status: {}, Body: {}",
                status, error_body
            )
            .into())
        }
    }
}

/// Reports the result of a webhook delivery attempt to the broker.
///
/// # Arguments
/// * `config` - Application settings containing broker configuration
/// * `client` - HTTP client for making requests
/// * `delivery_id` - ID of the delivery being reported
/// * `result` - The delivery result
///
/// # Returns
/// Success or error
pub async fn report_delivery_result(
    config: &Settings,
    client: &Client,
    delivery_id: Uuid,
    result: &DeliveryResult,
) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!(
        "{}/api/v1/webhook-deliveries/{}/result",
        config.agent.broker_url, delivery_id
    );

    debug!("Reporting delivery result for {} to {}", delivery_id, url);

    let request_body = DeliveryResultRequest {
        success: result.success,
        status_code: result.status_code,
        error: result.error.clone(),
        duration_ms: Some(result.duration_ms),
    };

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", config.agent.pak))
        .json(&request_body)
        .send()
        .await
        .map_err(|e| {
            error!("Failed to report delivery result: {}", e);
            Box::new(e) as Box<dyn std::error::Error>
        })?;

    match response.status() {
        StatusCode::OK => {
            debug!("Successfully reported delivery result for {}", delivery_id);
            Ok(())
        }
        status => {
            let error_body = response.text().await.unwrap_or_default();
            error!(
                "Failed to report delivery result for {}. Status {}: {}",
                delivery_id, status, error_body
            );
            Err(format!(
                "Failed to report delivery result. Status: {}, Body: {}",
                status, error_body
            )
            .into())
        }
    }
}

// =============================================================================
// Webhook Delivery
// =============================================================================

/// Delivers a webhook via HTTP POST.
///
/// # Arguments
/// * `delivery` - The pending webhook delivery with URL and payload
///
/// # Returns
/// DeliveryResult with success/failure info and timing
pub async fn deliver_webhook(delivery: &PendingWebhookDelivery) -> DeliveryResult {
    let start = Instant::now();

    // Build HTTP client with timeout
    let client = match reqwest::Client::builder()
        .timeout(Duration::from_secs(delivery.timeout_seconds as u64))
        .build()
    {
        Ok(c) => c,
        Err(e) => {
            return DeliveryResult {
                success: false,
                status_code: None,
                error: Some(format!("Failed to create HTTP client: {}", e)),
                duration_ms: start.elapsed().as_millis() as i64,
            };
        }
    };

    // Build the request
    let mut request = client
        .post(&delivery.url)
        .header("Content-Type", "application/json")
        .header("X-Brokkr-Event-Type", &delivery.event_type)
        .header("X-Brokkr-Delivery-Id", delivery.id.to_string())
        .body(delivery.payload.clone());

    // Add authorization header if present
    if let Some(ref auth) = delivery.auth_header {
        request = request.header("Authorization", auth);
    }

    // Send the request
    match request.send().await {
        Ok(response) => {
            let status_code = response.status().as_u16() as i32;
            let duration_ms = start.elapsed().as_millis() as i64;

            if response.status().is_success() {
                debug!(
                    "Webhook delivery {} succeeded with status {} in {}ms",
                    delivery.id, status_code, duration_ms
                );
                DeliveryResult {
                    success: true,
                    status_code: Some(status_code),
                    error: None,
                    duration_ms,
                }
            } else {
                // Get error body for context (limit to 500 chars)
                let error_body = response
                    .text()
                    .await
                    .unwrap_or_else(|_| "Failed to read response body".to_string());
                let error_preview: String = error_body.chars().take(500).collect();

                warn!(
                    "Webhook delivery {} failed with status {}: {}",
                    delivery.id, status_code, error_preview
                );

                DeliveryResult {
                    success: false,
                    status_code: Some(status_code),
                    error: Some(format!("HTTP {}: {}", status_code, error_preview)),
                    duration_ms,
                }
            }
        }
        Err(e) => {
            let duration_ms = start.elapsed().as_millis() as i64;
            let error_msg = classify_error(&e);

            warn!(
                "Webhook delivery {} failed after {}ms: {}",
                delivery.id, duration_ms, error_msg
            );

            DeliveryResult {
                success: false,
                status_code: None,
                error: Some(error_msg),
                duration_ms,
            }
        }
    }
}

/// Classifies request errors for logging and retry decisions.
fn classify_error(error: &reqwest::Error) -> String {
    if error.is_timeout() {
        "Request timed out".to_string()
    } else if error.is_connect() {
        "Connection failed".to_string()
    } else if error.is_request() {
        format!("Request error: {}", error)
    } else {
        format!("Error: {}", error)
    }
}

// =============================================================================
// Main Processing
// =============================================================================

/// Processes all pending webhook deliveries for this agent.
///
/// This function:
/// 1. Fetches pending webhooks from the broker
/// 2. Delivers each webhook via HTTP
/// 3. Reports results back to the broker
///
/// # Arguments
/// * `config` - Application settings
/// * `client` - HTTP client for broker communication
/// * `agent` - Agent details
///
/// # Returns
/// Number of webhooks processed or error
pub async fn process_pending_webhooks(
    config: &Settings,
    client: &Client,
    agent: &Agent,
) -> Result<usize, Box<dyn std::error::Error>> {
    // Fetch pending deliveries from broker
    let deliveries = fetch_pending_webhooks(config, client, agent).await?;

    if deliveries.is_empty() {
        return Ok(0);
    }

    info!(
        "Processing {} pending webhook deliveries for agent {}",
        deliveries.len(),
        agent.name
    );

    let mut processed = 0;

    for delivery in deliveries {
        debug!(
            "Delivering webhook {} (event: {}, attempt: {})",
            delivery.id, delivery.event_type, delivery.attempts + 1
        );

        // Deliver the webhook
        let result = deliver_webhook(&delivery).await;

        // Report result to broker
        if let Err(e) = report_delivery_result(config, client, delivery.id, &result).await {
            error!(
                "Failed to report delivery result for {}: {}",
                delivery.id, e
            );
            // Continue processing other deliveries even if reporting fails
        }

        processed += 1;

        if result.success {
            info!(
                "Webhook delivery {} succeeded in {}ms",
                delivery.id, result.duration_ms
            );
        } else {
            warn!(
                "Webhook delivery {} failed: {:?}",
                delivery.id,
                result.error.as_deref().unwrap_or("unknown error")
            );
        }
    }

    Ok(processed)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_delivery_result_request_serialization() {
        let request = DeliveryResultRequest {
            success: true,
            status_code: Some(200),
            error: None,
            duration_ms: Some(150),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"success\":true"));
        assert!(json.contains("\"status_code\":200"));
        assert!(json.contains("\"duration_ms\":150"));
        // error should be omitted when None
        assert!(!json.contains("error"));
    }

    #[test]
    fn test_delivery_result_request_with_error() {
        let request = DeliveryResultRequest {
            success: false,
            status_code: Some(500),
            error: Some("Internal server error".to_string()),
            duration_ms: Some(50),
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("\"success\":false"));
        assert!(json.contains("\"error\":\"Internal server error\""));
    }

    #[test]
    fn test_pending_webhook_delivery_deserialization() {
        let json = r#"{
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "subscription_id": "550e8400-e29b-41d4-a716-446655440001",
            "event_type": "health.degraded",
            "payload": "{\"status\": \"degraded\"}",
            "url": "https://example.com/webhook",
            "auth_header": "Bearer secret",
            "timeout_seconds": 30,
            "max_retries": 5,
            "attempts": 0
        }"#;

        let delivery: PendingWebhookDelivery = serde_json::from_str(json).unwrap();
        assert_eq!(delivery.event_type, "health.degraded");
        assert_eq!(delivery.url, "https://example.com/webhook");
        assert_eq!(delivery.auth_header, Some("Bearer secret".to_string()));
        assert_eq!(delivery.timeout_seconds, 30);
        assert_eq!(delivery.attempts, 0);
    }

    #[test]
    fn test_pending_webhook_delivery_without_auth() {
        let json = r#"{
            "id": "550e8400-e29b-41d4-a716-446655440000",
            "subscription_id": "550e8400-e29b-41d4-a716-446655440001",
            "event_type": "agent.offline",
            "payload": "{}",
            "url": "https://example.com/webhook",
            "auth_header": null,
            "timeout_seconds": 10,
            "max_retries": 3,
            "attempts": 2
        }"#;

        let delivery: PendingWebhookDelivery = serde_json::from_str(json).unwrap();
        assert_eq!(delivery.auth_header, None);
        assert_eq!(delivery.attempts, 2);
    }
}
