use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::CreateTaskCommentCommand;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::task_comment::{TaskCommentResponse, TaskCommentsPath, require_task_for_comments};

/// Carries `body` only — never an author. `author_user_id` is resolved from
/// the authenticated identity (`state.usecase.find_user_by_sub`), the same
/// way `require_org_membership` itself resolves the caller: a client that
/// could name its own author could publish under someone else's name. See
/// the planning module design doc's "Décision" on `task_comments`.
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateTaskCommentRequest {
    pub body: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/tasks/{task_id}/comments",
    operation_id = "createTaskComment",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("task_id" = mestier_core::TaskId, Path, description = "Task identifier"),
    ),
    request_body = CreateTaskCommentRequest,
    responses(
        (status = 201, description = "Comment created", body = inline(DataEnvelope<TaskCommentResponse>)),
        (status = 400, description = "Validation failed"),
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
    Json(payload): Json<CreateTaskCommentRequest>,
) -> Result<Response<TaskCommentResponse>, ApiError> {
    require_task_for_comments(&state, &identity, organization_id, task_id).await?;

    // Duplicates `require_org_membership`'s own `find_user_by_sub` lookup —
    // that helper only proves membership, it does not hand back the
    // resolved `User`, and this is the one place the comment's author
    // actually needs it.
    let author = state
        .usecase
        .find_user_by_sub(identity.id())
        .await?
        .ok_or(ApiError::Forbidden)?;

    let comment = state
        .usecase
        .create_task_comment(CreateTaskCommentCommand {
            organization_id,
            task_id,
            author_user_id: author.id,
            body: payload.body,
        })
        .await?;

    Ok(Response::Created(TaskCommentResponse::new(
        comment,
        author.name,
    )))
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    /// Locks in the invariant `CreateTaskCommentRequest`'s doc comment
    /// states: the wire type carries no field the client could use to name
    /// an author. Even a payload that *tries* to smuggle one in — as a
    /// hostile or confused client might — deserializes with that field
    /// silently dropped, because `CreateTaskCommentRequest` simply has
    /// nowhere to put it. The handler's actual `author_user_id` always
    /// comes from `state.usecase.find_user_by_sub(identity.id())`, never
    /// from this struct.
    #[test]
    fn an_attempted_author_field_in_the_payload_is_ignored() {
        let request: CreateTaskCommentRequest = serde_json::from_value(json!({
            "body": "Salut l'équipe",
            "author_user_id": "11111111-1111-1111-1111-111111111111",
            "author": { "id": "22222222-2222-2222-2222-222222222222" },
        }))
        .expect("unknown fields must not fail deserialization");

        assert_eq!(request.body, "Salut l'équipe");
    }

    #[test]
    fn a_minimal_payload_deserializes() {
        let request: CreateTaskCommentRequest =
            serde_json::from_value(json!({ "body": "Salut" })).unwrap();

        assert_eq!(request.body, "Salut");
    }
}
