use crate::models::{DeploymentObject, NewDeploymentObject};
use crate::schema::deployment_objects;
use diesel::pg::PgConnection;
use diesel::prelude::*;
use diesel::result::QueryResult;
use serde_json::json;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

use diesel::sql_query;
use diesel::sql_types::{Jsonb, Text};

/// Data Access Object for DeploymentObject entities
pub struct DeploymentObjectDAO {
    conn: Arc<Mutex<PgConnection>>,
}

impl DeploymentObjectDAO {
    /// Creates a new DeploymentObjectDAO instance
    pub fn new(conn: Arc<Mutex<PgConnection>>) -> Self {
        DeploymentObjectDAO { conn }
    }

    /// Creates a new DeploymentObject in the database
    pub fn create(&self, new_object: &NewDeploymentObject) -> QueryResult<DeploymentObject> {
        let mut conn = self.conn.lock().unwrap();
        diesel::insert_into(deployment_objects::table)
            .values(new_object)
            .get_result(&mut *conn)
    }

    /// Retrieves a DeploymentObject by its UUID
    pub fn get(&self, uuid: Uuid) -> QueryResult<DeploymentObject> {
        let mut conn = self.conn.lock().unwrap();
        deployment_objects::table
            .filter(deployment_objects::uuid.eq(uuid))
            .first(&mut *conn)
    }

    /// Updates an existing DeploymentObject
    pub fn update(
        &self,
        uuid: Uuid,
        changes: &NewDeploymentObject,
    ) -> QueryResult<DeploymentObject> {
        let mut conn = self.conn.lock().unwrap();
        diesel::update(deployment_objects::table.filter(deployment_objects::uuid.eq(uuid)))
            .set((
                deployment_objects::stack_uuid.eq(changes.stack_uuid),
                deployment_objects::yaml_content.eq(&changes.yaml_content),
                deployment_objects::object_type.eq(&changes.object_type),
                deployment_objects::object_name.eq(&changes.object_name),
                deployment_objects::object_namespace.eq(&changes.object_namespace),
                deployment_objects::target.eq(&changes.target),
                deployment_objects::is_deletion.eq(changes.is_deletion),
                deployment_objects::version.eq(changes.version),
                deployment_objects::labels.eq(&changes.labels),
                deployment_objects::annotations.eq(&changes.annotations),
                deployment_objects::modified_at.eq(diesel::dsl::now),
            ))
            .get_result(&mut *conn)
    }

    /// Deletes a DeploymentObject by its UUID
    pub fn delete(&self, uuid: Uuid) -> QueryResult<usize> {
        let mut conn = self.conn.lock().unwrap();
        diesel::delete(deployment_objects::table.filter(deployment_objects::uuid.eq(uuid)))
            .execute(&mut *conn)
    }

    pub fn find_by_label(&self, label: &str) -> QueryResult<Vec<DeploymentObject>> {
        let mut conn = self.conn.lock().unwrap();
        sql_query("SELECT * FROM deployment_objects WHERE labels IS NOT NULL AND labels @> $1")
            .bind::<Jsonb, _>(json!([label]))
            .load::<DeploymentObject>(&mut *conn)
    }

    pub fn find_by_any_label(&self, labels: &[&str]) -> QueryResult<Vec<DeploymentObject>> {
        let mut conn = self.conn.lock().unwrap();
        let condition = labels
            .iter()
            .map(|label| format!("labels @> '{}'", json!([label])))
            .collect::<Vec<_>>()
            .join(" OR ");
        let query = format!(
            "SELECT * FROM deployment_objects WHERE labels IS NOT NULL AND ({})",
            condition
        );
        sql_query(query).load::<DeploymentObject>(&mut *conn)
    }

    pub fn find_by_label_in(&self, labels: &[&str]) -> QueryResult<Vec<DeploymentObject>> {
        self.find_by_any_label(labels)
    }

    pub fn find_by_label_containing(&self, pattern: &str) -> QueryResult<Vec<DeploymentObject>> {
        let mut conn = self.conn.lock().unwrap();
        sql_query("SELECT * FROM deployment_objects WHERE labels IS NOT NULL AND EXISTS (SELECT 1 FROM jsonb_array_elements_text(labels) AS label WHERE label LIKE $1)")
            .bind::<Text, _>(format!("%{}%", pattern))
            .load::<DeploymentObject>(&mut *conn)
    }

    /// Finds DeploymentObjects by a specific annotation
    pub fn find_by_annotation(&self, key: &str, value: &str) -> QueryResult<Vec<DeploymentObject>> {
        let mut conn = self.conn.lock().unwrap();
        sql_query(
            "SELECT * FROM deployment_objects WHERE annotations IS NOT NULL AND annotations @> $1",
        )
        .bind::<Jsonb, _>(json!([[key, value]]))
        .load::<DeploymentObject>(&mut *conn)
    }

    /// Finds DeploymentObjects by stack UUID
    pub fn find_by_stack(&self, stack_uuid: Uuid) -> QueryResult<Vec<DeploymentObject>> {
        let mut conn = self.conn.lock().unwrap();
        deployment_objects::table
            .filter(deployment_objects::stack_uuid.eq(stack_uuid))
            .load::<DeploymentObject>(&mut *conn)
    }

    /// Finds DeploymentObjects by type and name
    pub fn find_by_type_and_name(
        &self,
        object_type: &str,
        object_name: &str,
    ) -> QueryResult<Vec<DeploymentObject>> {
        let mut conn = self.conn.lock().unwrap();
        deployment_objects::table
            .filter(deployment_objects::object_type.eq(object_type))
            .filter(deployment_objects::object_name.eq(object_name))
            .load::<DeploymentObject>(&mut *conn)
    }

    /// Lists all DeploymentObjects
    pub fn list(&self) -> QueryResult<Vec<DeploymentObject>> {
        let mut conn = self.conn.lock().unwrap();
        deployment_objects::table.load::<DeploymentObject>(&mut *conn)
    }
}
