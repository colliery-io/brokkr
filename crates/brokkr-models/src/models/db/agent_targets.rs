//! # Agent Targets Module
//!
//! This module defines the data structures and operations for agent targets in the Brokkr system.
//!
//! ## Core Data Model
//!
//! The core data model for agent targets is represented by the `AgentTarget` struct:
//!
//! - `id`: Uuid - Unique identifier for the agent target
//! - `stack_id`: Uuid - Identifier of the stack this agent target is associated with
//! - `agent_id`: Uuid - Identifier of the agent this target is associated with
//! - `created_at`: DateTime<Utc> - Timestamp when the agent target was created

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents an agent target in the system.
///
/// This struct is used for querying existing agent targets from the database.
#[derive(Queryable, Selectable, Identifiable, AsChangeset, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::agent_targets)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AgentTargetDB {
    /// Unique identifier for the agent target
    pub id: Uuid,
    /// Identifier of the stack this agent target is associated with
    pub stack_id: Uuid,
    /// Identifier of the agent this target is associated with
    pub agent_id: Uuid,
    /// Timestamp when the agent target was created
    pub created_at: DateTime<Utc>,
}

/// Represents a new agent target to be inserted into the database.
///
/// This struct is used when creating new agent targets.
#[derive(Insertable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::agent_targets)]
pub struct NewAgentTargetDB {
    /// Identifier of the stack this agent target is associated with
    pub stack_id: Uuid,
    /// Identifier of the agent this target is associated with
    pub agent_id: Uuid,
}

impl NewAgentTargetDB {
    /// Creates a new `NewAgentTarget` instance.
    ///
    /// # Arguments
    ///
    /// * `stack_id` - UUID of the stack this agent target is associated with
    /// * `agent_id` - UUID of the agent this target is associated with
    ///
    /// # Returns
    ///
    /// A `Result` containing a new `NewAgentTarget` instance if successful, or an error message if validation fails.
    pub fn new(stack_id: Uuid, agent_id: Uuid) -> Result<Self, String> {
        // Validate stack_id
        if stack_id.is_nil() {
            return Err("Invalid stack ID".to_string());
        }

        // Validate agent_id
        if agent_id.is_nil() {
            return Err("Invalid agent ID".to_string());
        }

        Ok(NewAgentTargetDB {
            stack_id,
            agent_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_agent_target_success() {
        let stack_id = Uuid::new_v4();
        let agent_id = Uuid::new_v4();

        let result = NewAgentTargetDB::new(stack_id, agent_id);

        assert!(result.is_ok(), "NewAgentTarget creation should succeed with valid inputs");
        let new_agent_target = result.unwrap();
        assert_eq!(new_agent_target.stack_id, stack_id, "stack_id should match the input value");
        assert_eq!(new_agent_target.agent_id, agent_id, "agent_id should match the input value");
    }

    #[test]
    fn test_new_agent_target_invalid_stack_id() {
        let result = NewAgentTargetDB::new(Uuid::nil(), Uuid::new_v4());
        assert!(result.is_err(), "NewAgentTarget creation should fail with nil stack ID");
        assert_eq!(result.unwrap_err(), "Invalid stack ID", "Error message should indicate invalid stack ID");
    }

    #[test]
    fn test_new_agent_target_invalid_agent_id() {
        let result = NewAgentTargetDB::new(Uuid::new_v4(), Uuid::nil());
        assert!(result.is_err(), "NewAgentTarget creation should fail with nil agent ID");
        assert_eq!(result.unwrap_err(), "Invalid agent ID", "Error message should indicate invalid agent ID");
    }
}