use chrono::Utc;
use common::{CoreError, generate_uuid_v7};

use crate::{
    TaskComment, TaskCommentId, TaskId, UserId,
    domain::task_comment::{
        commands::{CreateTaskCommentCommand, UpdateTaskCommentCommand},
        ports::TaskCommentRepository,
    },
};

/// Mirrors `chk_task_comments_body_not_blank`, ahead of the trip to the
/// database: a comment made only of whitespace carries nothing worth
/// reading.
fn validate_body(body: &str) -> Result<(), CoreError> {
    if body.trim().is_empty() {
        return Err(CoreError::Conflict(
            "task comment body cannot be blank".to_owned(),
        ));
    }

    Ok(())
}

pub struct TaskCommentService<R>
where
    R: TaskCommentRepository,
{
    repo: R,
}

impl<R> TaskCommentService<R>
where
    R: TaskCommentRepository,
{
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn create_task_comment(
        &mut self,
        command: CreateTaskCommentCommand,
    ) -> Result<TaskComment, CoreError> {
        validate_body(&command.body)?;

        let now = Utc::now();
        self.repo
            .insert(&TaskComment {
                id: TaskCommentId(generate_uuid_v7()),
                organization_id: command.organization_id,
                task_id: command.task_id,
                author_user_id: command.author_user_id,
                body: command.body,
                deleted_at: None,
                created_at: now,
                updated_at: now,
            })
            .await
    }

    pub async fn get_task_comment(&mut self, id: TaskCommentId) -> Result<TaskComment, CoreError> {
        self.repo.find_by_id(id).await?.ok_or(CoreError::NotFound)
    }

    pub async fn list_task_comments(
        &mut self,
        task_id: TaskId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<TaskComment>, u64), CoreError> {
        self.repo.list_by_task(task_id, limit, offset).await
    }

    /// Only the comment's own author may edit it — the safest rule when
    /// fine-grained permissions are out of scope: it grants nothing to
    /// anyone else rather than guess a moderator role. See the planning
    /// module design doc's "Décision" on `task_comments`.
    pub async fn update_task_comment(
        &mut self,
        command: UpdateTaskCommentCommand,
    ) -> Result<TaskComment, CoreError> {
        validate_body(&command.body)?;

        let mut comment = self.get_task_comment(command.id).await?;
        if comment.author_user_id != command.acting_user_id {
            return Err(CoreError::Forbidden {
                reason: Some("only the comment's author may edit it".to_owned()),
            });
        }

        comment.body = command.body;
        comment.updated_at = Utc::now();

        self.repo.update(&comment).await
    }

    /// Same authorship rule as [`Self::update_task_comment`]: only the
    /// author may delete their own comment.
    pub async fn delete_task_comment(
        &mut self,
        id: TaskCommentId,
        acting_user_id: UserId,
    ) -> Result<(), CoreError> {
        let comment = self.get_task_comment(id).await?;
        if comment.author_user_id != acting_user_id {
            return Err(CoreError::Forbidden {
                reason: Some("only the comment's author may delete it".to_owned()),
            });
        }

        self.repo.soft_delete(id, Utc::now()).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{OrganizationId, domain::task_comment::ports::MockTaskCommentRepository};
    use mockall::predicate::eq;
    use uuid::Uuid;

    fn comment(id: TaskCommentId, author_user_id: UserId) -> TaskComment {
        let now = Utc::now();
        TaskComment {
            id,
            organization_id: OrganizationId(Uuid::new_v4()),
            task_id: TaskId(Uuid::new_v4()),
            author_user_id,
            body: "Salut l'équipe".to_owned(),
            deleted_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    fn create_command(author_user_id: UserId) -> CreateTaskCommentCommand {
        CreateTaskCommentCommand {
            organization_id: OrganizationId(Uuid::new_v4()),
            task_id: TaskId(Uuid::new_v4()),
            author_user_id,
            body: "Salut l'équipe".to_owned(),
        }
    }

    // -- create_task_comment -------------------------------------------------

    #[tokio::test]
    async fn create_task_comment_persists_via_repo() {
        let author_user_id = UserId(Uuid::new_v4());
        let mut repo = MockTaskCommentRepository::new();
        repo.expect_insert().times(1).returning(|c| {
            let cloned = c.clone();
            Box::pin(async move { Ok(cloned) })
        });

        let mut service = TaskCommentService::new(repo);
        let created = service
            .create_task_comment(create_command(author_user_id))
            .await
            .unwrap();

        assert_eq!(created.body, "Salut l'équipe");
        assert_eq!(created.author_user_id, author_user_id);
        assert!(created.deleted_at.is_none());
    }

    #[tokio::test]
    async fn create_task_comment_rejects_a_blank_body() {
        let mut service = TaskCommentService::new(MockTaskCommentRepository::new());

        let mut command = create_command(UserId(Uuid::new_v4()));
        command.body = "".to_owned();

        let err = service.create_task_comment(command).await.unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn create_task_comment_rejects_a_whitespace_only_body() {
        let mut service = TaskCommentService::new(MockTaskCommentRepository::new());

        let mut command = create_command(UserId(Uuid::new_v4()));
        command.body = "   \n\t  ".to_owned();

        let err = service.create_task_comment(command).await.unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    // -- get_task_comment ------------------------------------------------------

    #[tokio::test]
    async fn get_task_comment_returns_not_found_when_missing() {
        let id = TaskCommentId(Uuid::new_v4());
        let mut repo = MockTaskCommentRepository::new();
        repo.expect_find_by_id()
            .with(eq(id))
            .returning(|_| Box::pin(async { Ok(None) }));

        let mut service = TaskCommentService::new(repo);
        let err = service.get_task_comment(id).await.unwrap_err();

        assert!(matches!(err, CoreError::NotFound));
    }

    #[tokio::test]
    async fn get_task_comment_returns_entity_when_found() {
        let id = TaskCommentId(Uuid::new_v4());
        let author_user_id = UserId(Uuid::new_v4());
        let mut repo = MockTaskCommentRepository::new();
        repo.expect_find_by_id().with(eq(id)).returning(move |_| {
            let c = comment(id, author_user_id);
            Box::pin(async move { Ok(Some(c)) })
        });

        let mut service = TaskCommentService::new(repo);
        let found = service.get_task_comment(id).await.unwrap();

        assert_eq!(found.id, id);
    }

    // -- list_task_comments ----------------------------------------------------

    #[tokio::test]
    async fn list_task_comments_delegates_to_repo() {
        let task_id = TaskId(Uuid::new_v4());
        let author_user_id = UserId(Uuid::new_v4());
        let mut repo = MockTaskCommentRepository::new();
        repo.expect_list_by_task()
            .withf(move |t, limit, offset| *t == task_id && *limit == 20 && *offset == 0)
            .returning(move |_, _, _| {
                let c = comment(TaskCommentId(Uuid::new_v4()), author_user_id);
                Box::pin(async move { Ok((vec![c], 1)) })
            });

        let mut service = TaskCommentService::new(repo);
        let (items, total) = service.list_task_comments(task_id, 20, 0).await.unwrap();

        assert_eq!(items.len(), 1);
        assert_eq!(total, 1);
    }

    // -- update_task_comment -----------------------------------------------------

    #[tokio::test]
    async fn update_task_comment_mutates_the_body_when_the_author_matches() {
        let id = TaskCommentId(Uuid::new_v4());
        let author_user_id = UserId(Uuid::new_v4());
        let existing = comment(id, author_user_id);

        let mut repo = MockTaskCommentRepository::new();
        repo.expect_find_by_id().with(eq(id)).returning(move |_| {
            let c = existing.clone();
            Box::pin(async move { Ok(Some(c)) })
        });
        repo.expect_update()
            .withf(|c| c.body == "Édité")
            .returning(|c| {
                let cloned = c.clone();
                Box::pin(async move { Ok(cloned) })
            });

        let mut service = TaskCommentService::new(repo);
        let updated = service
            .update_task_comment(UpdateTaskCommentCommand {
                id,
                acting_user_id: author_user_id,
                body: "Édité".to_owned(),
            })
            .await
            .unwrap();

        assert_eq!(updated.body, "Édité");
    }

    #[tokio::test]
    async fn update_task_comment_rejects_a_blank_body() {
        let mut service = TaskCommentService::new(MockTaskCommentRepository::new());

        let err = service
            .update_task_comment(UpdateTaskCommentCommand {
                id: TaskCommentId(Uuid::new_v4()),
                acting_user_id: UserId(Uuid::new_v4()),
                body: "   ".to_owned(),
            })
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    /// The security rule this workstream exists to enforce: editing someone
    /// else's comment must be refused, not silently allowed or downgraded
    /// to "not found".
    #[tokio::test]
    async fn update_task_comment_forbids_editing_someone_elses_comment() {
        let id = TaskCommentId(Uuid::new_v4());
        let author_user_id = UserId(Uuid::new_v4());
        let other_user_id = UserId(Uuid::new_v4());
        let existing = comment(id, author_user_id);

        let mut repo = MockTaskCommentRepository::new();
        repo.expect_find_by_id().with(eq(id)).returning(move |_| {
            let c = existing.clone();
            Box::pin(async move { Ok(Some(c)) })
        });
        // No `expect_update`: a non-author's PATCH must be rejected before
        // any write is attempted.

        let mut service = TaskCommentService::new(repo);
        let err = service
            .update_task_comment(UpdateTaskCommentCommand {
                id,
                acting_user_id: other_user_id,
                body: "Je réécris ton message".to_owned(),
            })
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Forbidden { .. }));
    }

    // -- delete_task_comment -------------------------------------------------

    #[tokio::test]
    async fn delete_task_comment_soft_deletes_when_the_author_matches() {
        let id = TaskCommentId(Uuid::new_v4());
        let author_user_id = UserId(Uuid::new_v4());
        let existing = comment(id, author_user_id);

        let mut repo = MockTaskCommentRepository::new();
        repo.expect_find_by_id().with(eq(id)).returning(move |_| {
            let c = existing.clone();
            Box::pin(async move { Ok(Some(c)) })
        });
        repo.expect_soft_delete()
            .withf(move |deleted_id, _| *deleted_id == id)
            .returning(|_, _| Box::pin(async { Ok(()) }));

        let mut service = TaskCommentService::new(repo);
        service
            .delete_task_comment(id, author_user_id)
            .await
            .unwrap();
    }

    /// Same security rule as the `PATCH` test, for `DELETE`.
    #[tokio::test]
    async fn delete_task_comment_forbids_deleting_someone_elses_comment() {
        let id = TaskCommentId(Uuid::new_v4());
        let author_user_id = UserId(Uuid::new_v4());
        let other_user_id = UserId(Uuid::new_v4());
        let existing = comment(id, author_user_id);

        let mut repo = MockTaskCommentRepository::new();
        repo.expect_find_by_id().with(eq(id)).returning(move |_| {
            let c = existing.clone();
            Box::pin(async move { Ok(Some(c)) })
        });
        // No `expect_soft_delete`: a non-author's DELETE must be rejected
        // before any write is attempted.

        let mut service = TaskCommentService::new(repo);
        let err = service
            .delete_task_comment(id, other_user_id)
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::Forbidden { .. }));
    }

    #[tokio::test]
    async fn delete_task_comment_returns_not_found_when_missing() {
        let id = TaskCommentId(Uuid::new_v4());
        let mut repo = MockTaskCommentRepository::new();
        repo.expect_find_by_id()
            .with(eq(id))
            .returning(|_| Box::pin(async { Ok(None) }));

        let mut service = TaskCommentService::new(repo);
        let err = service
            .delete_task_comment(id, UserId(Uuid::new_v4()))
            .await
            .unwrap_err();

        assert!(matches!(err, CoreError::NotFound));
    }
}
