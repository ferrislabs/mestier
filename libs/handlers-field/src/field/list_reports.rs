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
    paths::FieldAssignmentReportsPath, resolve_field_actor, response::AssignmentReportResponse,
};

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct ListFieldAssignmentReportsQuery {
    #[serde(default)]
    pub resolution: Option<AssignmentReportResolution>,
}

/// The caller's own reports — resolved ones included, so a worker can see
/// that their word was acted on. Never filtered to pending only: that is
/// what makes people keep reporting.
#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/field/assignment-reports",
    operation_id = "listMyAssignmentReports",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ListFieldAssignmentReportsQuery,
        PaginationParams,
    ),
    responses(
        (status = 200, description = "The caller's own reports, most recent first", body = inline(DataEnvelope<Vec<AssignmentReportResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Caller is not an employee of this organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    FieldAssignmentReportsPath { organization_id }: FieldAssignmentReportsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(filter): Query<ListFieldAssignmentReportsQuery>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Response<AssignmentReportResponse>, ApiError> {
    let actor = resolve_field_actor(&state, &identity, organization_id).await?;

    let per_page = pagination.per_page();
    let page = pagination.page();
    let offset = pagination.offset();
    let (reports, total) = state
        .usecase
        .list_assignment_reports_by_reporter(
            organization_id,
            actor.member_id,
            filter.resolution,
            per_page,
            offset,
        )
        .await?;

    let items: Vec<AssignmentReportResponse> = reports.into_iter().map(Into::into).collect();
    let is_empty = items.is_empty();
    let meta = PaginationMetadata::new(per_page, page, Some(total), is_empty);

    Ok(Response::Paginated(Page::new(items, meta)))
}
