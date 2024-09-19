use brokkr_models::models::generator::{Generator, NewGenerator};
use diesel::prelude::*;
use uuid::Uuid;
use crate::dal::DAL;
pub struct GeneratorsDal<'a> {
   pub dal: &'a DAL,
}

impl<'a> GeneratorsDal<'a> {
    pub fn new(dal: &'a DAL) -> Self {
        Self { dal }
    }

    pub fn create(&self, new_generator: &NewGenerator) -> Result<Generator, diesel::result::Error> {
        use brokkr_models::schema::generators::dsl::*;

        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        
        diesel::insert_into(generators)
            .values(new_generator)
            .get_result(conn)

        
    }

    pub fn get(&self, generator_id: Uuid) -> Result<Option<Generator>, diesel::result::Error> {
        use brokkr_models::schema::generators::dsl::*;

        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        generators
            .filter(id.eq(generator_id))
            .first::<Generator>(conn)
            .optional()

    }

    pub fn update(&self, generator_id: Uuid, updated_generator: &Generator) -> Result<Generator, diesel::result::Error> {
        use brokkr_models::schema::generators::dsl::*;

        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::update(generators.filter(id.eq(generator_id)))
            .set(updated_generator)
            .get_result(conn)
    }

    pub fn delete(&self, generator_id: Uuid) -> Result<usize, diesel::result::Error> {
        use brokkr_models::schema::generators::dsl::*;

        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::delete(generators.filter(id.eq(generator_id)))
            .execute(conn)

    }

    pub fn list(&self) -> Result<Vec<Generator>, diesel::result::Error> {
        use brokkr_models::schema::generators::dsl::*;

        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        generators
            .load::<Generator>(conn)

    }


    pub fn update_last_active(&self, generator_id: Uuid) -> Result<Generator, diesel::result::Error> {
        use brokkr_models::schema::generators::dsl::*;
        use diesel::dsl::now;

        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::update(generators.filter(id.eq(generator_id)))
            .set(last_active_at.eq(now))
            .get_result(conn)

    }

    pub fn soft_delete(&self, generator_id: Uuid) -> Result<usize, diesel::result::Error> {
        use brokkr_models::schema::generators::dsl::*;
        use diesel::dsl::now;

        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::update(generators.filter(id.eq(generator_id)))
            .set(deleted_at.eq(Some(now)))
            .execute(conn)

    }

    pub fn hard_delete(&self, generator_id: Uuid) -> Result<usize, diesel::result::Error> {
        use brokkr_models::schema::generators::dsl::*;

        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::delete(generators.filter(id.eq(generator_id)))
            .execute(conn)

    }

    pub fn list_all(&self) -> Result<Vec<Generator>, diesel::result::Error> {
        use brokkr_models::schema::generators::dsl::*;

        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        generators
            .load::<Generator>(conn)

    }

    pub fn get_including_deleted(&self, generator_id: Uuid) -> Result<Option<Generator>, diesel::result::Error> {
        use brokkr_models::schema::generators::dsl::*;

        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        generators
            .filter(id.eq(generator_id))
            .first::<Generator>(conn)
            .optional()

    }
}
