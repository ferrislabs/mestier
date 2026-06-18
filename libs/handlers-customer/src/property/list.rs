use auth::Identity;
use axum::{
    Extension,
    extract::{Query, State},
};
use handlers::{
    ApiError, AppState, DataEnvelope, Page, PaginationMetadata, PaginationParams, Response,
};

use crate::{paths::PropertiesPath, require_customer_membership, response::PropertyResponse};

#[utoipa::path(
    get,
    path = "/api/v1/customers/{customer_id}/properties",
    operation_id = "listProperties",
    tag = super::super::TAG,
    params(
        ("customer_id" = mestier_core::CustomerId, Path, description = "Customer identifier"),
        PaginationParams,
    ),
    responses(
        (status = 200, description = "Paginated list of properties", body = inline(DataEnvelope<Vec<PropertyResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Customer not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: PropertiesPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Response<PropertyResponse>, ApiError> {
    require_customer_membership(&state, &identity, path.customer_id).await?;

    let per_page = pagination.per_page();
    let page = pagination.page();
    let offset = pagination.offset();
    let (properties, total) = state
        .usecase
        .list_properties(path.customer_id, per_page, offset)
        .await?;
    let items: Vec<PropertyResponse> = properties.into_iter().map(PropertyResponse::from).collect();
    let is_empty = items.is_empty();
    let meta = PaginationMetadata::new(per_page, page, Some(total), is_empty);

    Ok(Response::Paginated(Page::new(items, meta)))
}
