use auth::Identity;
use axum::{Extension, extract::Query, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{OrganizationId, Permissions};

use crate::{
    paths::WorkedHoursPath, reporting::PeriodQuery, require_view_reports,
    response::WorkedHoursResponse,
};

/// Hours worked per employee over the period, for payroll.
///
/// Derived from the same read as the profitability report rather than a second
/// query: paying on one number while ranking on another is how two screens end
/// up disagreeing about the same week.
#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/reporting/worked-hours",
    operation_id = "getWorkedHours",
    tag = crate::TAG,
    params(
        ("organization_id" = OrganizationId, Path, description = "Organization identifier"),
        PeriodQuery,
    ),
    responses(
        (status = 200, description = "Hours worked over the period", body = inline(DataEnvelope<WorkedHoursResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Not a member of this organization, or missing VIEW_REPORTS"),
        (status = 409, description = "The period ends before it starts"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    WorkedHoursPath { organization_id }: WorkedHoursPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(period): Query<PeriodQuery>,
) -> Result<Response<WorkedHoursResponse>, ApiError> {
    let permissions = require_view_reports(&state, &identity, organization_id).await?;

    let report = state
        .usecase
        .profitability_report(organization_id, period.from, period.to)
        .await?;
    let costs_redacted = !permissions.contains(Permissions::VIEW_COST);

    Ok(Response::OK(WorkedHoursResponse::from_report(
        report,
        costs_redacted,
    )))
}
