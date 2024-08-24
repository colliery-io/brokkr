//! # Agent Events
//!
//! This module defines the data structures and operations for agent events in the Brokkr system.
//!
//! ## Core Data Model
//!
//! The core data model for agent events is represented by the `AgentEvent` struct:
//!
//! - `id`: Uuid - Unique identifier for the agent event
//! - `created_at`: DateTime<Utc> - Timestamp when the event was created
//! - `updated_at`: DateTime<Utc> - Timestamp when the event was last updated
//! - `deleted_at`: Option<DateTime<Utc>> - Timestamp when the event was soft-deleted (if applicable)
//! - `agent_id`: Uuid - UUID of the agent associated with this event
//! - `deployment_object_id`: Uuid - UUID of the deployment object associated with this event
//! - `event_type`: String - Type of the event (max 50 characters)
//! - `status`: String - Status of the event (max 10 characters)
//! - `message`: Option<String> - Optional message providing additional details about the event

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::schema::agent_events;

/// Represents an agent event in the system.
///
/// This struct is used for querying existing agent events from the database.
#[derive(Queryable, Selectable, Identifiable, Debug, Clone, Serialize, Deserialize, AsChangeset)]
#[diesel(table_name = agent_events)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AgentEvent {
    /// Unique identifier for the agent event
    pub id: Uuid,
    /// Timestamp when the event was created
    pub created_at: DateTime<Utc>,
    /// Timestamp when the event was last updated
    pub updated_at: DateTime<Utc>,
    /// Timestamp when the event was soft-deleted (if applicable)
    pub deleted_at: Option<DateTime<Utc>>,
    /// UUID of the agent associated with this event
    pub agent_id: Uuid,
    /// UUID of the deployment object associated with this event
    pub deployment_object_id: Uuid,
    /// Type of the event (max 50 characters)
    pub event_type: String,
    /// Status of the event (max 10 characters)
    pub status: String,
    /// Optional message providing additional details about the event
    pub message: Option<String>,
}

/// Represents a new agent event to be inserted into the database.
///
/// This struct is used when creating new agent events.
#[derive(Insertable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = agent_events)]
pub struct NewAgentEvent {
    /// UUID of the agent associated with this event
    pub agent_id: Uuid,
    /// UUID of the deployment object associated with this event
    pub deployment_object_id: Uuid,
    /// Type of the event (max 50 characters)
    pub event_type: String,
    /// Status of the event (max 10 characters)
    pub status: String,
    /// Optional message providing additional details about the event
    pub message: Option<String>,
}

impl NewAgentEvent {
    /// Creates a new `NewAgentEvent` instance.
    ///
    /// # Arguments
    ///
    /// * `agent_id` - UUID of the agent associated with this event
    /// * `deployment_object_id` - UUID of the deployment object associated with this event
    /// * `event_type` - Type of the event (max 50 characters)
    /// * `status` - Status of the event (max 10 characters)
    /// * `message` - Optional message providing additional details about the event
    ///
    /// # Returns
    ///
    /// A Result containing a new `NewAgentEvent` instance if successful, or an error message if validation fails.
    pub fn new(
        agent_id: Uuid,
        deployment_object_id: Uuid,
        event_type: String,
        status: String,
        message: Option<String>,
    ) -> Result<Self, String> {
        if event_type.len() > 50 {
            return Err("Event type cannot exceed 50 characters".to_string());
        }
        if status.len() > 10 {
            return Err("Status cannot exceed 10 characters".to_string());
        }

        Ok(NewAgentEvent {
            agent_id,
            deployment_object_id,
            event_type,
            status,
            message,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// Tests the successful creation of a NewAgentEvent with valid input parameters.
    fn test_new_agent_event_success() {
        let agent_id = Uuid::new_v4();
        let deployment_object_id = Uuid::new_v4();
        let event_type = "test_event".to_string();
        let status = "success".to_string();
        let message = Some("Test message".to_string());

        let result = NewAgentEvent::new(
            agent_id,
            deployment_object_id,
            event_type.clone(),
            status.clone(),
            message.clone(),
        );

        assert!(result.is_ok(), "NewAgentEvent creation should succeed with valid inputs");
        let new_event = result.unwrap();
        assert_eq!(new_event.agent_id, agent_id, "agent_id should match the input value");
        assert_eq!(new_event.deployment_object_id, deployment_object_id, "deployment_object_id should match the input value");
        assert_eq!(new_event.event_type, event_type, "event_type should match the input value");
        assert_eq!(new_event.status, status, "status should match the input value");
        assert_eq!(new_event.message, message, "message should match the input value");
    }

    #[test]
    /// Tests that NewAgentEvent creation fails when the event_type exceeds 50 characters.
    fn test_new_agent_event_event_type_too_long() {
        let long_event_type = "a".repeat(51);
        let result = NewAgentEvent::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            long_event_type,
            "success".to_string(),
            None,
        );
        assert!(result.is_err(), "NewAgentEvent creation should fail with an event_type longer than 50 characters");
        assert_eq!(result.unwrap_err(), "Event type cannot exceed 50 characters", "Error message should indicate the event_type is too long");
    }

    #[test]
    /// Tests that NewAgentEvent creation fails when the status exceeds 10 characters.
    fn test_new_agent_event_status_too_long() {
        let long_status = "a".repeat(11);
        let result = NewAgentEvent::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "test_event".to_string(),
            long_status,
            None,
        );
        assert!(result.is_err(), "NewAgentEvent creation should fail with a status longer than 10 characters");
        assert_eq!(result.unwrap_err(), "Status cannot exceed 10 characters", "Error message should indicate the status is too long");
    }

    #[test]
    /// Tests the creation of a NewAgentEvent with a "fail" status.
    fn test_new_agent_event_fail_status() {
        let result = NewAgentEvent::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "test_event".to_string(),
            "fail".to_string(),
            Some("Test failure message".to_string()),
        );
        assert!(result.is_ok(), "NewAgentEvent creation should succeed with a 'fail' status");
        let new_event = result.unwrap();
        assert_eq!(new_event.status, "fail", "status should be set to 'fail'");
    }
}