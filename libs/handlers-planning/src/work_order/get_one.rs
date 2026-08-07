use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::{
    response::WorkOrderResponse,
    work_order::{WorkOrderPath, require_work_order},
};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/work-orders/{work_order_id}",
    operation_id = "getWorkOrder",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("work_order_id" = mestier_core::WorkOrderId, Path, description = "Work order identifier"),
    ),
    responses(
        (status = 200, description = "Work order details", body = inline(DataEnvelope<WorkOrderResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Work order not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    WorkOrderPath {
        organization_id,
        work_order_id,
    }: WorkOrderPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<WorkOrderResponse>, ApiError> {
    let work_order = require_work_order(&state, &identity, organization_id, work_order_id).await?;

    Ok(Response::OK(work_order.into()))
}
