use chrono::{DateTime, NaiveDate, Utc};
use common::CoreError;

use crate::{
    EmployeeAbsence, EmployeeRhythm, EmployeeWorkSlot, OrganizationId,
    domain::planning::PlanningWorkOrder,
};

/// The planning read model's own repository: the tables no other aggregate
/// already owns a suitable batched, organization-wide query for
/// (`work_orders`/`assignments` enriched with the customer join,
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
    /// Every work order (with its assignments), enriched with
    /// `customer_name`/`context_label`, whose window overlaps
    /// `[from, to)` — for the whole organization in one query pair (work
    /// orders, then their assignments).
    fn list_work_orders_in_window(
        &mut self,
        organization_id: OrganizationId,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> impl Future<Output = Result<Vec<PlanningWorkOrder>, CoreError>> + Send;

    /// Every absence whose window overlaps `[from, to)`, for the whole
    /// organization in one query.
    fn list_absences_in_window(
        &mut self,
        organization_id: OrganizationId,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> impl Future<Output = Result<Vec<EmployeeAbsence>, CoreError>> + Send;

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
    ) -> impl Future<Output = Result<Vec<EmployeeWorkSlot>, CoreError>> + Send;
}
