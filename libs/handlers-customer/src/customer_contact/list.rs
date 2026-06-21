use auth::Identity;
use axum::{
    Extension,
    extract::{Query, State},
};
use handlers::{
    ApiError, AppState, DataEnvelope, Page, PaginationMetadata, PaginationParams, Response,
};

use crate::{
    paths::CustomerContactsPath, require_customer_membership, response::CustomerContactResponse,
};

#[utoipa::path(
    get,
    path = "/api/v1/customers/{customer_id}/contacts",
    operation_id = "listCustomerContacts",
    tag = super::super::TAG,
    params(
        ("customer_id" = mestier_core::CustomerId, Path, description = "Customer identifier"),
        PaginationParams,
    ),
    responses(
        (status = 200, description = "Paginated list of customer contacts", body = inline(DataEnvelope<Vec<CustomerContactResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Customer not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: CustomerContactsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Response<CustomerContactResponse>, ApiError> {
    require_customer_membership(&state, &identity, path.customer_id).await?;

    let per_page = pagination.per_page();
    let page = pagination.page();
    let offset = pagination.offset();
    let (customer_contacts, total) = state
        .usecase
        .list_customer_contacts(path.customer_id, per_page, offset)
        .await?;
    let items: Vec<CustomerContactResponse> = customer_contacts
        .into_iter()
        .map(CustomerContactResponse::from)
        .collect();
    let is_empty = items.is_empty();
    let meta = PaginationMetadata::new(per_page, page, Some(total), is_empty);

    Ok(Response::Paginated(Page::new(items, meta)))
}
