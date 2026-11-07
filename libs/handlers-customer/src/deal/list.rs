use auth::Identity;
use axum::{
    Extension,
    extract::{Query, State},
};
use handlers::{
    ApiError, AppState, DataEnvelope, Page, PaginationMetadata, PaginationParams, Response,
};
use mestier_core::{CustomerId, DealFilter, DealStageName};
use serde::Deserialize;
use utoipa::IntoParams;

use crate::{DEALS_TAG, paths::DealsPath, require_view_customers, response::DealResponse};

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct DealListFilter {
    customer_id: Option<CustomerId>,
    stage: Option<DealStageName>,
    q: Option<String>,
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/deals",
    operation_id = "listDeals",
    tag = DEALS_TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        PaginationParams,
        DealListFilter,
    ),
    responses(
        (status = 200, description = "Paginated list of deals", body = inline(DataEnvelope<Vec<DealResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: DealsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(pagination): Query<PaginationParams>,
    Query(filter): Query<DealListFilter>,
) -> Result<Response<DealResponse>, ApiError> {
    require_view_customers(&state, &identity, path.organization_id).await?;

    let per_page = pagination.per_page();
    let page = pagination.page();
    let offset = pagination.offset();
    let query = filter
        .q
        .map(|text| text.trim().to_owned())
        .filter(|text| !text.is_empty());
    let (deals, total) = state
        .usecase
        .list_deals(
            DealFilter {
                organization_id: path.organization_id,
                customer_id: filter.customer_id,
                stage: filter.stage,
                query,
            },
            per_page,
            offset,
        )
        .await?;
    let items: Vec<DealResponse> = deals.into_iter().map(DealResponse::from).collect();
    let is_empty = items.is_empty();
    let meta = PaginationMetadata::new(per_page, page, Some(total), is_empty);

    Ok(Response::Paginated(Page::new(items, meta)))
}
