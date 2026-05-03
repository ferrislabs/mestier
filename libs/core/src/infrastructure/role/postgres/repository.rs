use common::CoreError;
use mestier_macros::repository;

use crate::{
    domain::{
        organization::OrganizationId,
        role::{Role, ports::RoleRepository},
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
            INSERT INTO roles (id, organization_id, name, permissions, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6)
            RETURNING id, organization_id, name, permissions, created_at, updated_at
            "#,
            role.id.0,
            role.organization_id.0,
            role.name,
            role.permissions.bits(),
            role.created_at,
            role.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.into())
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
            SELECT id, organization_id, name, permissions, created_at, updated_at
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
}
