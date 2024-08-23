use diesel::prelude::*;
use uuid::Uuid;
use brokkr_models::models::agents::{Agent, NewAgent};
use brokkr_models::schema::agents;
use crate::dal::DAL;
use chrono::Utc;

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

   /// Retrieve an agent by its UUID, excluding soft-deleted agents
   #[allow(unused_variables)]
   pub fn get(&self, uuid: Uuid) -> Result<Option<Agent>, diesel::result::Error> {
    use brokkr_models::schema::agents::dsl::*;
    let conn = &mut self.dal.pool.get().unwrap();
    let result = agents
        .filter(uuid.eq(uuid))
        .filter(deleted_at.is_null())
        .first(conn)
        .optional();
    
    println!("Get agent result: {:?}", result);
    result
}

    /// Soft delete an agent
    #[allow(unused_variables)]
    pub fn soft_delete(&self, uuid: Uuid) -> Result<(), diesel::result::Error> {
        use brokkr_models::schema::agents::dsl::*;
        let conn = &mut self.dal.pool.get().unwrap();
        let now = Utc::now().naive_utc();
        let result = diesel::update(agents.filter(uuid.eq(uuid)))
            .set(deleted_at.eq(now))
            .execute(conn);
        
        println!("Soft delete result: {:?}", result);
        match result {
            Ok(num_affected) => {
                if num_affected == 0 {
                    println!("No rows were updated during soft delete");
                } else {
                    println!("{} row(s) were updated during soft delete", num_affected);
                }
                Ok(())
            },
            Err(e) => {
                println!("Error during soft delete: {:?}", e);
                Err(e)
            }
        }
    }
    /// List all agents
    pub fn list(&self) -> Result<Vec<Agent>, diesel::result::Error> {
        
        let conn = &mut self.dal.pool.get().unwrap();
        agents::table
        .filter(agents::deleted_at.is_null())
        .select(agents::all_columns)
        .load::<Agent>(conn)
    }

    /// Update an existing agent
    pub fn update(&self, uuid: Uuid, agent: &Agent) -> Result<Agent, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().unwrap();
        diesel::update(agents::table.filter(agents::uuid.eq(uuid)))
            .set(agent)
            .get_result(conn)
    }



    /// Update agent's last heartbeat
    pub fn update_heartbeat(&self, uuid: Uuid) -> Result<Agent, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().unwrap();
        diesel::update(agents::table.filter(agents::uuid.eq(uuid)))
            .set(agents::last_heartbeat.eq(diesel::dsl::now))
            .get_result(conn)
    }

    /// Update agent's status
    pub fn update_status(&self, uuid: Uuid, status: &str) -> Result<Agent, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().unwrap();
        diesel::update(agents::table.filter(agents::uuid.eq(uuid)))
            .set(agents::status.eq(status))
            .get_result(conn)
    }
}