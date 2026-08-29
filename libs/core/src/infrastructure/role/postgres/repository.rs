use common::CoreError;
use mestier_macros::repository;

use crate::{
    domain::{
        organization::OrganizationId,
        role::{Role, RoleId, ports::RoleRepository},
    },
    infrastructure::{
        postgres::{SharedTx, error::map_sqlx_error},
        role::postgres::model::RoleRow,
    },
};

#[repository(domain = Role, backend = Postgres)]
pub struct PgRoleRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgRoleRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> RoleRepository for PgRoleRepository<'tx> {
    #[tracing::instrument(skip(self, role), fields(db.system = "postgresql", db.operation = "insert", db.table = "roles", role.name = %role.name), err)]
    async fn insert(&mut self, role: &Role) -> Result<Role, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            RoleRow,
            r#"
            INSERT INTO roles (id, organization_id, name, permissions, is_seeded, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, organization_id, name, permissions, is_seeded, created_at, updated_at
            "#,
            role.id.0,
            role.organization_id.0,
            role.name,
            role.permissions.bits(),
            role.is_seeded,
            role.created_at,
            role.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.into())
    }

    #[tracing::instrument(skip(self), fields(db.system = "postgresql", db.operation = "select", db.table = "roles"), err)]
    async fn find_by_id(&mut self, id: RoleId) -> Result<Option<Role>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            RoleRow,
            r#"
            SELECT id, organization_id, name, permissions, is_seeded, created_at, updated_at
            FROM roles
            WHERE id = $1
            "#,
            id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.map(Into::into))
    }

    #[tracing::instrument(skip(self), fields(db.system = "postgresql", db.operation = "select", db.table = "roles"), err)]
    async fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
    ) -> Result<Vec<Role>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            RoleRow,
            r#"
            SELECT id, organization_id, name, permissions, is_seeded, created_at, updated_at
            FROM roles
            WHERE organization_id = $1
            ORDER BY created_at ASC
            "#,
            organization_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows.into_iter().map(Into::into).collect())
    }

    #[tracing::instrument(skip(self, role), fields(db.system = "postgresql", db.operation = "update", db.table = "roles", role.id = %role.id.0), err)]
    async fn update(&mut self, role: &Role) -> Result<Role, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            RoleRow,
            r#"
            UPDATE roles
            SET name = $2, permissions = $3, updated_at = $4
            WHERE id = $1
            RETURNING id, organization_id, name, permissions, is_seeded, created_at, updated_at
            "#,
            role.id.0,
            role.name,
            role.permissions.bits(),
            role.updated_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(Into::into).ok_or(CoreError::NotFound)
    }

    #[tracing::instrument(skip(self), fields(db.system = "postgresql", db.operation = "delete", db.table = "roles", role.id = %id.0), err)]
    async fn delete(&mut self, id: RoleId) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        sqlx::query!("DELETE FROM roles WHERE id = $1", id.0)
            .execute(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?;

        Ok(())
    }

    #[tracing::instrument(skip(self), fields(db.system = "postgresql", db.operation = "select", db.table = "member_roles", role.id = %id.0), err)]
    async fn count_assigned_members(&mut self, id: RoleId) -> Result<i64, CoreError> {
        let mut tx = self.tx.lock().await;
        let count =
            sqlx::query_scalar!("SELECT COUNT(*) FROM member_roles WHERE role_id = $1", id.0,)
                .fetch_one(&mut ***tx)
                .await
                .map_err(map_sqlx_error)?;

        Ok(count.unwrap_or(0))
    }
}
