use auth::Identity;
use axum::{
    Extension,
    extract::{Query, State},
};
use handlers::{
    ApiError, AppState, DataEnvelope, Page, PaginationMetadata, PaginationParams, Response,
};

use crate::{
    paths::CustomerContextsPath, require_customer_view, response::CustomerContextResponse,
};

#[utoipa::path(
    get,
    path = "/api/v1/customers/{customer_id}/customer-contexts",
    operation_id = "listCustomerContexts",
    tag = super::super::TAG,
    params(
        ("customer_id" = mestier_core::CustomerId, Path, description = "Customer identifier"),
        PaginationParams,
    ),
    responses(
        (status = 200, description = "Paginated list of customer contexts", body = inline(DataEnvelope<Vec<CustomerContextResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Customer not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: CustomerContextsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Response<CustomerContextResponse>, ApiError> {
    require_customer_view(&state, &identity, path.customer_id).await?;

    let per_page = pagination.per_page();
    let page = pagination.page();
    let offset = pagination.offset();
    let (customer_contexts, total) = state
        .usecase
        .list_customer_contexts(path.customer_id, per_page, offset)
        .await?;
    let items: Vec<CustomerContextResponse> = customer_contexts
        .into_iter()
        .map(CustomerContextResponse::from)
        .collect();
    let is_empty = items.is_empty();
    let meta = PaginationMetadata::new(per_page, page, Some(total), is_empty);

    Ok(Response::Paginated(Page::new(items, meta)))
}
