use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, Response};
use mestier_core::WithdrawAssignmentReportCommand;

use crate::{paths::FieldAssignmentReportPath, resolve_field_actor};

/// Withdraws a still-pending report. Physical delete — see the migration
/// comment on `assignment_reports`.
///
/// Same "load first, derive the organization, 404 on a mismatched owner"
/// rule as `amend_report`.
#[utoipa::path(
    delete,
    path = "/api/v1/field/assignment-reports/{assignment_report_id}",
    operation_id = "withdrawAssignmentReport",
    tag = crate::TAG,
    params(("assignment_report_id" = mestier_core::AssignmentReportId, Path, description = "Assignment report identifier")),
    responses(
        (status = 204, description = "Report withdrawn"),
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
) -> Result<Response<()>, ApiError> {
    let report = state
        .usecase
        .get_assignment_report(assignment_report_id)
        .await?;
    let actor = resolve_field_actor(&state, &identity, report.organization_id).await?;
    if report.reported_by != actor.member_id {
        return Err(ApiError::NotFound);
    }

    state
        .usecase
        .withdraw_assignment_report(WithdrawAssignmentReportCommand {
            id: assignment_report_id,
            acting_member_id: actor.member_id,
        })
        .await?;

    Ok(Response::NoContent)
}
