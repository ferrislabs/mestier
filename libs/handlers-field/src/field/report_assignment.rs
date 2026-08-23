use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::ReportAssignmentCommand;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    paths::FieldReportAssignmentPath, resolve_field_actor, response::AssignmentReportResponse,
};

/// Carries no `reported_by`: the caller always reports on their own
/// assignment, resolved from the authenticated identity — mirrors
/// `StartTimeEntryRequest`'s own "the employee is the caller, always" rule.
#[derive(Debug, Deserialize, ToSchema)]
pub struct ReportAssignmentRequest {
    /// Zero is a legitimate answer — "this did not happen".
    pub reported_minutes: u32,
    pub comment: Option<String>,
}

/// Files a report of an assignment's actual duration.
///
/// Refuses when the assignment does not exist, or belongs to somebody else —
/// `AssignmentReportService::report_assignment`'s own security rule.
/// Refuses a second pending report on the same assignment: amend the
/// existing one instead (`PATCH`).
#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/field/assignments/{task_assignment_id}/report",
    operation_id = "reportAssignment",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("task_assignment_id" = mestier_core::TaskAssignmentId, Path, description = "Assignment identifier"),
    ),
    request_body = ReportAssignmentRequest,
    responses(
        (status = 201, description = "Report filed", body = inline(DataEnvelope<AssignmentReportResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Caller is not this assignment's assignee"),
        (status = 404, description = "No such assignment"),
        (status = 409, description = "A pending report already exists for this assignment"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    FieldReportAssignmentPath {
        organization_id,
        task_assignment_id,
    }: FieldReportAssignmentPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<ReportAssignmentRequest>,
) -> Result<Response<AssignmentReportResponse>, ApiError> {
    let actor = resolve_field_actor(&state, &identity, organization_id).await?;

    let report = state
        .usecase
        .report_assignment(ReportAssignmentCommand {
            task_assignment_id,
            reported_by: actor.member_id,
            reported_minutes: payload.reported_minutes,
            comment: payload.comment,
        })
        .await?;

    Ok(Response::Created(report.into()))
}
