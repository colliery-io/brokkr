//! # Agent Target Module
//!
//! This module defines structures and methods for managing agent targets in the system.
//!
//! ## Data Model
//!
//! Agent targets represent the association between agents and stacks. They are stored in the
//! `agent_targets` table with the following structure:
//!
//! - `id`: UUID, primary key
//! - `agent_id`: UUID, foreign key referencing the `agents` table
//! - `stack_id`: UUID, foreign key referencing the `stacks` table
//!
//! ## Usage
//!
//! Agent targets are used to define which stacks an agent is responsible for or associated with.
//! This relationship allows the system to determine which agents should interact with specific stacks,
//! enabling efficient distribution and management of workloads across agents.
//!
//! ## Constraints
//!
//! - Both `agent_id` and `stack_id` must be valid, non-nil UUIDs.
//! - There is a unique constraint on the combination of `agent_id` and `stack_id` to prevent
//!   duplicate associations.

use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Represents an agent target in the database.
#[derive(
    Queryable,
    Selectable,
    Identifiable,
    AsChangeset,
    Debug,
    Clone,
    Serialize,
    Deserialize,
    Eq,
    PartialEq,
    Hash,
)]
#[diesel(table_name = crate::schema::agent_targets)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AgentTarget {
    /// Unique identifier for the agent target.
    pub id: Uuid,
    /// ID of the agent associated with this target.
    pub agent_id: Uuid,
    /// ID of the stack associated with this target.
    pub stack_id: Uuid,
}

/// Represents a new agent target to be inserted into the database.
#[derive(Insertable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = crate::schema::agent_targets)]
pub struct NewAgentTarget {
    /// ID of the agent to associate with a stack.
    pub agent_id: Uuid,
    /// ID of the stack to associate with an agent.
    pub stack_id: Uuid,
}

impl NewAgentTarget {
    /// Creates a new `NewAgentTarget` instance.
    ///
    /// # Parameters
    ///
    /// * `agent_id`: UUID of the agent to associate with a stack.
    /// * `stack_id`: UUID of the stack to associate with an agent.
    ///
    /// # Returns
    ///
    /// Returns `Ok(NewAgentTarget)` if both UUIDs are valid and non-nil,
    /// otherwise returns an `Err` with a description of the validation failure.
    pub fn new(agent_id: Uuid, stack_id: Uuid) -> Result<Self, String> {
        // Validate agent_id
        if agent_id.is_nil() {
            return Err("Invalid agent ID".to_string());
        }

        // Validate stack_id
        if stack_id.is_nil() {
            return Err("Invalid stack ID".to_string());
        }

        Ok(NewAgentTarget { agent_id, stack_id })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_agent_target_success() {
        let agent_id = Uuid::new_v4();
        let stack_id = Uuid::new_v4();

        let result = NewAgentTarget::new(agent_id, stack_id);

        assert!(
            result.is_ok(),
            "NewAgentTarget creation should succeed with valid inputs"
        );
        let new_target = result.unwrap();
        assert_eq!(
            new_target.agent_id, agent_id,
            "agent_id should match the input value"
        );
        assert_eq!(
            new_target.stack_id, stack_id,
            "stack_id should match the input value"
        );
    }

    #[test]
    fn test_new_agent_target_invalid_agent_id() {
        let result = NewAgentTarget::new(Uuid::nil(), Uuid::new_v4());
        assert!(
            result.is_err(),
            "NewAgentTarget creation should fail with nil agent ID"
        );
        assert_eq!(
            result.unwrap_err(),
            "Invalid agent ID",
            "Error message should indicate invalid agent ID"
        );
    }

    #[test]
    fn test_new_agent_target_invalid_stack_id() {
        let result = NewAgentTarget::new(Uuid::new_v4(), Uuid::nil());
        assert!(
            result.is_err(),
            "NewAgentTarget creation should fail with nil stack ID"
        );
        assert_eq!(
            result.unwrap_err(),
            "Invalid stack ID",
            "Error message should indicate invalid stack ID"
        );
    }
}
