/*
 * Copyright (c) 2025-2026 Dylan Storey
 * Licensed under the Elastic License 2.0.
 * See LICENSE file in the project root for full license text.
 */

//! Tenant listing for the operator console's scope selector (BROKKR-T-0269).
//!
//! Tenants ARE generators in Brokkr's tenancy model (ADR-0009): stacks carry a
//! `generator_id` and agents register under generators. This module exposes a
//! slim `[{id, name}]` view of the non-system generators at `GET /paks` so the
//! console can populate its scope selector without the full generator model
//! (and so the read-only UI PAK never sees `pak_hash` material).

use crate::api::v1::error::ApiError;
use crate::api::v1::middleware::AuthPayload;
use crate::dal::DAL;
use axum::{Json, Router, extract::Extension, extract::State, routing::get};
use serde::{Deserialize, Serialize};
use tracing::{error, info};
use utoipa::ToSchema;
use uuid::Uuid;

/// Creates and returns the tenant-listing routes.
pub fn routes() -> Router<DAL> {
    Router::new().route("/paks", get(list_paks))
}

/// Query parameters for tenant-scoped listings (BROKKR-T-0270): fleet,
/// stacks, and agent-events accept `?pak_id=<generator uuid>` and narrow the
/// result to that tenant's resources. A view filter, not an authorization
/// boundary (BROKKR-I-0032 non-goal). An unknown id yields an empty list.
#[derive(Debug, Clone, Copy, Deserialize, utoipa::IntoParams)]
pub struct PakScopeQuery {
    /// Tenant (generator) ID to scope the listing to.
    pub pak_id: Option<Uuid>,
}

/// One tenant entry for the console scope selector: a named PAK owner
/// (generator) reduced to its identity.
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct PakSummary {
    /// The tenant's generator ID (used as `?pak_id=` in scoped queries).
    pub id: Uuid,
    /// Human-readable tenant name (the generator name).
    pub name: String,
}

/// Lists named PAKs (tenants) for scope selection.
#[utoipa::path(
    get,
    path = "/paks",
    tag = "auth",
    responses(
        (status = 200, description = "List of named PAKs (tenants)", body = Vec<PakSummary>),
        (status = 403, description = "Forbidden - PAK does not have required rights", body = crate::api::v1::error::ErrorResponse),
        (status = 500, description = "Internal server error", body = crate::api::v1::error::ErrorResponse),
    ),
    security(("admin_pak" = []))
)]
async fn list_paks(
    State(dal): State<DAL>,
    Extension(auth): Extension<AuthPayload>,
) -> Result<Json<Vec<PakSummary>>, ApiError> {
    info!("Handling request to list named PAKs (tenants)");

    // Admin-gated; the read-only UI PAK is a readonly admin and qualifies.
    if !auth.admin {
        return Err(ApiError::forbidden(
            "admin_required",
            "admin access required",
        ));
    }

    let generators = dal.generators().list().map_err(|e| {
        error!("Failed to list generators for PAK listing: {:?}", e);
        ApiError::internal("failed to list PAKs")
    })?;

    Ok(Json(
        generators
            .into_iter()
            .map(|g| PakSummary {
                id: g.id,
                name: g.name,
            })
            .collect(),
    ))
}
