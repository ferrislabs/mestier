use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, Response};

use crate::task::{TaskPath, require_task};

#[utoipa::path(
    delete,
    path = "/api/v1/organizations/{organization_id}/tasks/{task_id}",
    operation_id = "deleteTask",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("task_id" = mestier_core::TaskId, Path, description = "Task identifier"),
    ),
    responses(
        (status = 204, description = "Task soft-deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Task not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    TaskPath {
        organization_id,
        task_id,
    }: TaskPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<()>, ApiError> {
    require_task(&state, &identity, organization_id, task_id).await?;
    state.usecase.soft_delete_task(task_id).await?;

    Ok(Response::NoContent)
}
