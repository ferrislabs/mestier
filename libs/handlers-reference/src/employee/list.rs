use auth::Identity;
use axum::{
    Extension,
    extract::{Query, State},
};
use handlers::{
    ApiError, AppState, DataEnvelope, Page, PaginationMetadata, PaginationParams, Response,
};

use crate::{paths::EmployeesPath, require_org_membership, response::EmployeeResponse};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/employees",
    operation_id = "listEmployees",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        PaginationParams,
    ),
    responses(
        (status = 200, description = "Paginated list of employees", body = inline(DataEnvelope<Vec<EmployeeResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
#[tracing::instrument(skip_all, fields(organization_id = %path.organization_id.0, page = pagination.page(), per_page = pagination.per_page()), err)]
pub async fn handler(
    path: EmployeesPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Response<EmployeeResponse>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;

    let per_page = pagination.per_page();
    let page = pagination.page();
    let offset = pagination.offset();
    let (employees, total) = state
        .usecase
        .list_employees(path.organization_id, per_page, offset)
        .await?;
    let items: Vec<EmployeeResponse> = employees.into_iter().map(EmployeeResponse::from).collect();
    let is_empty = items.is_empty();
    let meta = PaginationMetadata::new(per_page, page, Some(total), is_empty);

    Ok(Response::Paginated(Page::new(items, meta)))
}
