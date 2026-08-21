use chrono::{DateTime, Utc};
use common::CoreError;
use mestier_macros::repository;

use crate::{
    CustomerId, EmployeeId, EquipmentId, MemberId, OrganizationId, ProfitabilityFacts, ProjectId,
    TaskId,
    domain::profitability::{
        AssignedEquipment, PlannedAssignment, ProjectHeader, TaskExpense,
        ports::ProfitabilityRepository,
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
    /// Four queries, whatever the number of projects.
    ///
    /// Every one of them resolves a task's effective window the same way:
    /// `COALESCE(t.starts_at, pt.starts_at)` over a self-join on the parent. The
    /// hierarchy is capped at two levels by the domain, so one join is enough and
    /// no recursive query is needed. Doing it in SQL is what lets the calculation
    /// stay flat — it never walks a tree.
    ///
    /// A subtask whose parent was soft-deleted resolves to a NULL window and
    /// drops out of every list. It has no dates of its own and nothing left to
    /// inherit them from, so there is no honest duration to charge for it.
    async fn load(
        &mut self,
        organization_id: OrganizationId,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<ProfitabilityFacts, CoreError> {
        let mut tx = self.tx.lock().await;

        // A project is in scope when one of its planned tasks overlaps the
        // window. A project with nothing planned has no cost to report, and
        // listing every one of them would bury the ones that do.
        let projects = sqlx::query!(
            r#"
            SELECT p.id, p.name, p.customer_id, q.total_cents AS "total_cents?"
            FROM projects p
            LEFT JOIN quotes q ON q.id = p.quote_id AND q.deleted_at IS NULL
            WHERE p.org_id = $1
              AND EXISTS (
                SELECT 1
                FROM tasks t
                LEFT JOIN tasks pt ON pt.id = t.parent_task_id AND pt.deleted_at IS NULL
                WHERE t.project_id = p.id
                  AND t.deleted_at IS NULL
                  AND t.status <> 'CANCELLED'::task_status
                  AND COALESCE(t.starts_at, pt.starts_at) < $3
                  AND COALESCE(t.ends_at, pt.ends_at) > $2
              )
            ORDER BY p.name ASC, p.id
            "#,
            organization_id.0,
            from,
            to,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?
        .into_iter()
        .map(|row| ProjectHeader {
            project_id: ProjectId(row.id),
            name: row.name,
            customer_id: row.customer_id.map(CustomerId),
            quoted_cents: row.total_cents,
        })
        .collect();

        // Tasks with no project are loaded too: they cost the person's time even
        // though no subject is charged for it, and the per-member totals are what
        // payroll reads.
        //
        // `e.deleted_at IS NULL` matters more than it looks. Without it a member
        // whose profile was replaced would join twice and have their time counted
        // twice. A member with no live profile reads as "no rate set", which is
        // what it is.
        let assignments = sqlx::query!(
            r#"
            SELECT
                t.project_id,
                t.id AS task_id,
                a.member_id,
                e.id AS "employee_id?",
                e.hourly_rate_cents,
                COALESCE(e.is_salaried, false) AS "is_salaried!",
                e.monthly_cost_cents,
                COALESCE(e.weekly_contract_minutes, 0) AS "weekly_contract_minutes!",
                COALESCE(t.starts_at, pt.starts_at) AS "starts_at!",
                COALESCE(t.ends_at, pt.ends_at) AS "ends_at!",
                t.all_day
            FROM task_assignments a
            JOIN tasks t ON t.id = a.task_id AND t.deleted_at IS NULL
            LEFT JOIN tasks pt ON pt.id = t.parent_task_id AND pt.deleted_at IS NULL
            LEFT JOIN employees e
                ON e.member_id = a.member_id
               AND e.org_id = t.org_id
               AND e.deleted_at IS NULL
            WHERE t.org_id = $1
              AND t.status <> 'CANCELLED'::task_status
              AND COALESCE(t.starts_at, pt.starts_at) < $3
              AND COALESCE(t.ends_at, pt.ends_at) > $2
            ORDER BY COALESCE(t.starts_at, pt.starts_at), t.id, a.member_id
            "#,
            organization_id.0,
            from,
            to,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?
        .into_iter()
        .map(|row| PlannedAssignment {
            project_id: row.project_id.map(ProjectId),
            task_id: TaskId(row.task_id),
            member_id: MemberId(row.member_id),
            employee_id: row.employee_id.map(EmployeeId),
            hourly_rate_cents: row.hourly_rate_cents,
            is_salaried: row.is_salaried,
            monthly_cost_cents: row.monthly_cost_cents,
            weekly_contract_minutes: row.weekly_contract_minutes,
            starts_at: row.starts_at,
            ends_at: row.ends_at,
            all_day: row.all_day,
        })
        .collect();

        // Filtered on the task's *start*, not on overlap: an expense is spent
        // once, so it belongs whole to one period rather than being split across
        // the two a long task straddles.
        let expenses = sqlx::query!(
            r#"
            SELECT
                t.project_id AS "project_id!",
                t.id AS task_id,
                t.expenses_cents,
                COALESCE(t.starts_at, pt.starts_at) AS "starts_at!"
            FROM tasks t
            LEFT JOIN tasks pt ON pt.id = t.parent_task_id AND pt.deleted_at IS NULL
            WHERE t.org_id = $1
              AND t.deleted_at IS NULL
              AND t.project_id IS NOT NULL
              AND t.status <> 'CANCELLED'::task_status
              AND t.expenses_cents > 0
              AND COALESCE(t.starts_at, pt.starts_at) >= $2
              AND COALESCE(t.starts_at, pt.starts_at) < $3
            "#,
            organization_id.0,
            from,
            to,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?
        .into_iter()
        .map(|row| TaskExpense {
            project_id: ProjectId(row.project_id),
            task_id: TaskId(row.task_id),
            expenses_cents: row.expenses_cents,
            starts_at: row.starts_at,
        })
        .collect();

        // `DISTINCT` on the pair: the same machine attached to two tasks of one
        // project is one machine on one project, and counting it twice would
        // double its hourly rate.
        let equipment = sqlx::query!(
            r#"
            SELECT DISTINCT
                t.project_id AS "project_id!",
                l.equipment_id,
                eq.hourly_rate_cents
            FROM task_equipment_links l
            JOIN tasks t ON t.id = l.task_id AND t.deleted_at IS NULL
            LEFT JOIN tasks pt ON pt.id = t.parent_task_id AND pt.deleted_at IS NULL
            JOIN equipment eq ON eq.id = l.equipment_id
            WHERE t.org_id = $1
              AND t.project_id IS NOT NULL
              AND t.status <> 'CANCELLED'::task_status
              AND COALESCE(t.starts_at, pt.starts_at) < $3
              AND COALESCE(t.ends_at, pt.ends_at) > $2
            "#,
            organization_id.0,
            from,
            to,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?
        .into_iter()
        .map(|row| AssignedEquipment {
            project_id: ProjectId(row.project_id),
            equipment_id: EquipmentId(row.equipment_id),
            hourly_rate_cents: row.hourly_rate_cents,
        })
        .collect();

        Ok(ProfitabilityFacts {
            projects,
            assignments,
            expenses,
            equipment,
        })
    }
}
