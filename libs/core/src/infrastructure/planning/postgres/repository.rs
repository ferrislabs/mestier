use std::collections::HashMap;

use chrono::{DateTime, NaiveDate, Utc};
use common::CoreError;
use mestier_macros::repository;
use uuid::Uuid;

use crate::{
    Assignment, EmployeeAbsence, EmployeeRhythm, EmployeeWorkSlot, OrganizationId,
    PlanningWorkOrder, RhythmSlot,
    domain::planning::ports::PlanningRepository,
    infrastructure::{
        absence::postgres::model::AbsenceRow,
        planning::postgres::model::PlanningWorkOrderRow,
        postgres::{SharedTx, error::map_sqlx_error},
        work_order::postgres::model::AssignmentRow,
        work_time::postgres::model::{RhythmRow, RhythmSlotRow, WorkSlotRow},
    },
};

/// The planning read model's repository: batched, organization-wide reads
/// across `work_orders`/`assignments` (enriched with the customer join),
/// `employee_absences`, `employee_rhythms`/`employee_rhythm_slots` and
/// `employee_work_slots`. Every method loads the whole organization in a
/// small, fixed number of queries — never one per employee (see the
/// planning module design doc's N+1 warning).
#[repository(domain = Planning, backend = Postgres)]
pub struct PgPlanningRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgPlanningRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> PlanningRepository for PgPlanningRepository<'tx> {
    #[tracing::instrument(skip(self), fields(db.system = "postgresql", db.operation = "select", db.table = "work_orders"), err)]
    async fn list_work_orders_in_window(
        &mut self,
        organization_id: OrganizationId,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<PlanningWorkOrder>, CoreError> {
        let mut tx = self.tx.lock().await;

        let work_order_rows = sqlx::query_as!(
            PlanningWorkOrderRow,
            r#"
            SELECT
                wo.id, wo.org_id, wo.customer_id, wo.customer_context_id, wo.quote_id,
                wo.starts_at, wo.ends_at, wo.all_day, wo.status::text AS "status!", wo.title, wo.note,
                wo.deleted_at, wo.created_at, wo.updated_at,
                (c.first_name || ' ' || c.last_name) AS "customer_name!",
                cc.label AS "context_label!"
            FROM work_orders wo
            JOIN customers c ON c.id = wo.customer_id
            JOIN customer_contexts cc ON cc.id = wo.customer_context_id
            WHERE wo.org_id = $1 AND wo.deleted_at IS NULL
              AND wo.starts_at < $3 AND wo.ends_at > $2
            ORDER BY wo.starts_at ASC, wo.id ASC
            "#,
            organization_id.0,
            from,
            to,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let assignment_rows = sqlx::query_as!(
            AssignmentRow,
            r#"
            SELECT a.id, a.org_id, a.work_order_id, a.employee_id, a.created_at
            FROM assignments a
            JOIN work_orders wo ON wo.id = a.work_order_id
            WHERE wo.org_id = $1 AND wo.deleted_at IS NULL
              AND wo.starts_at < $3 AND wo.ends_at > $2
            ORDER BY a.created_at ASC, a.id ASC
            "#,
            organization_id.0,
            from,
            to,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let mut assignments_by_work_order: HashMap<Uuid, Vec<Assignment>> = HashMap::new();
        for row in assignment_rows {
            let work_order_id = row.work_order_id;
            assignments_by_work_order
                .entry(work_order_id)
                .or_default()
                .push(row.into());
        }

        let mut work_orders = Vec::with_capacity(work_order_rows.len());
        for row in work_order_rows {
            let id = row.id;
            let assignments = assignments_by_work_order.remove(&id).unwrap_or_default();
            work_orders.push(row.into_planning_work_order(assignments)?);
        }

        Ok(work_orders)
    }

    #[tracing::instrument(skip(self), fields(db.system = "postgresql", db.operation = "select", db.table = "employee_absences"), err)]
    async fn list_absences_in_window(
        &mut self,
        organization_id: OrganizationId,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<EmployeeAbsence>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            AbsenceRow,
            r#"
            SELECT id, org_id, employee_id, kind::text AS "kind!", starts_at, ends_at, all_day, note, deleted_at, created_at, updated_at
            FROM employee_absences
            WHERE org_id = $1 AND deleted_at IS NULL
              AND starts_at < $3 AND ends_at > $2
            ORDER BY starts_at ASC, id ASC
            "#,
            organization_id.0,
            from,
            to,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let mut absences = Vec::with_capacity(rows.len());
        for row in rows {
            absences.push(row.into_employee_absence()?);
        }

        Ok(absences)
    }

    #[tracing::instrument(skip(self), fields(db.system = "postgresql", db.operation = "select", db.table = "employee_rhythms"), err)]
    async fn list_rhythms_for_organization(
        &mut self,
        organization_id: OrganizationId,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<EmployeeRhythm>, CoreError> {
        let mut tx = self.tx.lock().await;

        let rhythm_rows = sqlx::query_as!(
            RhythmRow,
            r#"
            SELECT id, org_id, employee_id, effective_from, effective_to, created_at, updated_at
            FROM employee_rhythms
            WHERE org_id = $1
              AND effective_from <= $3
              AND (effective_to IS NULL OR effective_to > $2)
            ORDER BY employee_id ASC, effective_from ASC
            "#,
            organization_id.0,
            from,
            to,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let slot_rows = sqlx::query_as!(
            RhythmSlotRow,
            r#"
            SELECT rs.id, rs.rhythm_id, rs.weekday, rs.starts_minute, rs.ends_minute
            FROM employee_rhythm_slots rs
            JOIN employee_rhythms r ON r.id = rs.rhythm_id
            WHERE r.org_id = $1
              AND r.effective_from <= $3
              AND (r.effective_to IS NULL OR r.effective_to > $2)
            ORDER BY rs.weekday ASC, rs.starts_minute ASC
            "#,
            organization_id.0,
            from,
            to,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let mut slots_by_rhythm: HashMap<Uuid, Vec<RhythmSlot>> = HashMap::new();
        for row in slot_rows {
            let rhythm_id = row.rhythm_id;
            slots_by_rhythm
                .entry(rhythm_id)
                .or_default()
                .push(row.into());
        }

        let rhythms = rhythm_rows
            .into_iter()
            .map(|row| {
                let id = row.id;
                let slots = slots_by_rhythm.remove(&id).unwrap_or_default();
                row.into_employee_rhythm(slots)
            })
            .collect();

        Ok(rhythms)
    }

    #[tracing::instrument(skip(self), fields(db.system = "postgresql", db.operation = "select", db.table = "employee_work_slots"), err)]
    async fn list_work_slots_for_organization(
        &mut self,
        organization_id: OrganizationId,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<Vec<EmployeeWorkSlot>, CoreError> {
        let mut tx = self.tx.lock().await;
        let rows = sqlx::query_as!(
            WorkSlotRow,
            r#"
            SELECT id, org_id, employee_id, work_date, starts_minute, ends_minute
            FROM employee_work_slots
            WHERE org_id = $1 AND work_date BETWEEN $2 AND $3
            ORDER BY employee_id ASC, work_date ASC, starts_minute ASC
            "#,
            organization_id.0,
            from,
            to,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        Ok(rows.into_iter().map(Into::into).collect())
    }
}
