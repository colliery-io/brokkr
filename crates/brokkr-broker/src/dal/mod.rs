// src/dal/mod.rs

use diesel::PgConnection;
use diesel::r2d2::{ConnectionManager, Pool};

mod stacks;
mod deployment_objects;

pub use stacks::StacksDAL;
pub use deployment_objects::DeploymentObjectsDAL;

pub struct DAL {
    pub pool: Pool<ConnectionManager<PgConnection>>,
}

impl DAL {
    pub fn new(pool: Pool<ConnectionManager<PgConnection>>) -> Self {
        DAL { pool }
    }

    pub fn stacks(&self) -> StacksDAL {
        StacksDAL { dal: self }
    }

    pub fn deployment_objects(&self) -> DeploymentObjectsDAL {
        DeploymentObjectsDAL { dal: self }
    }
}