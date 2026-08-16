use chrono::{DateTime, Utc};
use common::CoreError;

use crate::{TaskComment, TaskCommentId, TaskId};

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait TaskCommentRepository: Send {
    fn insert(
        &mut self,
        comment: &TaskComment,
    ) -> impl Future<Output = Result<TaskComment, CoreError>> + Send;

    /// Excludes a logically deleted comment — mirrors `TaskRepository::find_by_id`'s
    /// own `deleted_at IS NULL` filter.
    fn find_by_id(
        &mut self,
        id: TaskCommentId,
    ) -> impl Future<Output = Result<Option<TaskComment>, CoreError>> + Send;

    /// A page of `task_id`'s thread, oldest first (mirrors the
    /// `(task_id, created_at)` index) — together with the thread's total
    /// count, so the caller can build pagination metadata without a second
    /// round trip. Mirrors `TaskRepository::list_by_organization`.
    fn list_by_task(
        &mut self,
        task_id: TaskId,
        limit: u64,
        offset: u64,
    ) -> impl Future<Output = Result<(Vec<TaskComment>, u64), CoreError>> + Send;

    fn update(
        &mut self,
        comment: &TaskComment,
    ) -> impl Future<Output = Result<TaskComment, CoreError>> + Send;

    /// Logical delete: sets `deleted_at`, the row stays — a comment
    /// disappears from the thread but is never physically erased.
    fn soft_delete(
        &mut self,
        id: TaskCommentId,
        deleted_at: DateTime<Utc>,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;
}
