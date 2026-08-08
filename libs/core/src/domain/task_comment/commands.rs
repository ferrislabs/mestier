use crate::{OrganizationId, TaskId, UserId, domain::task_comment::TaskCommentId};

/// `author_user_id` is resolved by the caller from the authenticated
/// identity (`state.usecase.find_user_by_sub`) — never carried by an HTTP
/// request payload. A client that could name its own author could publish
/// under someone else's name; see the planning module design doc's
/// "Décision" on `task_comments.author_user_id`.
#[derive(Debug, Clone)]
pub struct CreateTaskCommentCommand {
    pub organization_id: OrganizationId,
    pub task_id: TaskId,
    pub author_user_id: UserId,
    pub body: String,
}

/// `acting_user_id` is the authenticated caller, checked against the
/// comment's own `author_user_id` by
/// [`crate::domain::task_comment::service::TaskCommentService::update_task_comment`]
/// — only the author may edit their own comment.
#[derive(Debug, Clone)]
pub struct UpdateTaskCommentCommand {
    pub id: TaskCommentId,
    pub acting_user_id: UserId,
    pub body: String,
}
