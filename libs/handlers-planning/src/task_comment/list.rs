use auth::Identity;
use axum::{
    Extension,
    extract::{Query, State},
};
use handlers::{
    ApiError, AppState, DataEnvelope, Page, PaginationMetadata, PaginationParams, Response,
};
use mestier_core::UserId;

use crate::task_comment::{
    TaskCommentResponse, TaskCommentsPath, display_name_for, require_task_for_comments,
};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/tasks/{task_id}/comments",
    operation_id = "listTaskComments",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("task_id" = mestier_core::TaskId, Path, description = "Task identifier"),
        PaginationParams,
    ),
    responses(
        (status = 200, description = "Paginated thread, oldest first", body = inline(DataEnvelope<Vec<TaskCommentResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Task not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    TaskCommentsPath {
        organization_id,
        task_id,
    }: TaskCommentsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(pagination): Query<PaginationParams>,
) -> Result<Response<TaskCommentResponse>, ApiError> {
    require_task_for_comments(&state, &identity, organization_id, task_id).await?;

    // Duplicates `require_org_membership`'s own `find_user_by_sub` lookup —
    // that helper only proves membership, it does not hand back the
    // resolved `User`, and this is the one place a *list* of comments needs
    // to know who's asking: `author_is_self` is computed per comment below
    // against this viewer, not against whichever `find_user_by_sub` call a
    // single-comment endpoint (`create`/`update`) already made for itself.
    let viewer = state
        .usecase
        .find_user_by_sub(identity.id())
        .await?
        .ok_or(ApiError::Forbidden)?;

    let per_page = pagination.per_page();
    let page = pagination.page();
    let offset = pagination.offset();
    let (comments, total) = state
        .usecase
        .list_task_comments(task_id, per_page, offset)
        .await?;

    // One grouped query for the whole page's authors, never one per
    // comment — mirrors `task::list`'s own batched call to
    // `list_task_labels_for_tasks`.
    let author_ids: Vec<UserId> = comments
        .iter()
        .map(|comment| comment.author_user_id)
        .collect();
    let authors = state.usecase.list_task_comment_authors(author_ids).await?;

    let items: Vec<TaskCommentResponse> = comments
        .into_iter()
        .map(|comment| {
            let display_name = display_name_for(&authors, comment.author_user_id);
            let author_is_self = comment.author_user_id == viewer.id;
            TaskCommentResponse::new(comment, display_name, author_is_self)
        })
        .collect();
    let is_empty = items.is_empty();
    let meta = PaginationMetadata::new(per_page, page, Some(total), is_empty);

    Ok(Response::Paginated(Page::new(items, meta)))
}
