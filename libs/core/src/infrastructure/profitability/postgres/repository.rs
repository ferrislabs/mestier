use chrono::{DateTime, Utc};
use common::CoreError;
use mestier_macros::repository;

use crate::{
    CustomerId, EmployeeId, EquipmentId, OrganizationId, ProfitabilityFacts, TaskId,
    domain::profitability::{
        AssignedEquipment, ClockedTime, JobHeader, ports::ProfitabilityRepository,
    },
    infrastructure::postgres::{SharedTx, error::map_sqlx_error},
};

#[repository(domain = Profitability, backend = Postgres)]
pub struct PgProfitabilityRepository<'tx> {
    tx: SharedTx<'tx>,
}

impl<'tx> PgProfitabilityRepository<'tx> {
    pub fn new(tx: &SharedTx<'tx>) -> Self {
        Self { tx: tx.clone() }
    }
}

impl<'tx> ProfitabilityRepository for PgProfitabilityRepository<'tx> {
    /// Three queries, whatever the number of jobs.
    ///
    /// The hierarchy is capped at two levels by the domain, so a subtask's root
    /// is `COALESCE(parent_task_id, id)` and no recursive query is needed. Doing
    /// that resolution in SQL is what lets the calculation stay flat: it never
    /// walks a tree, it groups on an id the database already computed.
    async fn load(
        &mut self,
        organization_id: OrganizationId,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<ProfitabilityFacts, CoreError> {
        let mut tx = self.tx.lock().await;

        // A job is in scope when work was clocked on it, or on one of its
        // subtasks, inside the window. A chantier quoted and never worked has
        // no cost to report, and listing every one of them would bury the ones
        // that do.
        let jobs = sqlx::query!(
            r#"
            SELECT r.id, r.title, r.customer_id AS "customer_id!", q.total_cents AS "total_cents?"
            FROM tasks r
            LEFT JOIN quotes q ON q.id = r.quote_id AND q.deleted_at IS NULL
            WHERE r.org_id = $1
              AND r.parent_task_id IS NULL
              AND r.customer_id IS NOT NULL
              AND r.deleted_at IS NULL
              AND EXISTS (
                SELECT 1
                FROM time_entries te
                JOIN tasks t ON t.id = te.task_id AND t.deleted_at IS NULL
                WHERE COALESCE(t.parent_task_id, t.id) = r.id
                  AND te.started_at >= $2
                  AND te.started_at < $3
              )
            ORDER BY r.starts_at DESC NULLS LAST, r.id
            "#,
            organization_id.0,
            from,
            to,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?
        .into_iter()
        .map(|row| JobHeader {
            task_id: TaskId(row.id),
            title: row.title,
            customer_id: CustomerId(row.customer_id),
            quoted_cents: row.total_cents,
        })
        .collect();

        // The employee's rate is read as it stands, and may be null. The
        // calculation refuses to cost a null rather than treating it as free,
        // so a soft-deleted employee is deliberately not filtered out: the work
        // happened and its cost is still the job's.
        let clocked = sqlx::query!(
            r#"
            SELECT
                COALESCE(t.parent_task_id, t.id) AS "task_id!",
                te.employee_id,
                e.hourly_rate_cents,
                e.is_salaried,
                te.started_at,
                te.ended_at,
                te.closed_after_the_fact
            FROM time_entries te
            JOIN tasks t ON t.id = te.task_id AND t.deleted_at IS NULL
            JOIN employees e ON e.id = te.employee_id
            WHERE te.org_id = $1
              AND te.started_at >= $2
              AND te.started_at < $3
            ORDER BY te.started_at
            "#,
            organization_id.0,
            from,
            to,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?
        .into_iter()
        .map(|row| ClockedTime {
            task_id: TaskId(row.task_id),
            employee_id: EmployeeId(row.employee_id),
            hourly_rate_cents: row.hourly_rate_cents,
            is_salaried: row.is_salaried,
            started_at: row.started_at,
            ended_at: row.ended_at,
            closed_after_the_fact: row.closed_after_the_fact,
        })
        .collect();

        // `DISTINCT` on the pair: the same machine attached to a subtask and to
        // its root is one machine on one job, and counting it twice would double
        // its hourly rate.
        let equipment = sqlx::query!(
            r#"
            SELECT DISTINCT
                COALESCE(t.parent_task_id, t.id) AS "task_id!",
                l.equipment_id,
                eq.hourly_rate_cents
            FROM task_equipment_links l
            JOIN tasks t ON t.id = l.task_id AND t.deleted_at IS NULL
            JOIN equipment eq ON eq.id = l.equipment_id
            WHERE t.org_id = $1
            "#,
            organization_id.0,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?
        .into_iter()
        .map(|row| AssignedEquipment {
            task_id: TaskId(row.task_id),
            equipment_id: EquipmentId(row.equipment_id),
            hourly_rate_cents: row.hourly_rate_cents,
        })
        .collect();

        Ok(ProfitabilityFacts {
            jobs,
            clocked,
            equipment,
        })
    }
}
