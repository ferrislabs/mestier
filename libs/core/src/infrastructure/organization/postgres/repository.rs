use chrono::{DateTime, Utc};
use common::CoreError;
use mestier_macros::repository;
use tracing::info;

use crate::{
    UserId,
    domain::organization::{Organization, OrganizationId, ports::OrganizationRepository},
    infrastructure::{
        organization::postgres::model::OrganizationRow,
        postgres::{SharedTx, error::map_sqlx_error},
    },
};

#[repository(domain = Organization, backend = Postgres)]
pub struct PgOrganizationRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgOrganizationRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> OrganizationRepository for PgOrganizationRepository<'tx> {
    #[tracing::instrument(skip(self, organization), fields(db.system = "postgresql", db.operation = "insert", db.table = "organizations", organization.slug = %organization.slug), err)]
    async fn insert(&mut self, organization: &Organization) -> Result<Organization, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            OrganizationRow,
            r#"
            INSERT INTO organizations (id, name, slug, owner_id, deleted_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, name, slug, owner_id, deleted_at, created_at, updated_at
            "#,
            organization.id.0,
            organization.name,
            organization.slug,
            organization.owner_id.0,
            organization.deleted_at,
            organization.created_at,
            organization.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.into())
    }

    #[tracing::instrument(skip(self), fields(db.system = "postgresql", db.operation = "select", db.table = "organizations"), err)]
    async fn find_by_id(&mut self, id: OrganizationId) -> Result<Option<Organization>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            OrganizationRow,
            r#"
            SELECT id, name, slug, owner_id, deleted_at, created_at, updated_at
            FROM organizations
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(Into::into))
    }

    #[tracing::instrument(skip(self), fields(db.system = "postgresql", db.operation = "select", db.table = "organizations"), err)]
    async fn find_timezone(&mut self, id: OrganizationId) -> Result<Option<String>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query!(
            r#"SELECT timezone FROM organizations WHERE id = $1 AND deleted_at IS NULL"#,
            id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(|row| row.timezone))
    }

    #[tracing::instrument(skip(self), fields(db.system = "postgresql", db.operation = "select", db.table = "organizations"), err)]
    async fn list_for_user(&mut self, user_id: UserId) -> Result<Vec<Organization>, CoreError> {
        let mut tx = self.tx.lock().await;
        info!("user_id: {:?}", user_id);
        let rows = sqlx::query_as!(
            OrganizationRow,
            r#"
            SELECT o.id, o.name, o.slug, o.owner_id, o.deleted_at, o.created_at, o.updated_at
            FROM organizations o
            INNER JOIN organization_members m ON m.organization_id = o.id
            WHERE m.user_id = $1 AND o.deleted_at IS NULL
            ORDER BY o.created_at ASC
            "#,
            user_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    #[tracing::instrument(skip(self), fields(db.system = "postgresql", db.operation = "select", db.table = "organizations"), err)]
    async fn list_paginated(
        &mut self,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Organization>, u64), CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            OrganizationRow,
            r#"
            SELECT id, name, slug, owner_id, deleted_at, created_at, updated_at
            FROM organizations
            WHERE deleted_at IS NULL
            ORDER BY created_at ASC
            LIMIT $1 OFFSET $2
            "#,
            limit as i64,
            offset as i64,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let total: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM organizations WHERE deleted_at IS NULL"#
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok((rows.into_iter().map(Into::into).collect(), total as u64))
    }

    #[tracing::instrument(skip(self, organization), fields(db.system = "postgresql", db.operation = "update", db.table = "organizations"), err)]
    async fn update(&mut self, organization: &Organization) -> Result<Organization, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            OrganizationRow,
            r#"
            UPDATE organizations
            SET name = $2, slug = $3, updated_at = $4
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING id, name, slug, owner_id, deleted_at, created_at, updated_at
            "#,
            organization.id.0,
            organization.name,
            organization.slug,
            organization.updated_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(Into::into).ok_or(CoreError::NotFound)
    }

    #[tracing::instrument(skip(self), fields(db.system = "postgresql", db.operation = "update", db.table = "organizations"), err)]
    async fn soft_delete(
        &mut self,
        id: OrganizationId,
        deleted_at: DateTime<Utc>,
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        let result = sqlx::query!(
            r#"
            UPDATE organizations
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
