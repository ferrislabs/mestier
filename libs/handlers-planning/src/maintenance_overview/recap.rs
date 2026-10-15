use auth::Identity;
use axum::{
    Extension,
    extract::{Query, State},
};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::Recap;
use serde::Deserialize;
use utoipa::IntoParams;

use crate::{
    maintenance_overview::{MaintenanceRecapPath, parse_iso_week},
    require_org_membership,
};

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct RecapQuery {
    /// An ISO week, `2026-W43`.
    pub week: String,
}

/// What to schedule for a week: the lines due in it and the ones that were
/// due before it, grouped by town with the overdue ones first. Lines an open
/// task already carries, finished lines and on-demand lines are left out.
#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/maintenance/recap",
    operation_id = "getMaintenanceRecap",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        RecapQuery,
    ),
    responses(
        (status = 200, description = "The weekly recap", body = inline(DataEnvelope<Recap>)),
        (status = 400, description = "`week` is not an ISO week like 2026-W43"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 409, description = "The year has no such week"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    MaintenanceRecapPath { organization_id }: MaintenanceRecapPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(query): Query<RecapQuery>,
) -> Result<Response<Recap>, ApiError> {
    let (year, week) = parse_iso_week(&query.week)?;
    require_org_membership(&state, &identity, organization_id).await?;
    let (_, actor) = resolve_actor(&state, &identity).await?;

    let recap = state
        .usecase
        .maintenance_recap(actor, organization_id, year, week)
        .await?;

    Ok(Response::OK(recap))
}
