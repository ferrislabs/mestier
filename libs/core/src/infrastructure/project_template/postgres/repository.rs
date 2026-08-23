use chrono::{DateTime, Utc};
use common::CoreError;
use mestier_macros::repository;

use crate::{
    OrganizationId, ProjectTemplate, ProjectTemplateId, ProjectTemplateTask,
    domain::project_template::ports::ProjectTemplateRepository,
    infrastructure::{
        postgres::{SharedTx, error::map_sqlx_error},
        project_template::postgres::model::{ProjectTemplateRow, ProjectTemplateTaskRow},
    },
};

#[repository(domain = ProjectTemplate, backend = Postgres)]
pub struct PgProjectTemplateRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgProjectTemplateRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> ProjectTemplateRepository for PgProjectTemplateRepository<'tx> {
    async fn insert(&mut self, template: &ProjectTemplate) -> Result<ProjectTemplate, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            ProjectTemplateRow,
            r#"
            INSERT INTO project_templates (id, org_id, name, description, archived_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, org_id, name, description, archived_at, created_at, updated_at
            "#,
            template.id.0,
            template.organization_id.0,
            template.name,
            template.description,
            template.archived_at,
            template.created_at,
            template.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.into())
    }

    async fn find_by_id(
        &mut self,
        id: ProjectTemplateId,
    ) -> Result<Option<ProjectTemplate>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            ProjectTemplateRow,
            r#"
            SELECT id, org_id, name, description, archived_at, created_at, updated_at
            FROM project_templates
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
        include_archived: bool,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<ProjectTemplate>, u64), CoreError> {
        let mut tx = self.tx.lock().await;

        let rows = sqlx::query_as!(
            ProjectTemplateRow,
            r#"
            SELECT id, org_id, name, description, archived_at, created_at, updated_at
            FROM project_templates
            WHERE org_id = $1
              AND ($2::boolean OR archived_at IS NULL)
            ORDER BY name ASC, created_at ASC
            LIMIT $3 OFFSET $4
            "#,
            organization_id.0,
            include_archived,
            limit as i64,
            offset as i64,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let total: i64 = sqlx::query_scalar!(
            r#"
            SELECT COUNT(*) AS "count!"
            FROM project_templates
            WHERE org_id = $1
              AND ($2::boolean OR archived_at IS NULL)
            "#,
            organization_id.0,
            include_archived,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok((rows.into_iter().map(Into::into).collect(), total as u64))
    }

    async fn update(&mut self, template: &ProjectTemplate) -> Result<ProjectTemplate, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            ProjectTemplateRow,
            r#"
            UPDATE project_templates
            SET name = $2, description = $3, updated_at = $4
            WHERE id = $1
            RETURNING id, org_id, name, description, archived_at, created_at, updated_at
            "#,
            template.id.0,
            template.name,
            template.description,
            template.updated_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?
        .ok_or(CoreError::NotFound)?;

        Ok(row.into())
    }

    async fn set_archived_at(
        &mut self,
        id: ProjectTemplateId,
        archived_at: Option<DateTime<Utc>>,
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        let affected = sqlx::query!(
            r#"UPDATE project_templates SET archived_at = $2, updated_at = $3 WHERE id = $1"#,
            id.0,
            archived_at,
            Utc::now(),
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?
        .rows_affected();

        if affected == 0 {
            return Err(CoreError::NotFound);
        }

        Ok(())
    }

    async fn replace_tasks(
        &mut self,
        template_id: ProjectTemplateId,
        organization_id: OrganizationId,
        tasks: &[ProjectTemplateTask],
    ) -> Result<Vec<ProjectTemplateTask>, CoreError> {
        let mut tx = self.tx.lock().await;

        sqlx::query!(
            r#"DELETE FROM project_template_tasks WHERE template_id = $1"#,
            template_id.0,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let mut inserted = Vec::with_capacity(tasks.len());
        for task in tasks {
            let row = sqlx::query_as!(
                ProjectTemplateTaskRow,
                r#"
                INSERT INTO project_template_tasks (
                    id, org_id, template_id, title, description, day_offset,
                    starts_minute, ends_minute, all_day, blocks_availability,
                    expenses_cents, expenses_label, parent_index, position
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
                RETURNING id, org_id, template_id, title, description, day_offset,
                    starts_minute, ends_minute, all_day, blocks_availability,
                    expenses_cents, expenses_label, parent_index, position
                "#,
                task.id.0,
                organization_id.0,
                template_id.0,
                task.title,
                task.description,
                task.day_offset,
                task.starts_minute,
                task.ends_minute,
                task.all_day,
                task.blocks_availability,
                task.expenses_cents,
                task.expenses_label,
                task.parent_index,
                task.position,
            )
            .fetch_one(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?;

            inserted.push(row.into());
        }

        Ok(inserted)
    }

    async fn list_tasks(
        &mut self,
        template_id: ProjectTemplateId,
    ) -> Result<Vec<ProjectTemplateTask>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            ProjectTemplateTaskRow,
            r#"
            SELECT id, org_id, template_id, title, description, day_offset,
                starts_minute, ends_minute, all_day, blocks_availability,
                expenses_cents, expenses_label, parent_index, position
            FROM project_template_tasks
            WHERE template_id = $1
            ORDER BY position ASC
            "#,
            template_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows.into_iter().map(Into::into).collect())
    }
}
