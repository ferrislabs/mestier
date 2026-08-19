use axum_extra::routing::TypedPath;
use mestier_core::OrganizationId;
use serde::Deserialize;

/// Profitability per chantier, with the rankings derived from it.
#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/reporting/profitability")]
pub struct ProfitabilityPath {
    pub organization_id: OrganizationId,
}

/// Hours worked per employee over a period, which is what payroll needs.
#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/reporting/worked-hours")]
pub struct WorkedHoursPath {
    pub organization_id: OrganizationId,
}
