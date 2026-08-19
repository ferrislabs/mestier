use auth::Identity;
use axum::{Extension, extract::Query, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::OrganizationId;

use crate::{
    paths::ProfitabilityPath, reporting::PeriodQuery, require_org_membership,
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
        (status = 200, description = "Profitability over the period", body = inline(DataEnvelope<ProfitabilityResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Not a member of this organization"),
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
    require_org_membership(&state, &identity, organization_id).await?;

    let report = state
        .usecase
        .profitability_report(organization_id, period.from, period.to)
        .await?;

    Ok(Response::OK(report.into()))
}
