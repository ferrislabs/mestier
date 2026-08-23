use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{AssignmentReportResolution, ResolveAssignmentReportCommand};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::assignment_report::{
    AssignmentReportResolutionPath, AssignmentReportResponse, require_assignment_report,
    resolve_caller_member,
};

/// `resolution` must be `APPLIED` or `DISMISSED` — `PENDING` is refused by
/// `AssignmentReportService::resolve_report`, since it is not a decision a
/// manager makes *into*. `resolution_note` is independently optional on
/// either outcome.
#[derive(Debug, Deserialize, ToSchema)]
pub struct ResolveAssignmentReportRequest {
    pub resolution: AssignmentReportResolution,
    pub resolution_note: Option<String>,
}

/// The manager's arbitration: applies or dismisses a pending report.
///
/// Never touches the task — applying a report only records the decision.
/// Moving the plan is the existing `PATCH` on the task, which the webapp
/// chains after a successful apply (see the webapp issue).
#[utoipa::path(
    patch,
    path = "/api/v1/assignment-reports/{assignment_report_id}/resolution",
    operation_id = "resolveAssignmentReport",
    tag = crate::TAG,
    params(("assignment_report_id" = mestier_core::AssignmentReportId, Path, description = "Assignment report identifier")),
    request_body = ResolveAssignmentReportRequest,
    responses(
        (status = 200, description = "Report resolved", body = inline(DataEnvelope<AssignmentReportResponse>)),
        (status = 400, description = "`resolution` was `PENDING`, which is not a valid target"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "No such report"),
        (status = 409, description = "The report has already been resolved"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    AssignmentReportResolutionPath {
        assignment_report_id,
    }: AssignmentReportResolutionPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<ResolveAssignmentReportRequest>,
) -> Result<Response<AssignmentReportResponse>, ApiError> {
    let report = require_assignment_report(&state, &identity, assignment_report_id).await?;
    let resolved_by = resolve_caller_member(&state, &identity, report.organization_id).await?;

    let resolved = state
        .usecase
        .resolve_assignment_report(ResolveAssignmentReportCommand {
            id: assignment_report_id,
            resolved_by,
            resolution: payload.resolution,
            resolution_note: payload.resolution_note,
        })
        .await?;

    Ok(Response::OK(resolved.into()))
}
