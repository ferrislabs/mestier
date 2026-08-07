use std::collections::HashMap;

use common::{CoreError, generate_uuid_v7};
use mestier_macros::repository;
use uuid::Uuid;

use crate::{
    OrganizationId, TaskId, TaskLabel, TaskLabelId,
    domain::task_label::ports::TaskLabelRepository,
    infrastructure::{
        postgres::{SharedTx, error::map_sqlx_error},
        task_label::postgres::model::TaskLabelRow,
    },
};

#[repository(domain = TaskLabel, backend = Postgres)]
pub struct PgTaskLabelRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgTaskLabelRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> TaskLabelRepository for PgTaskLabelRepository<'tx> {
    async fn insert(&mut self, label: &TaskLabel) -> Result<TaskLabel, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            TaskLabelRow,
            r#"
            INSERT INTO task_labels (id, org_id, name, color, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, org_id, name, color, created_at, updated_at
            "#,
            label.id.0,
            label.organization_id.0,
            label.name,
            label.color,
            label.created_at,
            label.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.into())
    }

    async fn find_by_id(&mut self, id: TaskLabelId) -> Result<Option<TaskLabel>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            TaskLabelRow,
            r#"
            SELECT id, org_id, name, color, created_at, updated_at
            FROM task_labels
            WHERE id = $1
            "#,
            id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(Into::into))
    }

    async fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
    ) -> Result<Vec<TaskLabel>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            TaskLabelRow,
            r#"
            SELECT id, org_id, name, color, created_at, updated_at
            FROM task_labels
            WHERE org_id = $1
            ORDER BY name ASC
            "#,
            organization_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    async fn update(&mut self, label: &TaskLabel) -> Result<TaskLabel, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            TaskLabelRow,
            r#"
            UPDATE task_labels
            SET name = $2, color = $3, updated_at = $4
            WHERE id = $1
            RETURNING id, org_id, name, color, created_at, updated_at
            "#,
            label.id.0,
            label.name,
            label.color,
            label.updated_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(Into::into).ok_or(CoreError::NotFound)
    }

    async fn delete(&mut self, id: TaskLabelId) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        let result = sqlx::query!(r#"DELETE FROM task_labels WHERE id = $1"#, id.0)
            .execute(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?;

        if result.rows_affected() == 0 {
            return Err(CoreError::NotFound);
        }

        Ok(())
    }

    async fn replace_links(
        &mut self,
        task_id: TaskId,
        label_ids: &[TaskLabelId],
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;

        sqlx::query!(
            r#"DELETE FROM task_label_links WHERE task_id = $1"#,
            task_id.0,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        for label_id in label_ids {
            sqlx::query!(
                r#"
                INSERT INTO task_label_links (id, task_id, label_id, created_at)
                VALUES ($1, $2, $3, now())
                "#,
                generate_uuid_v7(),
                task_id.0,
                label_id.0,
            )
            .execute(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?;
        }

        Ok(())
    }

    async fn list_labels_for_tasks(
        &mut self,
        task_ids: &[TaskId],
    ) -> Result<HashMap<TaskId, Vec<TaskLabel>>, CoreError> {
        if task_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let mut tx = self.tx.lock().await;
        let ids: Vec<Uuid> = task_ids.iter().map(|id| id.0).collect();
        let rows = sqlx::query!(
            r#"
            SELECT
                lnk.task_id AS "task_id!",
                tl.id AS "label_id!",
                tl.org_id AS "org_id!",
                tl.name AS "name!",
                tl.color AS "color!",
                tl.created_at AS "created_at!",
                tl.updated_at AS "updated_at!"
            FROM task_label_links lnk
            JOIN task_labels tl ON tl.id = lnk.label_id
            WHERE lnk.task_id = ANY($1)
            ORDER BY tl.name ASC
            "#,
            &ids,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let mut labels_by_task: HashMap<TaskId, Vec<TaskLabel>> = HashMap::new();
        for row in rows {
            labels_by_task
                .entry(TaskId(row.task_id))
                .or_default()
                .push(TaskLabel {
                    id: TaskLabelId(row.label_id),
                    organization_id: OrganizationId(row.org_id),
                    name: row.name,
                    color: row.color,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                });
        }

        Ok(labels_by_task)
    }
}
