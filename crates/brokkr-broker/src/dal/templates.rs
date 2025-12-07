/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Data Access Layer for Stack Template operations.
//!
//! This module provides functionality to interact with the database for Stack Template-related
//! operations, including creating, retrieving, listing, and managing template versions.

use crate::dal::FilterType;
use crate::dal::DAL;
use brokkr_models::models::stack_templates::{NewStackTemplate, StackTemplate};
use brokkr_models::schema::{stack_templates, template_annotations, template_labels};
use chrono::Utc;
use diesel::prelude::*;
use std::collections::HashSet;
use uuid::Uuid;

/// Data Access Layer for Stack Template operations.
pub struct TemplatesDAL<'a> {
    /// Reference to the main DAL instance.
    pub dal: &'a DAL,
}

impl TemplatesDAL<'_> {
    /// Creates a new stack template in the database.
    ///
    /// # Arguments
    ///
    /// * `new_template` - A reference to the NewStackTemplate struct containing the template details.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the created StackTemplate on success, or a diesel::result::Error on failure.
    pub fn create(
        &self,
        new_template: &NewStackTemplate,
    ) -> Result<StackTemplate, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::insert_into(stack_templates::table)
            .values(new_template)
            .get_result(conn)
    }

    /// Creates a new version of an existing template.
    ///
    /// This method automatically determines the next version number based on
    /// the existing versions for the given generator_id and name combination.
    ///
    /// # Arguments
    ///
    /// * `generator_id` - Optional generator ID. None for system templates.
    /// * `name` - Name of the template.
    /// * `description` - Optional description.
    /// * `template_content` - The Tera template content.
    /// * `parameters_schema` - JSON Schema for parameter validation.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the created StackTemplate on success, or a diesel::result::Error on failure.
    pub fn create_new_version(
        &self,
        generator_id: Option<Uuid>,
        name: String,
        description: Option<String>,
        template_content: String,
        parameters_schema: String,
    ) -> Result<StackTemplate, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        // Get the current max version for this template name and generator
        let max_version: Option<i32> = match generator_id {
            Some(gid) => stack_templates::table
                .filter(stack_templates::deleted_at.is_null())
                .filter(stack_templates::name.eq(&name))
                .filter(stack_templates::generator_id.eq(Some(gid)))
                .select(diesel::dsl::max(stack_templates::version))
                .first(conn)?,
            None => stack_templates::table
                .filter(stack_templates::deleted_at.is_null())
                .filter(stack_templates::name.eq(&name))
                .filter(stack_templates::generator_id.is_null())
                .select(diesel::dsl::max(stack_templates::version))
                .first(conn)?,
        };

        let next_version = max_version.unwrap_or(0) + 1;

        let new_template = NewStackTemplate::new(
            generator_id,
            name,
            description,
            next_version,
            template_content,
            parameters_schema,
        )
        .map_err(|e| diesel::result::Error::QueryBuilderError(e.into()))?;

        diesel::insert_into(stack_templates::table)
            .values(&new_template)
            .get_result(conn)
    }

    /// Retrieves a non-deleted stack template by its UUID.
    ///
    /// # Arguments
    ///
    /// * `template_id` - The UUID of the template to retrieve.
    ///
    /// # Returns
    ///
    /// Returns a Result containing an Option<StackTemplate> if found, or a diesel::result::Error on failure.
    pub fn get(&self, template_id: Uuid) -> Result<Option<StackTemplate>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        stack_templates::table
            .filter(stack_templates::id.eq(template_id))
            .filter(stack_templates::deleted_at.is_null())
            .first(conn)
            .optional()
    }

    /// Retrieves a stack template by its UUID, including deleted templates.
    ///
    /// # Arguments
    ///
    /// * `template_id` - The UUID of the template to retrieve.
    ///
    /// # Returns
    ///
    /// Returns a Result containing an Option<StackTemplate> if found, or a diesel::result::Error on failure.
    pub fn get_including_deleted(
        &self,
        template_id: Uuid,
    ) -> Result<Option<StackTemplate>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        stack_templates::table
            .filter(stack_templates::id.eq(template_id))
            .first(conn)
            .optional()
    }

    /// Lists all non-deleted stack templates from the database.
    ///
    /// # Returns
    ///
    /// Returns a Result containing a Vec of all non-deleted StackTemplates on success,
    /// or a diesel::result::Error on failure.
    pub fn list(&self) -> Result<Vec<StackTemplate>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        stack_templates::table
            .filter(stack_templates::deleted_at.is_null())
            .load::<StackTemplate>(conn)
    }

    /// Lists all stack templates from the database, including deleted ones.
    ///
    /// # Returns
    ///
    /// Returns a Result containing a Vec of all StackTemplates on success,
    /// or a diesel::result::Error on failure.
    pub fn list_all(&self) -> Result<Vec<StackTemplate>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        stack_templates::table.load::<StackTemplate>(conn)
    }

    /// Gets the latest version of a template by name and generator_id.
    ///
    /// # Arguments
    ///
    /// * `generator_id` - Optional generator ID. None for system templates.
    /// * `name` - Name of the template.
    ///
    /// # Returns
    ///
    /// Returns a Result containing an Option<StackTemplate> with the highest version,
    /// or a diesel::result::Error on failure.
    pub fn get_latest_version(
        &self,
        generator_id: Option<Uuid>,
        name: &str,
    ) -> Result<Option<StackTemplate>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        match generator_id {
            Some(gid) => stack_templates::table
                .filter(stack_templates::deleted_at.is_null())
                .filter(stack_templates::name.eq(name))
                .filter(stack_templates::generator_id.eq(Some(gid)))
                .order(stack_templates::version.desc())
                .first(conn)
                .optional(),
            None => stack_templates::table
                .filter(stack_templates::deleted_at.is_null())
                .filter(stack_templates::name.eq(name))
                .filter(stack_templates::generator_id.is_null())
                .order(stack_templates::version.desc())
                .first(conn)
                .optional(),
        }
    }

    /// Lists all versions of a template by name and generator_id.
    ///
    /// # Arguments
    ///
    /// * `generator_id` - Optional generator ID. None for system templates.
    /// * `name` - Name of the template.
    ///
    /// # Returns
    ///
    /// Returns a Result containing a Vec of all versions of the template,
    /// ordered by version descending.
    pub fn list_versions(
        &self,
        generator_id: Option<Uuid>,
        name: &str,
    ) -> Result<Vec<StackTemplate>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        match generator_id {
            Some(gid) => stack_templates::table
                .filter(stack_templates::deleted_at.is_null())
                .filter(stack_templates::name.eq(name))
                .filter(stack_templates::generator_id.eq(Some(gid)))
                .order(stack_templates::version.desc())
                .load::<StackTemplate>(conn),
            None => stack_templates::table
                .filter(stack_templates::deleted_at.is_null())
                .filter(stack_templates::name.eq(name))
                .filter(stack_templates::generator_id.is_null())
                .order(stack_templates::version.desc())
                .load::<StackTemplate>(conn),
        }
    }

    /// Lists all non-deleted templates for a specific generator.
    ///
    /// # Arguments
    ///
    /// * `generator_id` - The UUID of the generator.
    ///
    /// # Returns
    ///
    /// Returns a Result containing a Vec of templates for the generator.
    pub fn list_for_generator(
        &self,
        generator_id: Uuid,
    ) -> Result<Vec<StackTemplate>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        stack_templates::table
            .filter(stack_templates::deleted_at.is_null())
            .filter(stack_templates::generator_id.eq(Some(generator_id)))
            .load::<StackTemplate>(conn)
    }

    /// Lists all non-deleted system templates (generator_id IS NULL).
    ///
    /// # Returns
    ///
    /// Returns a Result containing a Vec of system templates.
    pub fn list_system_templates(&self) -> Result<Vec<StackTemplate>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        stack_templates::table
            .filter(stack_templates::deleted_at.is_null())
            .filter(stack_templates::generator_id.is_null())
            .load::<StackTemplate>(conn)
    }

    /// Soft deletes a stack template by setting its deleted_at timestamp.
    ///
    /// # Arguments
    ///
    /// * `template_id` - The UUID of the template to soft delete.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the number of affected rows (0 or 1).
    pub fn soft_delete(&self, template_id: Uuid) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::update(stack_templates::table.filter(stack_templates::id.eq(template_id)))
            .set(stack_templates::deleted_at.eq(Utc::now()))
            .execute(conn)
    }

    /// Hard deletes a stack template from the database.
    ///
    /// # Arguments
    ///
    /// * `template_id` - The UUID of the template to hard delete.
    ///
    /// # Returns
    ///
    /// Returns a Result containing the number of affected rows (0 or 1).
    pub fn hard_delete(&self, template_id: Uuid) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");
        diesel::delete(stack_templates::table.filter(stack_templates::id.eq(template_id)))
            .execute(conn)
    }

    /// Filters templates by labels.
    ///
    /// # Arguments
    ///
    /// * `labels` - A vector of label strings to filter by.
    /// * `filter_type` - Specifies whether to use AND or OR logic for multiple labels.
    ///
    /// # Returns
    ///
    /// Returns a Result containing a Vec of matching StackTemplates.
    pub fn filter_by_labels(
        &self,
        labels: Vec<String>,
        filter_type: FilterType,
    ) -> Result<Vec<StackTemplate>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        match filter_type {
            FilterType::And => {
                let mut query = stack_templates::table
                    .filter(stack_templates::deleted_at.is_null())
                    .into_boxed();

                for label in &labels {
                    let subquery = template_labels::table
                        .filter(template_labels::template_id.eq(stack_templates::id))
                        .filter(template_labels::label.eq(label));
                    query = query.filter(diesel::dsl::exists(subquery));
                }

                query
                    .select(stack_templates::all_columns)
                    .distinct()
                    .load::<StackTemplate>(conn)
            }
            FilterType::Or => stack_templates::table
                .inner_join(template_labels::table)
                .filter(stack_templates::deleted_at.is_null())
                .filter(template_labels::label.eq_any(labels))
                .select(stack_templates::all_columns)
                .distinct()
                .load::<StackTemplate>(conn),
        }
    }

    /// Filters templates by annotations.
    ///
    /// # Arguments
    ///
    /// * `annotations` - A vector of (key, value) pairs to filter by.
    /// * `filter_type` - Specifies whether to use AND or OR logic for multiple annotations.
    ///
    /// # Returns
    ///
    /// Returns a Result containing a Vec of matching StackTemplates.
    pub fn filter_by_annotations(
        &self,
        annotations: Vec<(String, String)>,
        filter_type: FilterType,
    ) -> Result<Vec<StackTemplate>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        match filter_type {
            FilterType::Or => {
                let mut all_matching_templates = HashSet::new();

                for (key, value) in annotations {
                    let matching_templates: Vec<StackTemplate> = stack_templates::table
                        .inner_join(template_annotations::table)
                        .filter(stack_templates::deleted_at.is_null())
                        .filter(template_annotations::key.eq(key))
                        .filter(template_annotations::value.eq(value))
                        .select(stack_templates::all_columns)
                        .load(conn)?;

                    all_matching_templates.extend(matching_templates);
                }

                Ok(all_matching_templates.into_iter().collect())
            }
            FilterType::And => {
                if annotations.is_empty() {
                    return Ok(Vec::new());
                }

                let mut all_matching_templates: Option<HashSet<StackTemplate>> = None;

                for (key, value) in annotations {
                    let matching_templates: HashSet<StackTemplate> = stack_templates::table
                        .inner_join(template_annotations::table)
                        .filter(stack_templates::deleted_at.is_null())
                        .filter(template_annotations::key.eq(key))
                        .filter(template_annotations::value.eq(value))
                        .select(stack_templates::all_columns)
                        .load(conn)?
                        .into_iter()
                        .collect();

                    all_matching_templates = match all_matching_templates {
                        Some(templates) => {
                            Some(templates.intersection(&matching_templates).cloned().collect())
                        }
                        None => Some(matching_templates),
                    };

                    if let Some(ref templates) = all_matching_templates {
                        if templates.is_empty() {
                            break;
                        }
                    }
                }

                Ok(all_matching_templates
                    .map_or_else(Vec::new, |templates| templates.into_iter().collect()))
            }
        }
    }
}
