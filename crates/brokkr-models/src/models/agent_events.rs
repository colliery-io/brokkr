//! # Agent Event Module
//! 
//! This module defines structures and methods for managing agent events in the system.
//! 
//! ## Data Model
//! 
//! Agent events represent actions or occurrences related to agents and deployment objects. 
//! They are stored in the `agent_events` table with the following structure:
//! 
//! - `id`: UUID, primary key
//! - `created_at`: TIMESTAMP, when the event was created
//! - `updated_at`: TIMESTAMP, when the event was last updated
//! - `deleted_at`: TIMESTAMP, for soft deletion support
//! - `agent_id`: UUID, foreign key referencing the `agents` table
//! - `deployment_object_id`: UUID, foreign key referencing the `deployment_objects` table
//! - `event_type`: VARCHAR(50), type of the event
//! - `status`: VARCHAR(10), status of the event
//! - `message`: TEXT, optional message associated with the event
//! 
//! ## Usage
//! 
//! Agent events are used to track and record various actions and statuses related to agents
//! and their interactions with deployment objects. This can be useful for monitoring,
//! debugging, and auditing purposes.
//! 
//! ## Constraints
//! 
//! - Both `agent_id` and `deployment_object_id` must be valid, non-nil UUIDs.
//! - `event_type` must be a non-empty string.
//! - `status` must be one of: "SUCCESS", "FAILURE", "IN_PROGRESS", or "PENDING".

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents an agent event in the database.
#[derive(Queryable, Selectable, Identifiable, AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::agent_events)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AgentEvent {
    /// Unique identifier for the event.
    pub id: Uuid,
    /// Timestamp when the event was created.
    pub created_at: DateTime<Utc>,
    /// Timestamp when the event was last updated.
    pub updated_at: DateTime<Utc>,
    /// Timestamp for soft deletion, if applicable.
    pub deleted_at: Option<DateTime<Utc>>,
    /// ID of the agent associated with this event.
    pub agent_id: Uuid,
    /// ID of the deployment object associated with this event.
    pub deployment_object_id: Uuid,
    /// Type of the event.
    pub event_type: String,
    /// Status of the event (e.g., "SUCCESS", "FAILURE", "IN_PROGRESS", "PENDING").
    pub status: String,
    /// Optional message providing additional details about the event.
    pub message: Option<String>,
}

/// Represents a new agent event to be inserted into the database.
#[derive(Insertable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::agent_events)]
pub struct NewAgentEvent {
    /// ID of the agent associated with this event.
    pub agent_id: Uuid,
    /// ID of the deployment object associated with this event.
    pub deployment_object_id: Uuid,
    /// Type of the event.
    pub event_type: String,
    /// Status of the event (e.g., "SUCCESS", "FAILURE", "IN_PROGRESS", "PENDING").
    pub status: String,
    /// Optional message providing additional details about the event.
    pub message: Option<String>,
}

impl NewAgentEvent {
    /// Creates a new `NewAgentEvent` instance.
    ///
    /// # Parameters
    ///
    /// * `agent_id`: UUID of the agent associated with this event.
    /// * `deployment_object_id`: UUID of the deployment object associated with this event.
    /// * `event_type`: Type of the event. Must be a non-empty string.
    /// * `status`: Status of the event. Must be one of: "SUCCESS", "FAILURE", "IN_PROGRESS", or "PENDING".
    /// * `message`: Optional message providing additional details about the event.
    ///
    /// # Returns
    ///
    /// Returns `Ok(NewAgentEvent)` if all parameters are valid, otherwise returns an `Err` with a description of the validation failure.
    pub fn new(
        agent_id: Uuid,
        deployment_object_id: Uuid,
        event_type: String,
        status: String,
        message: Option<String>,
    ) -> Result<Self, String> {
        // Validate agent_id
        if agent_id.is_nil() {
            return Err("Invalid agent ID".to_string());
        }

        // Validate deployment_object_id
        if deployment_object_id.is_nil() {
            return Err("Invalid deployment object ID".to_string());
        }

        // Validate event_type
        if event_type.trim().is_empty() {
            return Err("Event type cannot be empty".to_string());
        }

        // Validate status
        let valid_statuses = ["SUCCESS", "FAILURE", "IN_PROGRESS", "PENDING"];
        if !valid_statuses.contains(&status.as_str()) {
            return Err(format!(
                "Invalid status. Must be one of: {}",
                valid_statuses.join(", ")
            ));
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
    fn test_new_agent_event_success() {
        let agent_id = Uuid::new_v4();
        let deployment_object_id = Uuid::new_v4();
        let event_type = "DEPLOYMENT".to_string();
        let status = "SUCCESS".to_string();
        let message = Some("Deployment completed successfully".to_string());

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
    fn test_new_agent_event_invalid_agent_id() {
        let result = NewAgentEvent::new(
            Uuid::nil(),
            Uuid::new_v4(),
            "DEPLOYMENT".to_string(),
            "SUCCESS".to_string(),
            None,
        );
        assert!(result.is_err(), "NewAgentEvent creation should fail with nil agent ID");
        assert_eq!(result.unwrap_err(), "Invalid agent ID", "Error message should indicate invalid agent ID");
    }

    #[test]
    fn test_new_agent_event_invalid_status() {
        let result = NewAgentEvent::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "DEPLOYMENT".to_string(),
            "INVALID_STATUS".to_string(),
            None,
        );
        assert!(result.is_err(), "NewAgentEvent creation should fail with invalid status");
        assert!(result.unwrap_err().contains("Invalid status"), "Error message should indicate invalid status");
    }

    #[test]
    fn test_new_agent_event_empty_event_type() {
        let result = NewAgentEvent::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "".to_string(),
            "SUCCESS".to_string(),
            None,
        );
        assert!(result.is_err(), "NewAgentEvent creation should fail with empty event type");
        assert_eq!(result.unwrap_err(), "Event type cannot be empty", "Error message should indicate empty event type");
    }
}