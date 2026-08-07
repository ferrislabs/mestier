use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, Response};

use crate::work_order::{WorkOrderPath, require_work_order};

#[utoipa::path(
    delete,
    path = "/api/v1/organizations/{organization_id}/work-orders/{work_order_id}",
    operation_id = "deleteWorkOrder",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("work_order_id" = mestier_core::WorkOrderId, Path, description = "Work order identifier"),
    ),
    responses(
        (status = 204, description = "Work order soft-deleted"),
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
) -> Result<Response<()>, ApiError> {
    require_work_order(&state, &identity, organization_id, work_order_id).await?;
    state.usecase.soft_delete_work_order(work_order_id).await?;

    Ok(Response::NoContent)
}
