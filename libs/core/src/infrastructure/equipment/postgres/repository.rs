use chrono::{DateTime, Utc};
use common::CoreError;
use mestier_macros::repository;

use crate::{
    Equipment, EquipmentId, OrganizationId,
    domain::equipment::ports::EquipmentRepository,
    infrastructure::{
        equipment::postgres::model::EquipmentRow,
        postgres::{SharedTx, error::map_sqlx_error},
    },
};

#[repository(domain = Equipment, backend = Postgres)]
pub struct PgEquipmentRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgEquipmentRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> EquipmentRepository for PgEquipmentRepository<'tx> {
    async fn insert(&mut self, equipment: &Equipment) -> Result<Equipment, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            EquipmentRow,
            r#"
            INSERT INTO equipment (id, org_id, name, hourly_rate_cents, deleted_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            RETURNING id, org_id, name, hourly_rate_cents, deleted_at, created_at, updated_at
            "#,
            equipment.id.0,
            equipment.organization_id.0,
            equipment.name,
            equipment.hourly_rate_cents,
            equipment.deleted_at,
            equipment.created_at,
            equipment.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(row.into())
    }

    async fn find_by_id(&mut self, id: EquipmentId) -> Result<Option<Equipment>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            EquipmentRow,
            r#"
            SELECT id, org_id, name, hourly_rate_cents, deleted_at, created_at, updated_at
            FROM equipment
            WHERE id = $1 AND deleted_at IS NULL
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
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<Equipment>, u64), CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            EquipmentRow,
            r#"
            SELECT id, org_id, name, hourly_rate_cents, deleted_at, created_at, updated_at
            FROM equipment
            WHERE org_id = $1 AND deleted_at IS NULL
            ORDER BY name ASC, created_at ASC
            LIMIT $2 OFFSET $3
            "#,
            organization_id.0,
            limit as i64,
            offset as i64,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let total: i64 = sqlx::query_scalar!(
            r#"SELECT COUNT(*) AS "count!" FROM equipment WHERE org_id = $1 AND deleted_at IS NULL"#,
            organization_id.0,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok((rows.into_iter().map(Into::into).collect(), total as u64))
    }

    async fn update(&mut self, equipment: &Equipment) -> Result<Equipment, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            EquipmentRow,
            r#"
            UPDATE equipment
            SET name = $2, hourly_rate_cents = $3, updated_at = $4
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING id, org_id, name, hourly_rate_cents, deleted_at, created_at, updated_at
            "#,
            equipment.id.0,
            equipment.name,
            equipment.hourly_rate_cents,
            equipment.updated_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        row.map(Into::into).ok_or(CoreError::NotFound)
    }

    async fn soft_delete(
        &mut self,
        id: EquipmentId,
        deleted_at: DateTime<Utc>,
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        let result = sqlx::query!(
            r#"
            UPDATE equipment
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
