use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::{
    DEALS_TAG, paths::DealsSummaryPath, require_view_customers, response::DealSummaryResponse,
};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/deals/summary",
    operation_id = "getDealSummary",
    tag = DEALS_TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    responses(
        (status = 200, description = "Pipeline summary", body = inline(DataEnvelope<DealSummaryResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: DealsSummaryPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<DealSummaryResponse>, ApiError> {
    require_view_customers(&state, &identity, path.organization_id).await?;

    let summary = state.usecase.deal_summary(path.organization_id).await?;

    Ok(Response::OK(summary.into()))
}
