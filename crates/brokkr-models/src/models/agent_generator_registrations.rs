/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! # Agent Generator Registration Module
//!
//! Subscription join table between agents and generators.
//!
//! An agent must be registered with a generator before any of that generator's
//! stacks can be targeted at it. Agents opt in; the broker enforces the check
//! in `authorize_target_mutation`.

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

/// A registration linking an agent to a generator scope.
#[derive(
    Queryable,
    Selectable,
    Identifiable,
    Debug,
    Clone,
    Serialize,
    Deserialize,
    Eq,
    PartialEq,
    Hash,
    ToSchema,
)]
#[diesel(table_name = crate::schema::agent_generator_registrations)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AgentGeneratorRegistration {
    pub id: Uuid,
    pub agent_id: Uuid,
    pub generator_id: Uuid,
    pub registered_at: DateTime<Utc>,
}

/// Data required to insert a new registration.
#[derive(Insertable, Debug, Clone, Serialize, Deserialize, ToSchema)]
#[diesel(table_name = crate::schema::agent_generator_registrations)]
pub struct NewAgentGeneratorRegistration {
    pub agent_id: Uuid,
    pub generator_id: Uuid,
}

impl NewAgentGeneratorRegistration {
    pub fn new(agent_id: Uuid, generator_id: Uuid) -> Result<Self, String> {
        if agent_id.is_nil() {
            return Err("Invalid agent ID".to_string());
        }
        if generator_id.is_nil() {
            return Err("Invalid generator ID".to_string());
        }
        Ok(NewAgentGeneratorRegistration {
            agent_id,
            generator_id,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_registration_success() {
        let r = NewAgentGeneratorRegistration::new(Uuid::new_v4(), Uuid::new_v4());
        assert!(r.is_ok());
    }

    #[test]
    fn test_new_registration_nil_agent() {
        let r = NewAgentGeneratorRegistration::new(Uuid::nil(), Uuid::new_v4());
        assert!(r.is_err());
        assert_eq!(r.unwrap_err(), "Invalid agent ID");
    }

    #[test]
    fn test_new_registration_nil_generator() {
        let r = NewAgentGeneratorRegistration::new(Uuid::new_v4(), Uuid::nil());
        assert!(r.is_err());
        assert_eq!(r.unwrap_err(), "Invalid generator ID");
    }
}
