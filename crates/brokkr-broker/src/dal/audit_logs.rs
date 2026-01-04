/*
 * Copyright (c) 2025 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Data Access Layer for AuditLog operations.
//!
//! This module provides functionality to interact with the audit_logs table.
//! Audit logs are immutable - only create and query operations are supported.

use crate::dal::DAL;
use brokkr_models::models::audit_logs::{AuditLog, AuditLogFilter, NewAuditLog};
use brokkr_models::schema::audit_logs;
use chrono::{DateTime, Duration, Utc};
use diesel::prelude::*;
use uuid::Uuid;

/// Data Access Layer for AuditLog operations.
pub struct AuditLogsDAL<'a> {
    /// Reference to the main DAL instance.
    pub dal: &'a DAL,
}

impl AuditLogsDAL<'_> {
    /// Creates a new audit log entry.
    ///
    /// # Arguments
    ///
    /// * `new_log` - The audit log entry to create.
    ///
    /// # Returns
    ///
    /// Returns the created AuditLog record.
    pub fn create(&self, new_log: &NewAuditLog) -> Result<AuditLog, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        diesel::insert_into(audit_logs::table)
            .values(new_log)
            .get_result(conn)
    }

    /// Creates multiple audit log entries in a batch.
    ///
    /// # Arguments
    ///
    /// * `logs` - The audit log entries to create.
    ///
    /// # Returns
    ///
    /// Returns the number of inserted rows.
    pub fn create_batch(&self, logs: &[NewAuditLog]) -> Result<usize, diesel::result::Error> {
        if logs.is_empty() {
            return Ok(0);
        }

        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        diesel::insert_into(audit_logs::table)
            .values(logs)
            .execute(conn)
    }

    /// Gets an audit log entry by ID.
    ///
    /// # Arguments
    ///
    /// * `id` - The audit log UUID.
    ///
    /// # Returns
    ///
    /// Returns the audit log if found.
    pub fn get(&self, id: Uuid) -> Result<Option<AuditLog>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        audit_logs::table
            .filter(audit_logs::id.eq(id))
            .first(conn)
            .optional()
    }

    /// Lists audit logs with optional filtering and pagination.
    ///
    /// # Arguments
    ///
    /// * `filter` - Optional filter criteria.
    /// * `limit` - Maximum number of results (default 100, max 1000).
    /// * `offset` - Number of results to skip.
    ///
    /// # Returns
    ///
    /// Returns a list of audit logs ordered by timestamp descending.
    pub fn list(
        &self,
        filter: Option<&AuditLogFilter>,
        limit: Option<i64>,
        offset: Option<i64>,
    ) -> Result<Vec<AuditLog>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        let mut query = audit_logs::table.into_boxed();

        // Apply filters
        if let Some(f) = filter {
            if let Some(ref actor_type) = f.actor_type {
                query = query.filter(audit_logs::actor_type.eq(actor_type));
            }
            if let Some(actor_id) = f.actor_id {
                query = query.filter(audit_logs::actor_id.eq(actor_id));
            }
            if let Some(ref action) = f.action {
                // Support prefix matching with wildcard
                if action.ends_with('*') {
                    let prefix = &action[..action.len() - 1];
                    query = query.filter(audit_logs::action.like(format!("{}%", prefix)));
                } else {
                    query = query.filter(audit_logs::action.eq(action));
                }
            }
            if let Some(ref resource_type) = f.resource_type {
                query = query.filter(audit_logs::resource_type.eq(resource_type));
            }
            if let Some(resource_id) = f.resource_id {
                query = query.filter(audit_logs::resource_id.eq(resource_id));
            }
            if let Some(from) = f.from {
                query = query.filter(audit_logs::timestamp.ge(from));
            }
            if let Some(to) = f.to {
                query = query.filter(audit_logs::timestamp.lt(to));
            }
        }

        // Apply pagination
        let limit = limit.unwrap_or(100).min(1000);
        let offset = offset.unwrap_or(0);

        query
            .order(audit_logs::timestamp.desc())
            .limit(limit)
            .offset(offset)
            .load(conn)
    }

    /// Counts audit logs matching the filter.
    ///
    /// # Arguments
    ///
    /// * `filter` - Optional filter criteria.
    ///
    /// # Returns
    ///
    /// Returns the count of matching audit logs.
    pub fn count(&self, filter: Option<&AuditLogFilter>) -> Result<i64, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        let mut query = audit_logs::table.into_boxed();

        // Apply same filters as list
        if let Some(f) = filter {
            if let Some(ref actor_type) = f.actor_type {
                query = query.filter(audit_logs::actor_type.eq(actor_type));
            }
            if let Some(actor_id) = f.actor_id {
                query = query.filter(audit_logs::actor_id.eq(actor_id));
            }
            if let Some(ref action) = f.action {
                if action.ends_with('*') {
                    let prefix = &action[..action.len() - 1];
                    query = query.filter(audit_logs::action.like(format!("{}%", prefix)));
                } else {
                    query = query.filter(audit_logs::action.eq(action));
                }
            }
            if let Some(ref resource_type) = f.resource_type {
                query = query.filter(audit_logs::resource_type.eq(resource_type));
            }
            if let Some(resource_id) = f.resource_id {
                query = query.filter(audit_logs::resource_id.eq(resource_id));
            }
            if let Some(from) = f.from {
                query = query.filter(audit_logs::timestamp.ge(from));
            }
            if let Some(to) = f.to {
                query = query.filter(audit_logs::timestamp.lt(to));
            }
        }

        query.count().get_result(conn)
    }

    /// Deletes audit logs older than the specified retention period.
    ///
    /// # Arguments
    ///
    /// * `retention_days` - Number of days to retain logs.
    ///
    /// # Returns
    ///
    /// Returns the number of deleted rows.
    pub fn cleanup_old_logs(&self, retention_days: i64) -> Result<usize, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        let cutoff = Utc::now() - Duration::days(retention_days);

        diesel::delete(audit_logs::table.filter(audit_logs::created_at.lt(cutoff))).execute(conn)
    }

    /// Gets recent audit logs for a specific resource.
    ///
    /// # Arguments
    ///
    /// * `resource_type` - The type of resource.
    /// * `resource_id` - The ID of the resource.
    /// * `limit` - Maximum number of results.
    ///
    /// # Returns
    ///
    /// Returns recent audit logs for the resource.
    pub fn get_resource_history(
        &self,
        resource_type: &str,
        resource_id: Uuid,
        limit: i64,
    ) -> Result<Vec<AuditLog>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        audit_logs::table
            .filter(audit_logs::resource_type.eq(resource_type))
            .filter(audit_logs::resource_id.eq(resource_id))
            .order(audit_logs::timestamp.desc())
            .limit(limit)
            .load(conn)
    }

    /// Gets recent audit logs for a specific actor.
    ///
    /// # Arguments
    ///
    /// * `actor_type` - The type of actor.
    /// * `actor_id` - The ID of the actor.
    /// * `limit` - Maximum number of results.
    ///
    /// # Returns
    ///
    /// Returns recent audit logs for the actor.
    pub fn get_actor_history(
        &self,
        actor_type: &str,
        actor_id: Uuid,
        limit: i64,
    ) -> Result<Vec<AuditLog>, diesel::result::Error> {
        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        audit_logs::table
            .filter(audit_logs::actor_type.eq(actor_type))
            .filter(audit_logs::actor_id.eq(actor_id))
            .order(audit_logs::timestamp.desc())
            .limit(limit)
            .load(conn)
    }

    /// Gets failed authentication attempts within a time window.
    ///
    /// # Arguments
    ///
    /// * `since` - Start of the time window.
    /// * `ip_address` - Optional filter by IP address.
    ///
    /// # Returns
    ///
    /// Returns failed auth audit logs.
    pub fn get_failed_auth_attempts(
        &self,
        since: DateTime<Utc>,
        ip_address: Option<&str>,
    ) -> Result<Vec<AuditLog>, diesel::result::Error> {
        use brokkr_models::models::audit_logs::ACTION_AUTH_FAILED;

        let conn = &mut self.dal.pool.get().expect("Failed to get DB connection");

        let mut query = audit_logs::table
            .filter(audit_logs::action.eq(ACTION_AUTH_FAILED))
            .filter(audit_logs::timestamp.ge(since))
            .into_boxed();

        if let Some(ip) = ip_address {
            query = query.filter(audit_logs::ip_address.eq(ip));
        }

        query.order(audit_logs::timestamp.desc()).load(conn)
    }
}
