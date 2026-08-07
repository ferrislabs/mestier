use chrono::{DateTime, Utc};
use common::CoreError;
use mestier_macros::repository;

use crate::{
    TaskComment, TaskCommentId, TaskId,
    domain::task_comment::ports::TaskCommentRepository,
    infrastructure::{
        postgres::{SharedTx, error::map_sqlx_error},
        task_comment::postgres::model::TaskCommentRow,
    },
};

#[repository(domain = TaskComment, backend = Postgres)]
pub struct PgTaskCommentRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgTaskCommentRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> TaskCommentRepository for PgTaskCommentRepository<'tx> {
    async fn insert(&mut self, comment: &TaskComment) -> Result<TaskComment, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            TaskCommentRow,
            r#"
            INSERT INTO task_comments (id, org_id, task_id, author_user_id, body, deleted_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
            RETURNING id, org_id, task_id, author_user_id, body, deleted_at, created_at, updated_at
            "#,
            comment.id.0,
            comment.organization_id.0,
            comment.task_id.0,
            comment.author_user_id.0,
            comment.body,
            comment.deleted_at,
            comment.created_at,
            comment.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.into())
    }

    async fn find_by_id(&mut self, id: TaskCommentId) -> Result<Option<TaskComment>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            TaskCommentRow,
            r#"
            SELECT id, org_id, task_id, author_user_id, body, deleted_at, created_at, updated_at
            FROM task_comments
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(Into::into))
    }

    async fn list_by_task(
        &mut self,
        task_id: TaskId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<TaskComment>, u64), CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            TaskCommentRow,
            r#"
            SELECT id, org_id, task_id, author_user_id, body, deleted_at, created_at, updated_at
            FROM task_comments
            WHERE task_id = $1 AND deleted_at IS NULL
            ORDER BY created_at ASC, id ASC
            LIMIT $2 OFFSET $3
            "#,
            task_id.0,
            limit as i64,
            offset as i64,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let total: i64 = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) AS "count!"
            FROM task_comments
            WHERE task_id = $1 AND deleted_at IS NULL
            "#,
            task_id.0,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok((rows.into_iter().map(Into::into).collect(), total as u64))
    }

    async fn update(&mut self, comment: &TaskComment) -> Result<TaskComment, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            TaskCommentRow,
            r#"
            UPDATE task_comments
            SET body = $2, updated_at = $3
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING id, org_id, task_id, author_user_id, body, deleted_at, created_at, updated_at
            "#,
            comment.id.0,
            comment.body,
            comment.updated_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(Into::into).ok_or(CoreError::NotFound)
    }

    async fn soft_delete(&mut self, id: TaskCommentId, deleted_at: DateTime<Utc>) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        let result = sqlx::query!(
            r#"
            UPDATE task_comments
            SET deleted_at = $2, updated_at = $2
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id.0,
            deleted_at,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        if result.rows_affected() == 0 {
            return Err(CoreError::NotFound);
        }

        Ok(())
    }
}
