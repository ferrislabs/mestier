use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::{MaintenanceLineId, PlanMaintenanceLinesCommand};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    maintenance_overview::{MaintenancePlanPath, parse_iso_week},
    require_org_membership,
    response::TaskResponse,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct PlanMaintenanceLinesRequest {
    pub line_ids: Vec<MaintenanceLineId>,
    /// The recap's ISO week, `2026-W43`.
    pub week: String,
}

/// Turns recap lines into backlog tasks: one undated task per line, in the
/// contract's project, with the site and the line. All or nothing.
#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/maintenance/recap/plan",
    operation_id = "planMaintenanceLines",
    tag = crate::TAG,
    params(("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier")),
    request_body = PlanMaintenanceLinesRequest,
    responses(
        (status = 201, description = "The tasks created, in the order of `line_ids`", body = inline(DataEnvelope<Vec<TaskResponse>>)),
        (status = 400, description = "`week` is not an ISO week like 2026-W43"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "A line does not exist in this organization"),
        (status = 409, description = "A line is already planned, or belongs to an archived or draft contract"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    MaintenancePlanPath { organization_id }: MaintenancePlanPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<PlanMaintenanceLinesRequest>,
) -> Result<Response<Vec<TaskResponse>>, ApiError> {
    let (year, week) = parse_iso_week(&payload.week)?;
    require_org_membership(&state, &identity, organization_id).await?;
    let (_, actor) = resolve_actor(&state, &identity).await?;

    let tasks = state
        .usecase
        .plan_maintenance_lines(PlanMaintenanceLinesCommand {
            actor,
            organization_id,
            line_ids: payload.line_ids,
            year,
            week,
        })
        .await?;

    Ok(Response::Created(
        tasks.into_iter().map(Into::into).collect(),
    ))
}
