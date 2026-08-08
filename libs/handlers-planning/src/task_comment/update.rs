use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::UpdateTaskCommentCommand;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::task_comment::{TaskCommentPath, TaskCommentResponse, require_task_comment};

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateTaskCommentRequest {
    pub body: String,
}

#[utoipa::path(
    patch,
    path = "/api/v1/organizations/{organization_id}/tasks/{task_id}/comments/{comment_id}",
    operation_id = "updateTaskComment",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("task_id" = mestier_core::TaskId, Path, description = "Task identifier"),
        ("comment_id" = mestier_core::TaskCommentId, Path, description = "Task comment identifier"),
    ),
    request_body = UpdateTaskCommentRequest,
    responses(
        (status = 200, description = "Comment updated", body = inline(DataEnvelope<TaskCommentResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden — only the comment's own author may edit it"),
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
    Json(payload): Json<UpdateTaskCommentRequest>,
) -> Result<Response<TaskCommentResponse>, ApiError> {
    require_task_comment(&state, &identity, organization_id, task_id, comment_id).await?;

    let actor = state
        .usecase
        .find_user_by_sub(identity.id())
        .await?
        .ok_or(ApiError::Forbidden)?;

    let comment = state
        .usecase
        .acting_as(actor.id)
        .update_task_comment(UpdateTaskCommentCommand {
            id: comment_id,
            acting_user_id: actor.id,
            body: payload.body,
        })
        .await?;

    // `update_task_comment` only ever succeeds when `actor.id` matches the
    // comment's own author (anything else comes back `Forbidden` above), so
    // `actor.name` is always the right display name here — no second
    // lookup needed — and `author_is_self` is always `true` for the same
    // reason.
    Ok(Response::OK(TaskCommentResponse::new(
        comment, actor.name, true,
    )))
}
