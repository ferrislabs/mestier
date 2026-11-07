use auth::Identity;
use axum::{
    Extension,
    extract::{Query, State},
};
use handlers::{
    ApiError, AppState, DataEnvelope, Page, PaginationMetadata, PaginationParams, Response,
};

use crate::{paths::CustomersPath, require_view_customers, response::CustomerListItemResponse};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/customers",
    operation_id = "listCustomers",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        PaginationParams,
    ),
    responses(
        (status = 200, description = "Paginated list of customers", body = inline(DataEnvelope<Vec<CustomerListItemResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: CustomersPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Response<CustomerListItemResponse>, ApiError> {
    require_view_customers(&state, &identity, path.organization_id).await?;

    let per_page = pagination.per_page();
    let page = pagination.page();
    let offset = pagination.offset();
    let (rows, total) = state
        .usecase
        .list_customer_rows(path.organization_id, per_page, offset)
        .await?;
    let items: Vec<CustomerListItemResponse> = rows
        .into_iter()
        .map(CustomerListItemResponse::from)
        .collect();
    let is_empty = items.is_empty();
    let meta = PaginationMetadata::new(per_page, page, Some(total), is_empty);

    Ok(Response::Paginated(Page::new(items, meta)))
}
