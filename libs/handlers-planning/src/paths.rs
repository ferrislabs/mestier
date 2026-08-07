//! Typed path extractors shared by the planning aggregates.
//!
//! Only the organization-scoped roots live here. Each aggregate declares its
//! own leaf paths in its own submodule, so workstreams do not collide on this
//! file.

use axum_extra::routing::TypedPath;
use mestier_core::OrganizationId;
use serde::Deserialize;

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/planning")]
pub struct PlanningPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/planning/availability")]
pub struct PlanningAvailabilityPath {
    pub organization_id: OrganizationId,
}
