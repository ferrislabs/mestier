use std::collections::HashMap;

use chrono::{DateTime, Utc};
use common::{CoreError, generate_uuid_v7};
use mestier_macros::repository;
use uuid::Uuid;

use crate::{
    Equipment, EquipmentId, OrganizationId, TaskId,
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

    async fn replace_task_links(
        &mut self,
        task_id: TaskId,
        equipment_ids: &[EquipmentId],
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;

        sqlx::query!(
            r#"DELETE FROM task_equipment_links WHERE task_id = $1"#,
            task_id.0,
        )
        .execute(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        for equipment_id in equipment_ids {
            sqlx::query!(
                r#"
                INSERT INTO task_equipment_links (id, task_id, equipment_id, created_at)
                VALUES ($1, $2, $3, now())
                "#,
                generate_uuid_v7(),
                task_id.0,
                equipment_id.0,
            )
            .execute(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?;
        }

        Ok(())
    }

    async fn list_equipment_for_tasks(
        &mut self,
        task_ids: &[TaskId],
    ) -> Result<HashMap<TaskId, Vec<Equipment>>, CoreError> {
        if task_ids.is_empty() {
            return Ok(HashMap::new());
        }

        let mut tx = self.tx.lock().await;
        let ids: Vec<Uuid> = task_ids.iter().map(|id| id.0).collect();
        let rows = sqlx::query!(
            r#"
            SELECT
                lnk.task_id AS "task_id!",
                e.id AS "equipment_id!",
                e.org_id AS "org_id!",
                e.name AS "name!",
                e.hourly_rate_cents AS "hourly_rate_cents!",
                e.deleted_at,
                e.created_at AS "created_at!",
                e.updated_at AS "updated_at!"
            FROM task_equipment_links lnk
            JOIN equipment e ON e.id = lnk.equipment_id
            WHERE lnk.task_id = ANY($1)
            ORDER BY e.name ASC
            "#,
            &ids,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let mut equipment_by_task: HashMap<TaskId, Vec<Equipment>> = HashMap::new();
        for row in rows {
            equipment_by_task
                .entry(TaskId(row.task_id))
                .or_default()
                .push(Equipment {
                    id: EquipmentId(row.equipment_id),
                    organization_id: OrganizationId(row.org_id),
                    name: row.name,
                    hourly_rate_cents: row.hourly_rate_cents,
                    deleted_at: row.deleted_at,
                    created_at: row.created_at,
                    updated_at: row.updated_at,
                });
        }

        Ok(equipment_by_task)
    }
}
