use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::{
    response::TaskResponse,
    task::{TaskPath, require_task},
};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/tasks/{task_id}",
    operation_id = "getTask",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("task_id" = mestier_core::TaskId, Path, description = "Task identifier"),
    ),
    responses(
        (status = 200, description = "Task details", body = inline(DataEnvelope<TaskResponse>)),
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
) -> Result<Response<TaskResponse>, ApiError> {
    let task = require_task(&state, &identity, organization_id, task_id).await?;

    Ok(Response::OK(task.into()))
}
