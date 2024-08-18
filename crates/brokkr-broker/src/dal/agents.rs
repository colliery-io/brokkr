use diesel::prelude::*;
use uuid::Uuid;
use brokkr_models::models::agents::{Agent, NewAgent};
use brokkr_models::schema::agents;
use crate::dal::DAL;

pub struct AgentsDAL<'a> {
    pub dal: &'a DAL,
}

impl<'a> AgentsDAL<'a> {
    /// Create a new agent in the database
    pub fn create(&self, new_agent: &NewAgent) -> Result<Agent, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().unwrap();
        diesel::insert_into(agents::table)
            .values(new_agent)
            .get_result(conn)
    }

    /// Retrieve an agent by its UUID
    pub fn get(&self, uuid: Uuid) -> Result<Option<Agent>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().unwrap();
        agents::table
            .filter(agents::uuid.eq(uuid))
            .first(conn)
            .optional()
    }

    /// List all agents
    pub fn list(&self) -> Result<Vec<Agent>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().unwrap();
        agents::table.load::<Agent>(conn)
    }

    /// Update an existing agent
    pub fn update(&self, uuid: Uuid, agent: &Agent) -> Result<Agent, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().unwrap();
        diesel::update(agents::table.filter(agents::uuid.eq(uuid)))
            .set(agent)
            .get_result(conn)
    }

    /// Soft delete an agent
    pub fn soft_delete(&self, uuid: Uuid) -> Result<(), diesel::result::Error> {
        let conn = &mut self.dal.pool.get().unwrap();
        diesel::update(agents::table.filter(agents::uuid.eq(uuid)))
            .set(agents::deleted_at.eq(diesel::dsl::now))
            .execute(conn)
            .map(|_| ())
    }

    /// Update agent's last heartbeat
    pub fn update_heartbeat(&self, uuid: Uuid) -> Result<(), diesel::result::Error> {
        let conn = &mut self.dal.pool.get().unwrap();
        diesel::update(agents::table.filter(agents::uuid.eq(uuid)))
            .set(agents::last_heartbeat.eq(diesel::dsl::now))
            .execute(conn)
            .map(|_| ())
    }
}