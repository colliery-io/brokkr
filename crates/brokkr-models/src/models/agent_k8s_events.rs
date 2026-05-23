/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Persisted kube Events for objects an agent manages.
//!
//! Short-lived operational buffer with a hard 6-hour retention ceiling
//! enforced by the broker's eviction worker (see `brokkr-broker`).
//! See [[BROKKR-I-0019]] and `project_log_retention_stance`.

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Queryable, Selectable, Identifiable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::agent_k8s_events)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AgentK8sEvent {
    pub id: Uuid,
    pub agent_id: Uuid,
    pub stack_id: Uuid,
    pub observed_at: DateTime<Utc>,
    pub reason: String,
    pub message: String,
    pub event_type: String,
    pub source: Option<String>,
    pub involved_object: serde_json::Value,
    pub created_at: DateTime<Utc>,
}

#[derive(Insertable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::agent_k8s_events)]
pub struct NewAgentK8sEvent {
    pub agent_id: Uuid,
    pub stack_id: Uuid,
    pub observed_at: DateTime<Utc>,
    pub reason: String,
    pub message: String,
    pub event_type: String,
    pub source: Option<String>,
    pub involved_object: serde_json::Value,
}
