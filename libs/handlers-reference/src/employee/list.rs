use auth::Identity;
use axum::{
    Extension,
    extract::{Query, State},
};
use handlers::{
    ApiError, AppState, DataEnvelope, Page, PaginationMetadata, PaginationParams, Response,
    resolve_user_id,
};
use mestier_core::application::policy;

use crate::{paths::EmployeeProfilesCollectionPath, response::EmployeeResponse};

/// Gated by `member.manage`, not plain organization membership — unlike the
/// other reference-data lists in this crate, `hourly_rate_cents` is
/// sensitive. See `MestierUseCase::list_employees`.
#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/employee-profiles",
    operation_id = "listEmployeeProfiles",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        PaginationParams,
    ),
    responses(
        (status = 200, description = "Paginated list of employee profiles", body = inline(DataEnvelope<Vec<EmployeeResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: EmployeeProfilesCollectionPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Response<EmployeeResponse>, ApiError> {
    let user_id = resolve_user_id(&state, &identity).await?;
    // TODO: thread JWT realm roles once Identity exposes them.
    let actor = policy::user_subject(user_id, Vec::new());

    let per_page = pagination.per_page();
    let page = pagination.page();
    let offset = pagination.offset();
    let (employees, total) = state
        .usecase
        .acting_as(user_id)
        .list_employees(actor, path.organization_id, per_page, offset)
        .await?;
    let items: Vec<EmployeeResponse> = employees.into_iter().map(EmployeeResponse::from).collect();
    let is_empty = items.is_empty();
    let meta = PaginationMetadata::new(per_page, page, Some(total), is_empty);

    Ok(Response::Paginated(Page::new(items, meta)))
}
