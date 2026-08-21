use chrono::{DateTime, Utc};
use common::CoreError;
use mestier_macros::repository;

use crate::{
    CustomerId, OrganizationId, Project, ProjectId,
    domain::project::ports::ProjectRepository,
    infrastructure::{
        postgres::{SharedTx, error::map_sqlx_error},
        project::postgres::model::ProjectRow,
    },
};

#[repository(domain = Project, backend = Postgres)]
pub struct PgProjectRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgProjectRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> ProjectRepository for PgProjectRepository<'tx> {
    async fn insert(&mut self, project: &Project) -> Result<Project, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            ProjectRow,
            r#"
            INSERT INTO projects (id, org_id, name, customer_id, customer_context_id, quote_id, archived_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
            RETURNING id, org_id, name, customer_id, customer_context_id, quote_id, archived_at, created_at, updated_at
            "#,
            project.id.0,
            project.organization_id.0,
            project.name,
            project.customer_id.map(|id| id.0),
            project.customer_context_id.map(|id| id.0),
            project.quote_id.map(|id| id.0),
            project.archived_at,
            project.created_at,
            project.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.into())
    }

    /// No `archived_at IS NULL` filter, deliberately: see the port's own note.
    /// A report over last spring has to resolve the project it names, whether
    /// or not the work has since been retired.
    async fn find_by_id(&mut self, id: ProjectId) -> Result<Option<Project>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            ProjectRow,
            r#"
            SELECT id, org_id, name, customer_id, customer_context_id, quote_id, archived_at, created_at, updated_at
            FROM projects
            WHERE id = $1
            "#,
            id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(Into::into))
    }

    /// Both filters are bound rather than concatenated, so the query stays one
    /// statement `query_as!` can check at compile time. A NULL `customer_id`
    /// parameter means "any customer", which is why the predicate tests the
    /// parameter and not the column.
    async fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
        customer_id: Option<CustomerId>,
        include_archived: bool,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Project>, u64), CoreError> {
        let mut tx = self.tx.lock().await;
        let customer_filter = customer_id.map(|id| id.0);

        let rows = sqlx::query_as!(
            ProjectRow,
            r#"
            SELECT id, org_id, name, customer_id, customer_context_id, quote_id, archived_at, created_at, updated_at
            FROM projects
            WHERE org_id = $1
              AND ($2::uuid IS NULL OR customer_id = $2)
              AND ($3::boolean OR archived_at IS NULL)
            ORDER BY name ASC, created_at ASC
            LIMIT $4 OFFSET $5
            "#,
            organization_id.0,
            customer_filter,
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
            FROM projects
            WHERE org_id = $1
              AND ($2::uuid IS NULL OR customer_id = $2)
              AND ($3::boolean OR archived_at IS NULL)
            "#,
            organization_id.0,
            customer_filter,
            include_archived,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok((rows.into_iter().map(Into::into).collect(), total as u64))
    }

    async fn update(&mut self, project: &Project) -> Result<Project, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            ProjectRow,
            r#"
            UPDATE projects
            SET name = $2, customer_id = $3, customer_context_id = $4, quote_id = $5, updated_at = $6
            WHERE id = $1
            RETURNING id, org_id, name, customer_id, customer_context_id, quote_id, archived_at, created_at, updated_at
            "#,
            project.id.0,
            project.name,
            project.customer_id.map(|id| id.0),
            project.customer_context_id.map(|id| id.0),
            project.quote_id.map(|id| id.0),
            project.updated_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?
        .ok_or(CoreError::NotFound)?;

        Ok(row.into())
    }

    async fn set_archived_at(
        &mut self,
        id: ProjectId,
        archived_at: Option<DateTime<Utc>>,
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        let affected = sqlx::query!(
            r#"UPDATE projects SET archived_at = $2, updated_at = $3 WHERE id = $1"#,
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
}
