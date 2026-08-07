use auth::Identity;
use axum::{
    Extension,
    extract::{Query, State},
};
use handlers::{
    ApiError, AppState, DataEnvelope, Page, PaginationMetadata, PaginationParams, Response,
};

use crate::{require_org_membership, response::WorkOrderResponse, work_order::WorkOrdersPath};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/work-orders",
    operation_id = "listWorkOrders",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        PaginationParams,
    ),
    responses(
        (status = 200, description = "Paginated list of work orders", body = inline(DataEnvelope<Vec<WorkOrderResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: WorkOrdersPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Response<WorkOrderResponse>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;

    let per_page = pagination.per_page();
    let page = pagination.page();
    let offset = pagination.offset();
    let (work_orders, total) = state
        .usecase
        .list_work_orders(path.organization_id, per_page, offset)
        .await?;
    let items: Vec<WorkOrderResponse> = work_orders
        .into_iter()
        .map(WorkOrderResponse::from)
        .collect();
    let is_empty = items.is_empty();
    let meta = PaginationMetadata::new(per_page, page, Some(total), is_empty);

    Ok(Response::Paginated(Page::new(items, meta)))
}
