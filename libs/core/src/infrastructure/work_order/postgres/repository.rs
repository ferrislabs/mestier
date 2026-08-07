use chrono::{DateTime, Utc};
use common::CoreError;
use mestier_macros::repository;
use sqlx::PgConnection;

use crate::{
    Assignment, OrganizationId, WorkOrder, WorkOrderId,
    domain::work_order::ports::WorkOrderRepository,
    infrastructure::{
        postgres::{SharedTx, error::map_sqlx_error},
        work_order::postgres::model::{AssignmentRow, WorkOrderRow},
    },
};

#[repository(domain = WorkOrder, backend = Postgres)]
pub struct PgWorkOrderRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgWorkOrderRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> WorkOrderRepository for PgWorkOrderRepository<'tx> {
    async fn insert(&mut self, work_order: &WorkOrder) -> Result<WorkOrder, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            WorkOrderRow,
            r#"
            INSERT INTO work_orders (id, org_id, customer_id, customer_context_id, quote_id, starts_at, ends_at, all_day, status, title, note, deleted_at, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, CAST($9 AS text)::work_order_status, $10, $11, $12, $13, $14)
            RETURNING id, org_id, customer_id, customer_context_id, quote_id, starts_at, ends_at, all_day, status::text AS "status!", title, note, deleted_at, created_at, updated_at
            "#,
            work_order.id.0,
            work_order.organization_id.0,
            work_order.customer_id.0,
            work_order.customer_context_id.0,
            work_order.quote_id.map(|id| id.0),
            work_order.starts_at,
            work_order.ends_at,
            work_order.all_day,
            work_order.status.as_str(),
            work_order.title,
            work_order.note,
            work_order.deleted_at,
            work_order.created_at,
            work_order.updated_at,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        replace_assignments(&mut tx, work_order.id, &work_order.assignments).await?;
        row.into_work_order(work_order.assignments.clone())
    }

    async fn find_by_id(&mut self, id: WorkOrderId) -> Result<Option<WorkOrder>, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            WorkOrderRow,
            r#"
            SELECT id, org_id, customer_id, customer_context_id, quote_id, starts_at, ends_at, all_day, status::text AS "status!", title, note, deleted_at, created_at, updated_at
            FROM work_orders
            WHERE id = $1 AND deleted_at IS NULL
            "#,
            id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        match row {
            Some(row) => {
                let assignments = fetch_assignments(&mut tx, id).await?;
                Ok(Some(row.into_work_order(assignments)?))
            }
            None => Ok(None),
        }
    }

    async fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
        limit: u64,
        offset: u64,
    ) -> Result<(Vec<WorkOrder>, u64), CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            WorkOrderRow,
            r#"
            SELECT id, org_id, customer_id, customer_context_id, quote_id, starts_at, ends_at, all_day, status::text AS "status!", title, note, deleted_at, created_at, updated_at
            FROM work_orders
            WHERE org_id = $1 AND deleted_at IS NULL
            ORDER BY starts_at ASC, id ASC
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
            r#"SELECT COUNT(*) AS "count!" FROM work_orders WHERE org_id = $1 AND deleted_at IS NULL"#,
            organization_id.0,
        )
        .fetch_one(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let mut work_orders = Vec::with_capacity(rows.len());
        for row in rows {
            let assignments = fetch_assignments(&mut tx, WorkOrderId(row.id)).await?;
            work_orders.push(row.into_work_order(assignments)?);
        }

        Ok((work_orders, total as u64))
    }

    async fn update(&mut self, work_order: &WorkOrder) -> Result<WorkOrder, CoreError> {
        let mut tx = self.tx.lock().await;
        let row = sqlx::query_as!(
            WorkOrderRow,
            r#"
            UPDATE work_orders
            SET starts_at = $2,
                ends_at = $3,
                all_day = $4,
                status = CAST($5 AS text)::work_order_status,
                title = $6,
                note = $7,
                updated_at = $8
            WHERE id = $1 AND deleted_at IS NULL
            RETURNING id, org_id, customer_id, customer_context_id, quote_id, starts_at, ends_at, all_day, status::text AS "status!", title, note, deleted_at, created_at, updated_at
            "#,
            work_order.id.0,
            work_order.starts_at,
            work_order.ends_at,
            work_order.all_day,
            work_order.status.as_str(),
            work_order.title,
            work_order.note,
            work_order.updated_at,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let row = row.ok_or(CoreError::NotFound)?;

        replace_assignments(&mut tx, work_order.id, &work_order.assignments).await?;
        row.into_work_order(work_order.assignments.clone())
    }

    async fn soft_delete(
        &mut self,
        id: WorkOrderId,
        deleted_at: DateTime<Utc>,
    ) -> Result<(), CoreError> {
        let mut tx = self.tx.lock().await;
        let result = sqlx::query!(
            r#"
            UPDATE work_orders
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

async fn fetch_assignments(
    conn: &mut PgConnection,
    work_order_id: WorkOrderId,
) -> Result<Vec<Assignment>, CoreError> {
    let rows = sqlx::query_as!(
        AssignmentRow,
        r#"
        SELECT id, org_id, work_order_id, employee_id, created_at
        FROM assignments
        WHERE work_order_id = $1
        ORDER BY created_at ASC, id ASC
        "#,
        work_order_id.0,
    )
    .fetch_all(conn)
    .await
    .map_err(map_sqlx_error)?;

    Ok(rows.into_iter().map(Into::into).collect())
}

/// Replaces the complete assignment set for a work order: physical delete of
/// whatever is there, then insert of the new list. Assignments carry no
/// history worth keeping once removed (unlike work orders themselves, which
/// are soft-deleted), and the `PATCH` contract treats `assignees` as the
/// full list rather than a delta, so a diff would add complexity without
/// adding correctness.
async fn replace_assignments(
    conn: &mut PgConnection,
    work_order_id: WorkOrderId,
    assignments: &[Assignment],
) -> Result<(), CoreError> {
    sqlx::query!(
        r#"DELETE FROM assignments WHERE work_order_id = $1"#,
        work_order_id.0,
    )
    .execute(&mut *conn)
    .await
    .map_err(map_sqlx_error)?;

    for assignment in assignments {
        sqlx::query!(
            r#"
            INSERT INTO assignments (id, org_id, work_order_id, employee_id, created_at)
            VALUES ($1, $2, $3, $4, $5)
            "#,
            assignment.id.0,
            assignment.organization_id.0,
            assignment.work_order_id.0,
            assignment.employee_id.0,
            assignment.created_at,
        )
        .execute(&mut *conn)
        .await
        .map_err(map_sqlx_error)?;
    }

    Ok(())
}
