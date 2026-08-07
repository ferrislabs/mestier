use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, Response};

use crate::task_label::{TaskLabelPath, require_task_label};

#[utoipa::path(
    delete,
    path = "/api/v1/organizations/{organization_id}/task-labels/{label_id}",
    operation_id = "deleteTaskLabel",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("label_id" = mestier_core::TaskLabelId, Path, description = "Task label identifier"),
    ),
    responses(
        (status = 204, description = "Task label deleted — removed from every task that carried it"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Task label not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    TaskLabelPath {
        organization_id,
        label_id,
    }: TaskLabelPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<()>, ApiError> {
    require_task_label(&state, &identity, organization_id, label_id).await?;
    state.usecase.delete_task_label(label_id).await?;

    Ok(Response::NoContent)
}
