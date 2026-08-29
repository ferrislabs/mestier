use auth::Identity;
use axum::{Extension, extract::Query, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{OrganizationId, Permissions};

use crate::{
    paths::ProfitabilityPath, reporting::PeriodQuery, require_view_reports,
    response::ProfitabilityResponse,
};

/// What every job in the period cost, against what it was quoted.
///
/// Carries the rankings and the list of jobs whose figures are incomplete, so a
/// dashboard never has to decide what "least profitable" means or which numbers
/// to trust.
#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/reporting/profitability",
    operation_id = "getProfitability",
    tag = crate::TAG,
    params(
        ("organization_id" = OrganizationId, Path, description = "Organization identifier"),
        PeriodQuery,
    ),
    responses(
        (status = 200, description = "Profitability over the period, money fields redacted without VIEW_COST", body = inline(DataEnvelope<ProfitabilityResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Not a member of this organization, or missing VIEW_REPORTS"),
        (status = 409, description = "The period ends before it starts"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    ProfitabilityPath { organization_id }: ProfitabilityPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(period): Query<PeriodQuery>,
) -> Result<Response<ProfitabilityResponse>, ApiError> {
    let permissions = require_view_reports(&state, &identity, organization_id).await?;

    let report = state
        .usecase
        .profitability_report(organization_id, period.from, period.to)
        .await?;
    let costs_redacted = !permissions.contains(Permissions::VIEW_COST);

    Ok(Response::OK(ProfitabilityResponse::from_report(
        report,
        costs_redacted,
    )))
}
