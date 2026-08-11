use chrono::{DateTime, NaiveDate, Utc};
use common::CoreError;

use crate::{Absence, EmployeeRhythm, OrganizationId, WorkSlot, domain::planning::PlanningTask};

/// The planning read model's own repository: the tables no other aggregate
/// already owns a suitable batched, organization-wide query for
/// (`tasks`/`task_assignments` enriched with the customer join,
/// `employee_absences`, `employee_rhythms`/`employee_rhythm_slots`,
/// `employee_work_slots`). Resources (`employees`/`organization_members`)
/// and the organization's timezone are read through the existing
/// `EmployeeRepository`/`MemberRepository`/`OrganizationRepository` ports
/// instead — see `domain::planning::service::PlanningService`.
///
/// Every method here loads *the whole organization* in one query and lets
/// the application layer group the result in memory: an org of 40
/// employees must not turn into 40 queries per table (see the planning
/// module design doc's N+1 warning).
#[cfg_attr(test, mockall::automock)]
pub trait PlanningRepository: Send {
    /// Every task (with its assignments and its child count), enriched with
    /// `customer_name`/`context_label`, whose *effective* window overlaps
    /// `[from, to)` — for the whole organization in one small, fixed set of
    /// queries, independent of how many tasks or employees the organization
    /// has. A task's effective window is its own when it carries one, or
    /// its parent's when it doesn't (a subtask inheriting its parent's
    /// window, see `resolve_task_window` in `domain::task::service`) — the
    /// returned `Task.starts_at`/`ends_at` always carry this resolved
    /// value, never `None`, so a dateless subtask assigned to someone
    /// appears on their row exactly like any other task.
    fn list_tasks_in_window(
        &mut self,
        organization_id: OrganizationId,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> impl Future<Output = Result<Vec<PlanningTask>, CoreError>> + Send;

    /// Every absence whose window overlaps `[from, to)`, for the whole
    /// organization in one query.
    fn list_absences_in_window(
        &mut self,
        organization_id: OrganizationId,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> impl Future<Output = Result<Vec<Absence>, CoreError>> + Send;

    /// Every rhythm version (with its slots) overlapping `[from, to]`, for
    /// every employee of the organization, in one query pair (rhythms,
    /// then their slots).
    fn list_rhythms_for_organization(
        &mut self,
        organization_id: OrganizationId,
        from: NaiveDate,
        to: NaiveDate,
    ) -> impl Future<Output = Result<Vec<EmployeeRhythm>, CoreError>> + Send;

    /// Every dated work slot inside `[from, to]`, for every employee of the
    /// organization, in one query.
    fn list_work_slots_for_organization(
        &mut self,
        organization_id: OrganizationId,
        from: NaiveDate,
        to: NaiveDate,
    ) -> impl Future<Output = Result<Vec<WorkSlot>, CoreError>> + Send;
}
