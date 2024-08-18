use crate::models::{Stack, NewStack};
use crate::schema::stacks;
use crate::{Pool,PooledConnection};
use diesel::prelude::*;
use uuid::Uuid;

pub struct StackDAO {
    pool: Pool,
}

impl StackDAO {
    pub fn new(pool: Pool) -> Self {
        StackDAO { pool }
    }

    fn get_conn(&self) -> PooledConnection {
        self.pool.get().expect("Failed to get connection from pool")
    }

    pub fn create(&self, new_stack: &NewStack) -> QueryResult<Stack> {
        let mut conn = self.get_conn();
        diesel::insert_into(stacks::table)
            .values(new_stack)
            .get_result(&mut *conn)
    }

    pub fn get(&self, uuid: Uuid) -> QueryResult<Stack> {
        let mut conn = self.get_conn();
        stacks::table.find(uuid).first(&mut *conn)
    }

    pub fn update(&self, uuid: Uuid, new_stack: &NewStack) -> QueryResult<Stack> {
        let mut conn = self.get_conn();
        diesel::update(stacks::table.find(uuid))
            .set(new_stack)
            .get_result(&mut *conn)
    }

    pub fn delete(&self, uuid: Uuid) -> QueryResult<usize> {
        let mut conn = self.get_conn();
        diesel::delete(stacks::table.find(uuid)).execute(&mut *conn)
    }

    pub fn list(&self) -> QueryResult<Vec<Stack>> {
        let mut conn = self.get_conn();
        stacks::table.load::<Stack>(&mut *conn)
    }
}
