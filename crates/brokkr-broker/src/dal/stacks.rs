use brokkr_models::models::stacks::{Stack, NewStack};
use chrono::Utc;
use crate::dal::DAL;
use diesel::prelude::*;
use uuid::Uuid;

pub struct StacksDAL<'a> {
    pub(crate) dal: &'a DAL,
}

impl<'a> StacksDAL<'a> {
    /// Create a new stack in the database
    pub fn create(&self, new_stack: &NewStack) -> QueryResult<Stack> {
        use brokkr_models::schema::stacks::dsl::*;

        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        diesel::insert_into(stacks)
            .values(new_stack)
            .get_result(conn)
    }

    /// Retrieve a stack by its ID
    pub fn get_by_id(&self, stack_id: Uuid) -> QueryResult<Stack> {
        use brokkr_models::schema::stacks::dsl::*;

        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        stacks.filter(id.eq(stack_id)).first(conn)
    }

    /// Retrieve all stacks
    pub fn get_all(&self) -> QueryResult<Vec<Stack>> {
        use brokkr_models::schema::stacks::dsl::*;

        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        stacks.load(conn)
    }

    /// Update an existing stack
    pub fn update(&self, stack_id: Uuid, updated_stack: &Stack) -> QueryResult<Stack> {
        use brokkr_models::schema::stacks::dsl::*;

        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        diesel::update(stacks.filter(id.eq(stack_id)))
            .set(updated_stack)
            .get_result(conn)
    }

    /// Soft delete a stack by setting its deleted_at timestamp
    pub fn soft_delete(&self, stack_id: Uuid) -> QueryResult<Stack> {
        use brokkr_models::schema::stacks::dsl::*;

        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        diesel::update(stacks.filter(id.eq(stack_id)))
            .set(deleted_at.eq(Some(Utc::now())))
            .get_result(conn)
    }

    /// Retrieve all active (non-deleted) stacks
    pub fn get_active(&self) -> QueryResult<Vec<Stack>> {
        use brokkr_models::schema::stacks::dsl::*;

        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        stacks.filter(deleted_at.is_null()).load(conn)
    }
}

