use diesel::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};

mod agents;
mod agent_events;
mod stacks;
mod deployment_objects;


pub use stacks::StacksDAL;
pub use deployment_objects::DeploymentObjectsDAL;
pub use agents::AgentsDAL;
pub use agent_events::AgentEventsDAL;
pub struct DAL {
    pub pool: Pool<ConnectionManager<PgConnection>>,
}

impl DAL {
    pub fn new(pool: Pool<ConnectionManager<PgConnection>>) -> Self {
        DAL { pool }
    }

    pub fn agents(&self) -> AgentsDAL {
        AgentsDAL{dal: self}
    }

    pub fn agent_events(&self) -> AgentEventsDAL {
        AgentEventsDAL{dal: self}
    }

    pub fn stacks(&self) -> StacksDAL {
        StacksDAL { dal: self }
    }

    pub fn deployment_objects(&self) -> DeploymentObjectsDAL {
        DeploymentObjectsDAL { dal: self }
    }
}