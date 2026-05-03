use axum::extract::{Query, State};
use handlers::{
    ApiError, AppState, DataEnvelope, Page, PaginationMetadata, PaginationParams, Response,
};

use crate::{paths::OrganizationsPath, response::OrganizationResponse};

#[utoipa::path(
    get,
    path = "/api/v1/organizations",
    operation_id = "listOrganizations",
    tag = super::TAG,
    params(PaginationParams),
    responses(
        (status = 200, description = "Paginated list of organizations", body = inline(DataEnvelope<Vec<OrganizationResponse>>)),
        (status = 401, description = "Unauthorized"),
    ),
    security(("bearer_auth" = []))
)]
#[tracing::instrument(skip_all, fields(page = pagination.page(), per_page = pagination.per_page()), err)]
pub async fn handler(
    _: OrganizationsPath,
    State(state): State<AppState>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Response<OrganizationResponse>, ApiError> {
    let per_page = pagination.per_page();
    let page = pagination.page();
    let offset = pagination.offset();

    let (organizations, total) = state.usecase.list_organizations(per_page, offset).await?;

    let items: Vec<OrganizationResponse> = organizations
        .into_iter()
        .map(OrganizationResponse::from)
        .collect();
    let is_empty = items.is_empty();
    let meta = PaginationMetadata::new(per_page, page, Some(total), is_empty);

    Ok(Response::Paginated(Page::new(items, meta)))
}
