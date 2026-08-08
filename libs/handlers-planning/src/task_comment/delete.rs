use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, Response, resolve_user_id};

use crate::task_comment::{TaskCommentPath, require_task_comment};

#[utoipa::path(
    delete,
    path = "/api/v1/organizations/{organization_id}/tasks/{task_id}/comments/{comment_id}",
    operation_id = "deleteTaskComment",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("task_id" = mestier_core::TaskId, Path, description = "Task identifier"),
        ("comment_id" = mestier_core::TaskCommentId, Path, description = "Task comment identifier"),
    ),
    responses(
        (status = 204, description = "Comment removed from the thread — the row survives, deleted_at is set"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden — only the comment's own author may delete it"),
        (status = 404, description = "Task or comment not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    TaskCommentPath {
        organization_id,
        task_id,
        comment_id,
    }: TaskCommentPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<()>, ApiError> {
    require_task_comment(&state, &identity, organization_id, task_id, comment_id).await?;
    let actor = resolve_user_id(&state, &identity).await?;

    state
        .usecase
        .acting_as(actor)
        .delete_task_comment(comment_id, actor)
        .await?;

    Ok(Response::NoContent)
}
