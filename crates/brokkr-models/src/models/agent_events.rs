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
//! - `uuid`: Uuid - Secondary UUID for the agent event (used for external references)
//! - `agent_id`: Uuid - UUID of the agent associated with this event
//! - `deployment_object_id`: Uuid - UUID of the deployment object associated with this event
//! - `event_type`: String - Type of the event (e.g., "deployment_started", "deployment_completed")
//! - `status`: String - Status of the event (e.g., "success", "fail")
//! - `message`: Option<String> - Optional message providing additional details about the event
//!
//! The `NewAgentEvent` struct is used for creating new agent events and contains the same fields
//! as `AgentEvent`, except for `id`, `created_at`, `updated_at`, and `deleted_at`, which are
//! managed by the database.

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
    /// Secondary UUID for the agent event (used for external references)
    pub uuid: Uuid,
    /// UUID of the agent associated with this event
    pub agent_id: Uuid,
    /// UUID of the deployment object associated with this event
    pub deployment_object_id: Uuid,
    /// Type of the event (e.g., "deployment_started", "deployment_completed")
    pub event_type: String,
    /// Status of the event (e.g., "success", "fail")
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
    /// UUID for the new agent event
    pub uuid: Uuid,
    /// UUID of the agent associated with this event
    pub agent_id: Uuid,
    /// UUID of the deployment object associated with this event
    pub deployment_object_id: Uuid,
    /// Type of the event
    pub event_type: String,
    /// Status of the event
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
    /// * `event_type` - Type of the event
    /// * `status` - Status of the event
    /// * `message` - Optional message providing additional details about the event
    ///
    /// # Returns
    ///
    /// A new `NewAgentEvent` instance with a generated UUID.
    pub fn new(
        agent_id: Uuid,
        deployment_object_id: Uuid,
        event_type: String,
        status: String,
        message: Option<String>,
    ) -> Self {
        NewAgentEvent {
            uuid: Uuid::new_v4(),
            agent_id,
            deployment_object_id,
            event_type,
            status,
            message,
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    /// Tests the creation and field values of a new NewAgentEvent with a "success" status.
    ///
    /// This test:
    /// 1. Creates a new NewAgentEvent with specific values for each field.
    /// 2. Verifies that each field of the created event matches the input values.
    /// 3. Checks that the automatically generated UUID is not nil.
    ///
    /// It ensures that the NewAgentEvent::new() method correctly sets all fields
    /// and generates a valid UUID for the event.
    fn test_new_agent_event() {
        let agent_id = Uuid::new_v4();
        let deployment_object_id = Uuid::new_v4();
        let event_type = "test_event".to_string();
        let status = "success".to_string();
        let message = Some("Test message".to_string());

        let new_event = NewAgentEvent::new(
            agent_id,
            deployment_object_id,
            event_type.clone(),
            status.clone(),
            message.clone(),
        );

        assert_eq!(new_event.agent_id, agent_id, "agent_id should match the input value");
        assert_eq!(new_event.deployment_object_id, deployment_object_id, "deployment_object_id should match the input value");
        assert_eq!(new_event.event_type, event_type, "event_type should match the input value");
        assert_eq!(new_event.status, status, "status should match the input value");
        assert_eq!(new_event.message, message, "message should match the input value");
        assert!(!new_event.uuid.is_nil(), "generated UUID should not be nil");
    }

    #[test]
    /// Tests the creation of a NewAgentEvent with a "fail" status.
    ///
    /// This test:
    /// 1. Creates a new NewAgentEvent with a "fail" status and a failure message.
    /// 2. Verifies that the status field is correctly set to "fail".
    ///
    /// It ensures that the NewAgentEvent::new() method correctly handles
    /// the creation of events with a failure status.
    fn test_new_agent_event_fail_status() {
        let new_event = NewAgentEvent::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "test_event".to_string(),
            "fail".to_string(),
            Some("Test failure message".to_string()),
        );

        assert_eq!(new_event.status, "fail", "status should be set to 'fail'");
    }
}