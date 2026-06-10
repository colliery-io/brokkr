/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Broker communication for work order operations.
//!
//! Migrated to use [`brokkr_client::BrokkrClient`] in T-D1. The
//! 202 "retry scheduled" response is documented inline because the SDK only
//! types the 200 success path (T-A1 carry-over).

use brokkr_client::{BrokkrClient, BrokkrError};
use brokkr_models::models::agents::Agent;
use brokkr_models::models::work_orders::WorkOrder;
use brokkr_utils::config::Settings;
use serde::de::DeserializeOwned;
use serde::Serialize;
use tracing::{debug, error, info, warn};
use uuid::Uuid;

fn status_u16(err: &BrokkrError) -> Option<u16> {
    err.status().map(|s| s.as_u16())
}

fn convert<F: Serialize, T: DeserializeOwned>(value: F) -> Result<T, serde_json::Error> {
    let v = serde_json::to_value(value)?;
    serde_json::from_value(v)
}

fn boxed(prefix: &str, err: BrokkrError) -> Box<dyn std::error::Error> {
    let msg = match status_u16(&err) {
        Some(s) => format!("{prefix}. Status: {s}, Error: {err}"),
        None => format!("{prefix}: {err}"),
    };
    msg.into()
}

/// Fetches pending work orders for the agent from the broker.
pub async fn fetch_pending_work_orders(
    _config: &Settings,
    client: &BrokkrClient,
    agent: &Agent,
    work_type: Option<&str>,
) -> Result<Vec<WorkOrder>, Box<dyn std::error::Error>> {
    debug!(
        "Fetching pending work orders for agent {} (work_type={:?})",
        agent.name, work_type
    );

    let mut builder = client.api().list_pending_for_agent().agent_id(agent.id);
    if let Some(wt) = work_type {
        builder = builder.work_type(wt);
    }

    match builder.send().await {
        Ok(rv) => {
            let orders: Vec<WorkOrder> = convert(rv.into_inner()).map_err(|e| {
                error!("Failed to convert work orders: {}", e);
                Box::new(e) as Box<dyn std::error::Error>
            })?;
            debug!(
                "Successfully fetched {} pending work orders for agent {}",
                orders.len(),
                agent.name
            );
            Ok(orders)
        }
        Err(raw) => {
            let wrapped = BrokkrError::from(raw);
            if status_u16(&wrapped) == Some(403) {
                error!(
                    "Access denied when fetching pending work orders for agent {}",
                    agent.id
                );
                Err("Access denied".into())
            } else {
                error!("Failed to fetch pending work orders: {}", wrapped);
                Err(boxed("Failed to fetch pending work orders", wrapped))
            }
        }
    }
}

/// Claims a work order for the agent.
pub async fn claim_work_order(
    _config: &Settings,
    client: &BrokkrClient,
    agent: &Agent,
    work_order_id: Uuid,
) -> Result<WorkOrder, Box<dyn std::error::Error>> {
    debug!(
        "Claiming work order {} for agent {}",
        work_order_id, agent.name
    );

    let body = brokkr_client::types::ClaimWorkOrderRequest { agent_id: agent.id };

    match client
        .api()
        .claim_work_order()
        .id(work_order_id)
        .body(body)
        .send()
        .await
    {
        Ok(rv) => {
            let order: WorkOrder = convert(rv.into_inner()).map_err(|e| {
                error!("Failed to convert claimed work order: {}", e);
                Box::new(e) as Box<dyn std::error::Error>
            })?;
            info!(
                "Successfully claimed work order {} for agent {}",
                work_order_id, agent.name
            );
            Ok(order)
        }
        Err(raw) => {
            let wrapped = BrokkrError::from(raw);
            match status_u16(&wrapped) {
                Some(404) => {
                    warn!(
                        "Work order {} not found or not claimable by agent {}",
                        work_order_id, agent.id
                    );
                    Err("Work order not found or not claimable".into())
                }
                Some(409) => {
                    warn!("Work order {} already claimed", work_order_id);
                    Err("Work order already claimed".into())
                }
                Some(403) => {
                    error!(
                        "Access denied when claiming work order {} for agent {}",
                        work_order_id, agent.id
                    );
                    Err("Access denied".into())
                }
                _ => {
                    error!("Failed to claim work order {}: {}", work_order_id, wrapped);
                    Err(boxed("Failed to claim work order", wrapped))
                }
            }
        }
    }
}

/// Reports work order completion to the broker.
///
/// The broker returns 200 on success (or final failure logged to log table)
/// and 202 when a failure is retryable and a retry is scheduled. The OpenAPI
/// spec only types the 200 case (T-A1 carry-over), so the generated SDK
/// surfaces the 202 case as `UnexpectedResponse`. We unwrap that here and
/// treat it as success.
pub async fn complete_work_order(
    _config: &Settings,
    client: &BrokkrClient,
    work_order_id: Uuid,
    success: bool,
    message: Option<String>,
    retryable: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!(
        "Completing work order {} (success: {}, retryable: {})",
        work_order_id, success, retryable
    );

    let body = brokkr_client::types::CompleteWorkOrderRequest {
        success,
        message,
        retryable: Some(retryable),
    };

    match client
        .api()
        .complete_work_order()
        .id(work_order_id)
        .body(body)
        .send()
        .await
    {
        Ok(_) => {
            info!(
                "Successfully reported work order {} completion (success: {})",
                work_order_id, success
            );
            Ok(())
        }
        Err(raw) => {
            let wrapped = BrokkrError::from(raw);
            match status_u16(&wrapped) {
                // 202: failed-but-retryable. Spec only types 200; SDK surfaces
                // this as UnexpectedResponse. Treat as success.
                Some(202) => {
                    info!(
                        "Work order {} scheduled for retry after failure",
                        work_order_id
                    );
                    Ok(())
                }
                Some(404) => {
                    warn!(
                        "Work order {} not found when reporting completion",
                        work_order_id
                    );
                    Err("Work order not found".into())
                }
                Some(403) => {
                    error!("Access denied when completing work order {}", work_order_id);
                    Err("Access denied".into())
                }
                _ => {
                    error!(
                        "Failed to complete work order {}: {}",
                        work_order_id, wrapped
                    );
                    Err(boxed("Failed to complete work order", wrapped))
                }
            }
        }
    }
}
