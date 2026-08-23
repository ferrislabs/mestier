use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::AmendAssignmentReportCommand;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    paths::FieldAssignmentReportPath, resolve_field_actor, response::AssignmentReportResponse,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct AmendAssignmentReportRequest {
    pub reported_minutes: u32,
    pub comment: Option<String>,
}

/// Amends a still-pending report — the worker changing their mind, not a
/// second opinion.
///
/// The id is bare, so unlike `report_assignment` this route has to load the
/// row first and derive the organization from it — same rule as
/// `FieldStopPath`'s own handler. Refuses an entry that belongs to somebody
/// else with a 404 rather than a 403: it keeps this from confirming a
/// colleague's report exists.
#[utoipa::path(
    patch,
    path = "/api/v1/field/assignment-reports/{assignment_report_id}",
    operation_id = "amendAssignmentReport",
    tag = crate::TAG,
    params(("assignment_report_id" = mestier_core::AssignmentReportId, Path, description = "Assignment report identifier")),
    request_body = AmendAssignmentReportRequest,
    responses(
        (status = 200, description = "Report amended", body = inline(DataEnvelope<AssignmentReportResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 404, description = "No such report for this caller"),
        (status = 409, description = "The report has already been resolved"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    FieldAssignmentReportPath {
        assignment_report_id,
    }: FieldAssignmentReportPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<AmendAssignmentReportRequest>,
) -> Result<Response<AssignmentReportResponse>, ApiError> {
    let report = state
        .usecase
        .get_assignment_report(assignment_report_id)
        .await?;
    let actor = resolve_field_actor(&state, &identity, report.organization_id).await?;
    if report.reported_by != actor.member_id {
        return Err(ApiError::NotFound);
    }

    let amended = state
        .usecase
        .amend_assignment_report(AmendAssignmentReportCommand {
            id: assignment_report_id,
            acting_member_id: actor.member_id,
            reported_minutes: payload.reported_minutes,
            comment: payload.comment,
        })
        .await?;

    Ok(Response::OK(amended.into()))
}
