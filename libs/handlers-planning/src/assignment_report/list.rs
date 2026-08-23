use auth::Identity;
use axum::{
    Extension,
    extract::{Query, State},
};
use handlers::{
    ApiError, AppState, DataEnvelope, Page, PaginationMetadata, PaginationParams, Response,
};
use mestier_core::AssignmentReportResolution;
use serde::Deserialize;
use utoipa::IntoParams;

use crate::{
    assignment_report::{AssignmentReportResponse, AssignmentReportsPath},
    require_org_membership,
};

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ListAssignmentReportsQuery {
    /// Defaults to `PENDING` when absent — the list a manager opens is the
    /// one that needs a decision, not the whole history.
    #[serde(default)]
    pub resolution: Option<AssignmentReportResolution>,
}

/// The organization's reports, most recent first.
///
/// Defaults to pending: this is the list a manager works through, and
/// filtering it away by default would bury the one thing it exists to
/// surface. `?resolution=` widens it to any state, including the whole
/// history.
#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/assignment-reports",
    operation_id = "listAssignmentReports",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ListAssignmentReportsQuery,
        PaginationParams,
    ),
    responses(
        (status = 200, description = "Paginated reports, defaulting to pending", body = inline(DataEnvelope<Vec<AssignmentReportResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    AssignmentReportsPath { organization_id }: AssignmentReportsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(filter): Query<ListAssignmentReportsQuery>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Response<AssignmentReportResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;

    let resolution = Some(
        filter
            .resolution
            .unwrap_or(AssignmentReportResolution::Pending),
    );
    let per_page = pagination.per_page();
    let page = pagination.page();
    let offset = pagination.offset();
    let (reports, total) = state
        .usecase
        .list_assignment_reports_by_organization(organization_id, resolution, per_page, offset)
        .await?;

    let items: Vec<AssignmentReportResponse> = reports.into_iter().map(Into::into).collect();
    let is_empty = items.is_empty();
    let meta = PaginationMetadata::new(per_page, page, Some(total), is_empty);

    Ok(Response::Paginated(Page::new(items, meta)))
}
