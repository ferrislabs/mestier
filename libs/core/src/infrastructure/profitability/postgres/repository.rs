use std::collections::HashSet;

use chrono::{DateTime, Utc};
use common::CoreError;
use mestier_macros::repository;

use crate::{
    CustomerId, EmployeeCostBasis, EmployeeCostBasisId, EmployeeId, EquipmentId, MemberId,
    OrganizationId, ProfitabilityFacts, ProjectId, TaskId,
    domain::profitability::{
        AssignedEquipment, PlannedAssignment, ProjectHeader, SupplierCostAllocation, TaskExpense,
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
            SELECT p.id, p.name, p.customer_id, q.net_cents AS "net_cents?"
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
            quoted_cents: row.net_cents,
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
        //
        // The cost fields come from `employee_cost_bases`, not from `employees`'
        // live columns: the version joined is the one covering the task's own
        // start date, cast to a UTC calendar date the same way a cost basis
        // version's own `effective_from`/`effective_to` are set (see
        // `SetEmployeeCostBasisCommand`). A raise entered today must not move
        // what a task planned before it already cost. `e.id` still anchors the
        // join — a member with no live profile has no version to find either.
        let assignments: Vec<PlannedAssignment> = sqlx::query!(
            r#"
            SELECT
                t.project_id,
                t.id AS task_id,
                a.member_id,
                e.id AS "employee_id?",
                cb.hourly_rate_cents,
                COALESCE(cb.is_salaried, false) AS "is_salaried!",
                cb.monthly_cost_cents,
                COALESCE(cb.weekly_contract_minutes, 0) AS "weekly_contract_minutes!",
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
            LEFT JOIN employee_cost_bases cb
                ON cb.employee_id = e.id
               AND cb.effective_from <= ((COALESCE(t.starts_at, pt.starts_at)) AT TIME ZONE 'UTC')::date
               AND (cb.effective_to IS NULL OR cb.effective_to > ((COALESCE(t.starts_at, pt.starts_at)) AT TIME ZONE 'UTC')::date)
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

        // All-day tasks need the whole version history for the days they
        // cover, not just the one version anchored on their start date above
        // — see `cost_of_assignment`. Scoped to the employees who actually
        // have an all-day assignment in this window, most reports have none.
        let all_day_employee_ids: Vec<uuid::Uuid> = assignments
            .iter()
            .filter(|assignment| assignment.all_day)
            .filter_map(|assignment| assignment.employee_id)
            .map(|employee_id| employee_id.0)
            .collect::<HashSet<_>>()
            .into_iter()
            .collect();

        let cost_bases = if all_day_employee_ids.is_empty() {
            Vec::new()
        } else {
            sqlx::query!(
                r#"
                SELECT id, org_id, employee_id, effective_from, effective_to, is_salaried, hourly_rate_cents, monthly_cost_cents, weekly_contract_minutes, created_at, updated_at
                FROM employee_cost_bases
                WHERE employee_id = ANY($1)
                  AND effective_from < (($3) AT TIME ZONE 'UTC')::date
                  AND (effective_to IS NULL OR effective_to > (($2) AT TIME ZONE 'UTC')::date)
                "#,
                &all_day_employee_ids,
                from,
                to,
            )
            .fetch_all(&mut ***tx)
            .await
            .map_err(map_sqlx_error)?
            .into_iter()
            .map(|row| EmployeeCostBasis {
                id: EmployeeCostBasisId(row.id),
                organization_id: OrganizationId(row.org_id),
                employee_id: EmployeeId(row.employee_id),
                effective_from: row.effective_from,
                effective_to: row.effective_to,
                is_salaried: row.is_salaried,
                hourly_rate_cents: row.hourly_rate_cents,
                monthly_cost_cents: row.monthly_cost_cents,
                weekly_contract_minutes: row.weekly_contract_minutes,
                created_at: row.created_at,
                updated_at: row.updated_at,
            })
            .collect()
        };

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

        // Only a `CONFIRMED` invoice's cost is real: `RECEIVED` is a
        // proposal, `REJECTED` means it never happened. Filtered on the
        // invoice's own `issued_on` — the one date both the supplier and
        // the accountant agree on, per #338 — cast the same `AT TIME ZONE
        // 'UTC'` way the all-day cost-basis lookups above already treat an
        // instant as a calendar day, for the same reason: a plain `::date`
        // cast depends on the session's own timezone setting, and this
        // adapter never wants that.
        let supplier_costs = sqlx::query!(
            r#"
            SELECT
                a.project_id AS "project_id!",
                a.amount_cents AS "net_amount_cents!",
                l.vat_rate_basis_points
            FROM supplier_invoice_line_allocations a
            JOIN supplier_invoice_lines l
                ON l.id = a.supplier_invoice_line_id AND l.org_id = a.org_id
            JOIN supplier_invoices si
                ON si.id = l.supplier_invoice_id AND si.org_id = a.org_id
            WHERE a.org_id = $1
              AND si.status = 'CONFIRMED'::supplier_invoice_status
              AND si.issued_on >= ($2 AT TIME ZONE 'UTC')::date
              AND si.issued_on < ($3 AT TIME ZONE 'UTC')::date
            "#,
            organization_id.0,
            from,
            to,
        )
        .fetch_all(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?
        .into_iter()
        .map(|row| SupplierCostAllocation {
            project_id: ProjectId(row.project_id),
            net_amount_cents: row.net_amount_cents,
            vat_rate_basis_points: row.vat_rate_basis_points,
        })
        .collect();

        // The same `VatStatus` an invoice's own totals are computed from
        // (`invoice::service::calculate_totals`), reused here rather than a
        // second source of truth: only `'subject'` recovers VAT, an
        // incomplete legal identity (`NULL`) reads the same as
        // `'not_subject'` — both cannot recover it.
        let vat_status: Option<String> = sqlx::query_scalar!(
            r#"SELECT vat_status FROM organizations WHERE id = $1"#,
            organization_id.0,
        )
        .fetch_optional(&mut ***tx)
        .await
        .map_err(map_sqlx_error)?
        .flatten();
        let organization_vat_subject = vat_status.as_deref() == Some("subject");

        Ok(ProfitabilityFacts {
            projects,
            assignments,
            expenses,
            equipment,
            cost_bases,
            supplier_costs,
            organization_vat_subject,
        })
    }
}
