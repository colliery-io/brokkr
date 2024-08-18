// src/models/agent_events.rs

use chrono::{DateTime, Utc};
use diesel::prelude::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use crate::schema::agent_events;

#[derive(Queryable, Selectable, Identifiable, Debug, Clone, Serialize, Deserialize, AsChangeset)]
#[diesel(table_name = agent_events)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub struct AgentEvent {
    pub id: Uuid,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub uuid: Uuid,
    pub agent_id: Uuid,
    pub deployment_object_id: Uuid,
    pub event_type: String,
    pub status: String,
    pub message: Option<String>,
}

#[derive(Insertable, Debug, Clone, Serialize, Deserialize)]
#[diesel(table_name = agent_events)]
pub struct NewAgentEvent {
    pub uuid: Uuid,
    pub agent_id: Uuid,
    pub deployment_object_id: Uuid,
    pub event_type: String,
    pub status: String,
    pub message: Option<String>,
}

impl NewAgentEvent {
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

        assert_eq!(new_event.agent_id, agent_id);
        assert_eq!(new_event.deployment_object_id, deployment_object_id);
        assert_eq!(new_event.event_type, event_type);
        assert_eq!(new_event.status, status);
        assert_eq!(new_event.message, message);
        assert!(!new_event.uuid.is_nil());
    }

    #[test]
    fn test_new_agent_event_fail_status() {
        let new_event = NewAgentEvent::new(
            Uuid::new_v4(),
            Uuid::new_v4(),
            "test_event".to_string(),
            "fail".to_string(),
            Some("Test failure message".to_string()),
        );

        assert_eq!(new_event.status, "fail");
    }
}