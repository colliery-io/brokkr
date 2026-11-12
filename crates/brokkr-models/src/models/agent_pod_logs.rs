/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Persisted pod log lines for managed workloads.
//!
//! Bounded by the 6h retention ceiling (`project_log_retention_stance`).
//! Per-stack opt-in is enforced agent-side (WS-08); the broker accepts
//! whatever the agent streams and the eviction worker keeps growth bounded.

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Queryable, Selectable, Identifiable, Debug, Clone, Serialize, Deserialize, ToSchema)]
#[diesel(table_name = crate::schema::agent_pod_logs)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AgentPodLog {
    pub id: Uuid,
    pub agent_id: Uuid,
    /// The stack the row belongs to. No value: it belongs to a work order.
    pub stack_id: Option<Uuid>,
    pub namespace: String,
    pub pod: String,
    pub container: String,
    pub ts: DateTime<Utc>,
    pub line: String,
    pub created_at: DateTime<Utc>,
    /// The work order the row belongs to. No value: it belongs to a stack.
    /// Exactly one of `stack_id` and `work_order_id` has a value (BROKKR-T-0405).
    #[serde(default)]
    pub work_order_id: Option<Uuid>,
}

#[derive(Insertable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::agent_pod_logs)]
pub struct NewAgentPodLog {
    pub agent_id: Uuid,
    pub stack_id: Option<Uuid>,
    pub work_order_id: Option<Uuid>,
    pub namespace: String,
    pub pod: String,
    pub container: String,
    pub ts: DateTime<Utc>,
    pub line: String,
}
