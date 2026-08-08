use std::collections::{HashMap, HashSet};

use common::CoreError;
use mestier_macros::transactional;

use crate::{
    TaskComment, TaskCommentId, TaskId, User, UserId,
    application::MestierUseCase,
    domain::task_comment::{
        commands::{CreateTaskCommentCommand, UpdateTaskCommentCommand},
        service::TaskCommentService,
    },
    domain::user::ports::UserRepository,
};

mod tests;

impl MestierUseCase {
    #[transactional(task_comment)]
    pub async fn create_task_comment(
        &self,
        command: CreateTaskCommentCommand,
    ) -> Result<TaskComment, CoreError> {
        let mut service = TaskCommentService::new(task_comment_repository);
        service.create_task_comment(command).await
    }

    #[transactional(task_comment)]
    pub async fn get_task_comment(&self, id: TaskCommentId) -> Result<TaskComment, CoreError> {
        let mut service = TaskCommentService::new(task_comment_repository);
        service.get_task_comment(id).await
    }

    /// A page of `task_id`'s thread, oldest first, together with the
    /// thread's total count — see `TaskCommentRepository::list_by_task`.
    #[transactional(task_comment)]
    pub async fn list_task_comments(
        &self,
        task_id: TaskId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<TaskComment>, u64), CoreError> {
        let mut service = TaskCommentService::new(task_comment_repository);
        service.list_task_comments(task_id, limit, offset).await
    }

    #[transactional(task_comment)]
    pub async fn update_task_comment(
        &self,
        command: UpdateTaskCommentCommand,
    ) -> Result<TaskComment, CoreError> {
        let mut service = TaskCommentService::new(task_comment_repository);
        service.update_task_comment(command).await
    }

    #[transactional(task_comment)]
    pub async fn delete_task_comment(
        &self,
        id: TaskCommentId,
        acting_user_id: UserId,
    ) -> Result<(), CoreError> {
        let mut service = TaskCommentService::new(task_comment_repository);
        service.delete_task_comment(id, acting_user_id).await
    }

    /// Every author of a page of comments, resolved in one query — mirrors
    /// `list_task_labels_for_tasks`'s own batching (see its N+1 warning,
    /// itself mirrored from `TaskRepository::count_children`). Lives in
    /// this workstream's own file rather than in `application/user/mod.rs`,
    /// which this workstream does not own: the call exists only to serve
    /// `GET .../comments`, which needs to render each comment's author
    /// without one lookup per comment (see the planning module design
    /// doc's "the author must be displayable" requirement).
    #[transactional(user)]
    pub async fn list_task_comment_authors(
        &self,
        user_ids: Vec<UserId>,
    ) -> Result<HashMap<UserId, User>, CoreError> {
        let mut user_repository = user_repository;

        let mut seen = HashSet::new();
        let deduped: Vec<UserId> = user_ids.into_iter().filter(|id| seen.insert(*id)).collect();

        let users = user_repository.list_by_ids(&deduped).await?;
        Ok(users.into_iter().map(|user| (user.id, user)).collect())
    }
}
