use std::collections::HashMap;

use chrono::{DateTime, NaiveDate, Utc};
use common::CoreError;
use mestier_macros::repository;
use uuid::Uuid;

use crate::{
    EmployeeAbsence, EmployeeRhythm, EmployeeWorkSlot, OrganizationId, PlanningTask, RhythmSlot,
    TaskAssignment,
    domain::planning::ports::PlanningRepository,
    infrastructure::{
        absence::postgres::model::AbsenceRow,
        planning::postgres::model::PlanningTaskRow,
        postgres::{SharedTx, error::map_sqlx_error},
        task::postgres::model::TaskAssignmentRow,
        work_time::postgres::model::{RhythmRow, RhythmSlotRow, WorkSlotRow},
    },
};

/// The planning read model's repository: batched, organization-wide reads
/// across `tasks`/`task_assignments` (enriched with the customer join and
/// each task's child count), `employee_absences`,
/// `employee_rhythms`/`employee_rhythm_slots` and `employee_work_slots`.
/// Every method loads the whole organization in a small, fixed number of
/// queries — never one per employee (see the planning module design doc's
/// N+1 warning).
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
    #[tracing::instrument(skip(self), fields(db.system = "postgresql", db.operation = "select", db.table = "tasks"), err)]
    async fn list_tasks_in_window(
        &mut self,
        organization_id: OrganizationId,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<Vec<PlanningTask>, CoreError> {
        let mut tx = self.tx.lock().await;

        // `customer_id`/`customer_context_id` are nullable now, so the
        // customer join is a `LEFT JOIN` (a task with no customer must
        // still appear) and `customer_name`/`context_label` resolve to
        // `NULL` rather than dropping the row. `starts_at`/`ends_at` are
        // nullable too — the window predicate below naturally excludes a
        // task with neither (a subtask inheriting its parent's window);
        // see `PlanningRepository::list_tasks_in_window`'s doc comment.
        let task_rows = sqlx::query_as!(
            PlanningTaskRow,
            r#"
            SELECT
                t.id, t.org_id, t.parent_task_id, t.title, t.description,
                t.starts_at, t.ends_at, t.all_day, t.status::text AS "status!", t.blocks_availability,
                t.customer_id, t.customer_context_id, t.quote_id,
                t.deleted_at, t.created_at, t.updated_at,
                (c.first_name || ' ' || c.last_name) AS customer_name,
                cc.label AS context_label,
                COALESCE(child_counts.count, 0) AS "child_count!"
            FROM tasks t
            LEFT JOIN customers c ON c.id = t.customer_id
            LEFT JOIN customer_contexts cc ON cc.id = t.customer_context_id
            LEFT JOIN (
                SELECT parent_task_id, COUNT(*) AS count
                FROM tasks
                WHERE org_id = $1 AND deleted_at IS NULL AND parent_task_id IS NOT NULL
                GROUP BY parent_task_id
            ) child_counts ON child_counts.parent_task_id = t.id
            WHERE t.org_id = $1 AND t.deleted_at IS NULL
              -- NULL < / > NULL is NULL, never TRUE, so a task with no
              -- dates of its own (a subtask inheriting its parent's window)
              -- is excluded here automatically, not by an explicit
              -- IS NOT NULL guard. Deliberate for T1: rendering an
              -- inherited window on the grid needs `resolve_task_window`
              -- plus a parent fetch, which no front code calls yet.
              AND t.starts_at < $3 AND t.ends_at > $2
            ORDER BY t.starts_at ASC, t.id ASC
            "#,
            organization_id.0,
            from,
            to,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let assignment_rows = sqlx::query_as!(
            TaskAssignmentRow,
            r#"
            SELECT a.id, a.org_id, a.task_id, a.employee_id, a.created_at
            FROM task_assignments a
            JOIN tasks t ON t.id = a.task_id
            WHERE t.org_id = $1 AND t.deleted_at IS NULL
              -- Same window predicate as above, kept in sync: an assignment
              -- must only be attached to a task that itself made the cut.
              AND t.starts_at < $3 AND t.ends_at > $2
            ORDER BY a.created_at ASC, a.id ASC
            "#,
            organization_id.0,
            from,
            to,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?;

        let mut assignments_by_task: HashMap<Uuid, Vec<TaskAssignment>> = HashMap::new();
        for row in assignment_rows {
            let task_id = row.task_id;
            assignments_by_task
                .entry(task_id)
                .or_default()
                .push(row.into());
        }

        let mut tasks = Vec::with_capacity(task_rows.len());
        for row in task_rows {
            let id = row.id;
            let assignments = assignments_by_task.remove(&id).unwrap_or_default();
            tasks.push(row.into_planning_task(assignments)?);
        }

        Ok(tasks)
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
