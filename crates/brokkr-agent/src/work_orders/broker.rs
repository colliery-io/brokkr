/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Broker communication for work order operations.
//!
//! This module handles all HTTP communication with the broker for work orders:
//! - Fetching pending work orders
//! - Claiming work orders
//! - Reporting completion (success/failure)

use brokkr_models::models::agents::Agent;
use brokkr_models::models::work_orders::WorkOrder;
use brokkr_utils::config::Settings;
use brokkr_utils::logging::prelude::*;
use reqwest::{Client, StatusCode};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Request body for claiming a work order.
#[derive(Debug, Serialize)]
struct ClaimRequest {
    agent_id: Uuid,
}

/// Request body for completing a work order.
#[derive(Debug, Serialize)]
struct CompleteRequest {
    success: bool,
    message: Option<String>,
    /// Whether the error is retryable. Only meaningful when success=false.
    /// If false, the broker will immediately fail the work order without retry.
    retryable: bool,
}

/// Response for retry scheduling.
#[derive(Debug, Deserialize)]
struct RetryResponse {
    status: String,
}

/// Fetches pending work orders for the agent from the broker.
///
/// # Arguments
/// * `config` - Application settings
/// * `client` - HTTP client
/// * `agent` - Agent details
/// * `work_type` - Optional filter by work type
///
/// # Returns
/// Vector of pending work orders that can be claimed by this agent
pub async fn fetch_pending_work_orders(
    config: &Settings,
    client: &Client,
    agent: &Agent,
    work_type: Option<&str>,
) -> Result<Vec<WorkOrder>, Box<dyn std::error::Error>> {
    let mut url = format!(
        "{}/api/v1/agents/{}/work-orders/pending",
        config.agent.broker_url, agent.id
    );

    if let Some(wt) = work_type {
        url.push_str(&format!("?work_type={}", wt));
    }

    debug!("Fetching pending work orders from {}", url);

    let response = client
        .get(&url)
        .header("Authorization", format!("Bearer {}", config.agent.pak))
        .send()
        .await
        .map_err(|e| {
            error!("Failed to fetch pending work orders: {}", e);
            Box::new(e) as Box<dyn std::error::Error>
        })?;

    match response.status() {
        StatusCode::OK => {
            let work_orders: Vec<WorkOrder> = response.json().await.map_err(|e| {
                error!("Failed to deserialize work orders: {}", e);
                Box::new(e) as Box<dyn std::error::Error>
            })?;

            debug!(
                "Successfully fetched {} pending work orders for agent {}",
                work_orders.len(),
                agent.name
            );

            Ok(work_orders)
        }
        StatusCode::FORBIDDEN => {
            error!("Access denied when fetching pending work orders for agent {}", agent.id);
            Err("Access denied".into())
        }
        status => {
            let error_body = response.text().await.unwrap_or_default();
            error!(
                "Failed to fetch pending work orders. Status {}: {}",
                status, error_body
            );
            Err(format!(
                "Failed to fetch pending work orders. Status: {}, Body: {}",
                status, error_body
            )
            .into())
        }
    }
}

/// Claims a work order for the agent.
///
/// # Arguments
/// * `config` - Application settings
/// * `client` - HTTP client
/// * `agent` - Agent details
/// * `work_order_id` - ID of the work order to claim
///
/// # Returns
/// The claimed work order with updated status
pub async fn claim_work_order(
    config: &Settings,
    client: &Client,
    agent: &Agent,
    work_order_id: Uuid,
) -> Result<WorkOrder, Box<dyn std::error::Error>> {
    let url = format!(
        "{}/api/v1/work-orders/{}/claim",
        config.agent.broker_url, work_order_id
    );

    debug!("Claiming work order {} at {}", work_order_id, url);

    let request = ClaimRequest { agent_id: agent.id };

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", config.agent.pak))
        .json(&request)
        .send()
        .await
        .map_err(|e| {
            error!("Failed to claim work order {}: {}", work_order_id, e);
            Box::new(e) as Box<dyn std::error::Error>
        })?;

    match response.status() {
        StatusCode::OK => {
            let work_order: WorkOrder = response.json().await.map_err(|e| {
                error!("Failed to deserialize claimed work order: {}", e);
                Box::new(e) as Box<dyn std::error::Error>
            })?;

            info!(
                "Successfully claimed work order {} for agent {}",
                work_order_id, agent.name
            );

            Ok(work_order)
        }
        StatusCode::NOT_FOUND => {
            warn!(
                "Work order {} not found or not claimable by agent {}",
                work_order_id, agent.id
            );
            Err("Work order not found or not claimable".into())
        }
        StatusCode::CONFLICT => {
            warn!("Work order {} already claimed", work_order_id);
            Err("Work order already claimed".into())
        }
        StatusCode::FORBIDDEN => {
            error!(
                "Access denied when claiming work order {} for agent {}",
                work_order_id, agent.id
            );
            Err("Access denied".into())
        }
        status => {
            let error_body = response.text().await.unwrap_or_default();
            error!(
                "Failed to claim work order {}. Status {}: {}",
                work_order_id, status, error_body
            );
            Err(format!(
                "Failed to claim work order. Status: {}, Body: {}",
                status, error_body
            )
            .into())
        }
    }
}

/// Reports work order completion to the broker.
///
/// # Arguments
/// * `config` - Application settings
/// * `client` - HTTP client
/// * `work_order_id` - ID of the work order
/// * `success` - Whether the work completed successfully
/// * `message` - Optional result message (image digest on success, error on failure)
/// * `retryable` - Whether a failure is retryable (ignored on success)
///
/// # Returns
/// Ok(()) on success, Err on failure
pub async fn complete_work_order(
    config: &Settings,
    client: &Client,
    work_order_id: Uuid,
    success: bool,
    message: Option<String>,
    retryable: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    let url = format!(
        "{}/api/v1/work-orders/{}/complete",
        config.agent.broker_url, work_order_id
    );

    debug!(
        "Completing work order {} (success: {}, retryable: {}) at {}",
        work_order_id, success, retryable, url
    );

    let request = CompleteRequest {
        success,
        message,
        retryable,
    };

    let response = client
        .post(&url)
        .header("Authorization", format!("Bearer {}", config.agent.pak))
        .json(&request)
        .send()
        .await
        .map_err(|e| {
            error!("Failed to complete work order {}: {}", work_order_id, e);
            Box::new(e) as Box<dyn std::error::Error>
        })?;

    match response.status() {
        StatusCode::OK => {
            info!(
                "Successfully reported work order {} completion (success: {})",
                work_order_id, success
            );
            Ok(())
        }
        StatusCode::ACCEPTED => {
            // Work order scheduled for retry
            info!(
                "Work order {} scheduled for retry after failure",
                work_order_id
            );
            Ok(())
        }
        StatusCode::NOT_FOUND => {
            warn!("Work order {} not found when reporting completion", work_order_id);
            Err("Work order not found".into())
        }
        StatusCode::FORBIDDEN => {
            error!(
                "Access denied when completing work order {}",
                work_order_id
            );
            Err("Access denied".into())
        }
        status => {
            let error_body = response.text().await.unwrap_or_default();
            error!(
                "Failed to complete work order {}. Status {}: {}",
                work_order_id, status, error_body
            );
            Err(format!(
                "Failed to complete work order. Status: {}, Body: {}",
                status, error_body
            )
            .into())
        }
    }
}
