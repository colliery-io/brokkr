use diesel::prelude::*;
use uuid::Uuid;
use brokkr_models::models::agent_events::{AgentEvent, NewAgentEvent};
use brokkr_models::schema::agent_events;
use chrono::Utc;
use crate::dal::DAL;

pub struct AgentEventsDAL<'a> {
    pub dal: &'a DAL,
}

impl<'a> AgentEventsDAL<'a> {
    /// Create a new agent event in the database
    pub fn create(&self, new_event: &NewAgentEvent) -> Result<AgentEvent, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().unwrap();
        diesel::insert_into(agent_events::table)
            .values(new_event)
            .get_result(conn)
    }

    

    /// List all agent events
    pub fn list(&self) -> Result<Vec<AgentEvent>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().unwrap();
        agent_events::table.load::<AgentEvent>(conn)
    }

    pub fn get(&self, event_uuid: Uuid) -> Result<Option<AgentEvent>, diesel::result::Error> {
        use brokkr_models::schema::agent_events::dsl::*;
        let conn = &mut self.dal.pool.get().unwrap();
        agent_events
            .filter(uuid.eq(event_uuid))
            .filter(deleted_at.is_null())
            .first(conn)
            .optional()
    }

    pub fn soft_delete(&self, event_uuid: Uuid) -> Result<(), diesel::result::Error> {
        use brokkr_models::schema::agent_events::dsl::*;
        let conn = &mut self.dal.pool.get().unwrap();
        diesel::update(agent_events.filter(uuid.eq(event_uuid)))
            .set(deleted_at.eq(Utc::now().naive_utc()))
            .execute(conn)
            .map(|_| ())
    }

    pub fn get_including_deleted(&self, event_uuid: Uuid) -> Result<Option<AgentEvent>, diesel::result::Error> {
        use brokkr_models::schema::agent_events::dsl::*;
        let conn = &mut self.dal.pool.get().unwrap();
        agent_events
            .filter(uuid.eq(event_uuid))
            .first(conn)
            .optional()
    }

}