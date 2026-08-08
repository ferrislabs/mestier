//! Task comments — a task's discussion thread, the annotable half of the
//! planning remodel's "a task resembles a GitHub issue" framing (see the
//! planning module design doc). All four endpoints are nested under the
//! task, `PATCH`/`DELETE` included, and mirror the `absence`/`work_order`
//! submodule layout: one file per verb.
//!
//! The author is always resolved from the authenticated identity
//! (`state.usecase.find_user_by_sub`), never from the request payload — see
//! `create::CreateTaskCommentRequest`'s doc comment. Only a comment's own
//! author may `PATCH`/`DELETE` it; that rule is enforced in
//! `TaskCommentService`, not here, so it applies uniformly whatever adapter
//! calls it.

use auth::Identity;
use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use chrono::{DateTime, Utc};
use handlers::{ApiError, AppState};
use mestier_core::{OrganizationId, TaskComment, TaskCommentId, TaskId, UserId};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::require_org_membership;

pub mod create;
pub mod delete;
pub mod list;
pub mod update;

/// This aggregate's routes, unlayered — the crate-level `router` in
/// `lib.rs` merges every aggregate submodule's router before applying the
/// shared rate-limit/auth middleware once, rather than each submodule
/// layering its own.
pub fn router(_state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_get(list::handler)
        .typed_post(create::handler)
        .typed_patch(update::handler)
        .typed_delete(delete::handler)
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/tasks/{task_id}/comments")]
pub struct TaskCommentsPath {
    pub organization_id: OrganizationId,
    pub task_id: TaskId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/tasks/{task_id}/comments/{comment_id}")]
pub struct TaskCommentPath {
    pub organization_id: OrganizationId,
    pub task_id: TaskId,
    pub comment_id: TaskCommentId,
}

/// Checks org membership and that `task_id` actually belongs to
/// `organization_id` — every comment endpoint needs this before doing
/// anything else. Mirrors `crate::task::require_task`, duplicated rather
/// than shared: this workstream owns no file `task::require_task` lives
/// in, and the two crates already duplicate `require_org_membership`
/// itself the same way.
pub(crate) async fn require_task_for_comments(
    state: &AppState,
    identity: &Identity,
    organization_id: OrganizationId,
    task_id: TaskId,
) -> Result<(), ApiError> {
    require_org_membership(state, identity, organization_id).await?;

    let task = state.usecase.get_task(task_id).await?;
    if task.organization_id != organization_id {
        return Err(ApiError::NotFound);
    }

    Ok(())
}

/// Loads the comment and checks that it actually belongs to `task_id` (and,
/// transitively via [`require_task_for_comments`], to `organization_id`) —
/// a real `comment_id` from a different task is treated as "does not
/// exist" rather than leaking cross-task existence via 403. Does **not**
/// check authorship: that is `TaskCommentService::update_task_comment`'s/
/// `delete_task_comment`'s own job, so a `PATCH`/`DELETE` by a member who
/// is not the author still reaches the service and gets a `Forbidden`
/// there. Mirrors `crate::task_label::require_task_label`.
pub(crate) async fn require_task_comment(
    state: &AppState,
    identity: &Identity,
    organization_id: OrganizationId,
    task_id: TaskId,
    comment_id: TaskCommentId,
) -> Result<TaskComment, ApiError> {
    require_task_for_comments(state, identity, organization_id, task_id).await?;

    let comment = state.usecase.get_task_comment(comment_id).await?;
    if comment.organization_id != organization_id || comment.task_id != task_id {
        return Err(ApiError::NotFound);
    }

    Ok(comment)
}

/// The author, projected down to what the front needs to render a name —
/// see the planning module design doc's "a thread that only shows user ids
/// is unusable" requirement. `display_name` falls back to `"Unknown"` when
/// the batched author lookup somehow comes up empty for this id (the
/// author's `users` row was hard-deleted out from under an FK that allows
/// it, or similar): one odd row should not take the whole response down.
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct TaskCommentAuthorResponse {
    pub id: UserId,
    pub display_name: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct TaskCommentResponse {
    pub id: TaskCommentId,
    pub organization_id: OrganizationId,
    pub task_id: TaskId,
    pub author: TaskCommentAuthorResponse,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    /// Whether the caller who requested this response is this comment's own
    /// author — resolved here, server-side, from the authenticated identity,
    /// rather than left for the frontend to guess by comparing display
    /// names (which two members sharing a name would each pass for the
    /// other's comment). This field only decides whether the UI *offers*
    /// the edit/delete actions; `TaskCommentService::update_task_comment`/
    /// `delete_task_comment` still enforce authorship independently, so a
    /// forged request never gets further than `403` regardless of what this
    /// says.
    pub author_is_self: bool,
}

impl TaskCommentResponse {
    pub fn new(comment: TaskComment, author_display_name: String, author_is_self: bool) -> Self {
        Self {
            id: comment.id,
            organization_id: comment.organization_id,
            task_id: comment.task_id,
            author: TaskCommentAuthorResponse {
                id: comment.author_user_id,
                display_name: author_display_name,
            },
            body: comment.body,
            created_at: comment.created_at,
            updated_at: comment.updated_at,
            author_is_self,
        }
    }
}

/// Falls back to `"Unknown"` rather than failing the whole request — mirrors
/// `PlanningService::load_resources`'s own fallback for a member-only
/// resource whose `users` row cannot be found.
pub(crate) fn display_name_for(
    authors: &std::collections::HashMap<UserId, mestier_core::User>,
    author_user_id: UserId,
) -> String {
    authors
        .get(&author_user_id)
        .map(|user| user.name.clone())
        .unwrap_or_else(|| "Unknown".to_owned())
}

#[cfg(test)]
mod response_tests {
    use super::*;
    use chrono::Utc;

    fn comment(author_user_id: UserId) -> TaskComment {
        let now = Utc::now();
        TaskComment {
            id: "33333333-3333-3333-3333-333333333333".parse().unwrap(),
            organization_id: "44444444-4444-4444-4444-444444444444".parse().unwrap(),
            task_id: "55555555-5555-5555-5555-555555555555".parse().unwrap(),
            author_user_id,
            body: "Salut".to_owned(),
            deleted_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    /// Locks in that `author_is_self` is carried through verbatim from the
    /// caller — the actual "is it really them" resolution happens at each
    /// handler's call site (comparing the authenticated identity's user id
    /// against `comment.author_user_id`), not inside this constructor.
    #[test]
    fn author_is_self_is_passed_through_as_given() {
        let author_user_id = "66666666-6666-6666-6666-666666666666".parse().unwrap();

        let own = TaskCommentResponse::new(comment(author_user_id), "Alice".to_owned(), true);
        let not_own = TaskCommentResponse::new(comment(author_user_id), "Alice".to_owned(), false);

        assert!(own.author_is_self);
        assert!(!not_own.author_is_self);
    }
}
