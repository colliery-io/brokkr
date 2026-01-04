/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Diagnostic Result model for storing collected diagnostic data.
//!
//! Diagnostic results contain the pod statuses, events, and log tails
//! collected by agents in response to diagnostic requests.

use crate::schema::diagnostic_results;
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// A diagnostic result record from the database.
#[derive(
    Debug, Clone, Queryable, Selectable, Identifiable, Serialize, Deserialize, ToSchema,
)]
#[diesel(table_name = diagnostic_results)]
pub struct DiagnosticResult {
    /// Unique identifier for the diagnostic result.
    pub id: Uuid,
    /// The diagnostic request this result belongs to.
    pub request_id: Uuid,
    /// JSON-encoded pod statuses.
    pub pod_statuses: String,
    /// JSON-encoded Kubernetes events.
    pub events: String,
    /// JSON-encoded log tails (optional).
    pub log_tails: Option<String>,
    /// When the diagnostics were collected by the agent.
    pub collected_at: DateTime<Utc>,
    /// When the result was created in the database.
    pub created_at: DateTime<Utc>,
}

/// A new diagnostic result to be inserted.
#[derive(Debug, Clone, Insertable, Serialize, Deserialize)]
#[diesel(table_name = diagnostic_results)]
pub struct NewDiagnosticResult {
    /// The diagnostic request this result belongs to.
    pub request_id: Uuid,
    /// JSON-encoded pod statuses.
    pub pod_statuses: String,
    /// JSON-encoded Kubernetes events.
    pub events: String,
    /// JSON-encoded log tails (optional).
    pub log_tails: Option<String>,
    /// When the diagnostics were collected by the agent.
    pub collected_at: DateTime<Utc>,
}

impl NewDiagnosticResult {
    /// Creates a new diagnostic result.
    ///
    /// # Arguments
    /// * `request_id` - The diagnostic request this result belongs to.
    /// * `pod_statuses` - JSON-encoded pod statuses.
    /// * `events` - JSON-encoded Kubernetes events.
    /// * `log_tails` - Optional JSON-encoded log tails.
    /// * `collected_at` - When the diagnostics were collected.
    ///
    /// # Returns
    /// A Result containing the new diagnostic result or an error.
    pub fn new(
        request_id: Uuid,
        pod_statuses: String,
        events: String,
        log_tails: Option<String>,
        collected_at: DateTime<Utc>,
    ) -> Result<Self, String> {
        // Validate request_id is not nil
        if request_id.is_nil() {
            return Err("Request ID cannot be nil".to_string());
        }

        // Validate pod_statuses is not empty
        if pod_statuses.is_empty() {
            return Err("Pod statuses cannot be empty".to_string());
        }

        // Validate events is not empty
        if events.is_empty() {
            return Err("Events cannot be empty".to_string());
        }

        Ok(Self {
            request_id,
            pod_statuses,
            events,
            log_tails,
            collected_at,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_diagnostic_result_success() {
        let request_id = Uuid::new_v4();
        let collected_at = Utc::now();

        let result = NewDiagnosticResult::new(
            request_id,
            r#"[{"name": "pod-1", "status": "Running"}]"#.to_string(),
            r#"[{"type": "Normal", "reason": "Started"}]"#.to_string(),
            Some(r#"{"pod-1": "log content..."}"#.to_string()),
            collected_at,
        );

        assert!(result.is_ok());
        let diagnostic = result.unwrap();
        assert_eq!(diagnostic.request_id, request_id);
        assert!(!diagnostic.pod_statuses.is_empty());
        assert!(!diagnostic.events.is_empty());
        assert!(diagnostic.log_tails.is_some());
        assert_eq!(diagnostic.collected_at, collected_at);
    }

    #[test]
    fn test_new_diagnostic_result_nil_request_id() {
        let result = NewDiagnosticResult::new(
            Uuid::nil(),
            r#"[{"name": "pod-1"}]"#.to_string(),
            r#"[]"#.to_string(),
            None,
            Utc::now(),
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Request ID cannot be nil");
    }

    #[test]
    fn test_new_diagnostic_result_empty_pod_statuses() {
        let result = NewDiagnosticResult::new(
            Uuid::new_v4(),
            "".to_string(),
            r#"[]"#.to_string(),
            None,
            Utc::now(),
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Pod statuses cannot be empty");
    }

    #[test]
    fn test_new_diagnostic_result_empty_events() {
        let result = NewDiagnosticResult::new(
            Uuid::new_v4(),
            r#"[]"#.to_string(),
            "".to_string(),
            None,
            Utc::now(),
        );

        assert!(result.is_err());
        assert_eq!(result.unwrap_err(), "Events cannot be empty");
    }

    #[test]
    fn test_new_diagnostic_result_no_log_tails() {
        let request_id = Uuid::new_v4();

        let result = NewDiagnosticResult::new(
            request_id,
            r#"[]"#.to_string(),
            r#"[]"#.to_string(),
            None,
            Utc::now(),
        );

        assert!(result.is_ok());
        let diagnostic = result.unwrap();
        assert!(diagnostic.log_tails.is_none());
    }
}
