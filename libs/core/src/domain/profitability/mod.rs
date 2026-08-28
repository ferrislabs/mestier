//! What a job is planned to cost, against what it was quoted.
//!
//! The payoff the whole product is built for. It used to read the clock:
//! `coût = Σ(temps pointé × €/h)`, which meant a task planned, assigned and
//! done but never clocked cost nothing, and work with no customer was invisible
//! entirely. It now reads the plan. A task with three people on it for two
//! hours costs six person-hours, whether or not anybody registered anything.
//! See `docs/adr/0002-planned-cost-model.md`.
//!
//! Read-only, so no commands and no events: this module computes, it never
//! writes.

use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

use crate::{
    CustomerId, EmployeeCostBasis, EmployeeId, EmployeeRhythm, EquipmentId, MemberId, ProjectId,
    TaskId, Tz, WorkSlot, domain::employee::service::salaried_hourly_rate_cents,
};

pub mod ports;
pub mod service;

/// The window a report covers, plus the timezone its calendar days are
/// expressed in.
///
/// The timezone travels with the period because an all-day task has no duration
/// until one is chosen: "Tuesday" is a different number of minutes depending on
/// where the organization lives.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ReportPeriod {
    pub from: DateTime<Utc>,
    /// Excluded, so consecutive periods neither overlap nor leave a gap.
    pub to: DateTime<Utc>,
    pub timezone: Tz,
}

/// One person booked on one planned task, carrying the rate that applied.
///
/// The rate travels with the fact rather than being looked up later: someone's
/// rate can change, and a cost has to be built from what was true. This is no
/// longer an aspiration — the adapter resolves these fields from
/// `employee_cost_bases`, the version covering the task's own start date, not
/// from `employees`' live columns. A raise entered today no longer moves a
/// number this struct already carried for a task planned before it.
///
/// An all-day task's own days can straddle a version boundary the timed
/// task's single start date cannot, so [`service::build_report`] does not use
/// these fields for one: it resolves each covered day against
/// [`ProfitabilityFacts::cost_bases`] instead. These fields still describe the
/// version in effect on the task's start date, kept populated for whichever
/// caller wants that anchor.
#[derive(Debug, Clone, PartialEq)]
pub struct PlannedAssignment {
    /// `None` for a task attached to no project. Such a task still costs the
    /// person's time — it is counted in [`MemberProfitability`] — it simply has
    /// no subject to be charged to.
    pub project_id: Option<ProjectId>,
    pub task_id: TaskId,
    pub member_id: MemberId,
    /// `None` for a member with no contract at all, which is a different
    /// absence from a contract whose rate was left blank. Both land in
    /// [`ProjectProfitability::members_without_rate`], because both mean the
    /// same thing to the reader: go and set a rate for this person.
    pub employee_id: Option<EmployeeId>,
    /// Set for someone costed by the hour. `None` for a salaried person, whose
    /// hourly cost is derived instead — see
    /// [`Self::effective_hourly_rate_cents`].
    pub hourly_rate_cents: Option<i32>,
    /// True when this person is costed from [`Self::monthly_cost_cents`].
    ///
    /// This used to mean "costs nothing", and their time was counted at zero.
    /// That was wrong: a salaried person costs a salary every month, and zeroing
    /// them understated every project they touched, silently, because they were
    /// deliberately kept out of `members_without_rate`. A salaried person is now
    /// simply someone whose rate is computed rather than typed.
    pub is_salaried: bool,
    /// Monthly employer cost, for a salaried person.
    pub monthly_cost_cents: Option<i32>,
    /// The contract the monthly cost is spread over. `0` for someone with no
    /// contract, which is why a salary alone is not enough to state an hourly
    /// cost.
    pub weekly_contract_minutes: i32,
    /// The task's effective window: its own, or its parent's when a subtask
    /// inherits it. Resolved by the adapter in SQL, so the calculation never
    /// walks the hierarchy.
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    /// An all-day task has no clock window to measure. Its cost comes from this
    /// member's work slots on each day it covers — see
    /// [`service::build_report`].
    pub all_day: bool,
}

/// Why an hour of somebody's time cannot be priced.
///
/// Three distinct gaps, and the reader has to be told which one: a screen that
/// says "hourly rate or salary missing" when what is actually missing is the
/// contract sends somebody looking in the wrong place. That happened.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum MissingCost {
    /// Costed by the hour, and no rate was entered.
    HourlyRate,
    /// Costed from a salary, and no amount was entered.
    MonthlyCost,
    /// An amount was entered, but there are no contracted hours to spread it
    /// over, so it has no hourly equivalent.
    ContractedHours,
    /// No cost basis version covers this date at all — for instance a task
    /// planned before the employee's very first version. Kept distinct from
    /// [`Self::HourlyRate`] rather than folded into it: that variant means a
    /// figure was left blank on an existing basis, this one means there was
    /// no basis to read from on that day, and the two send a reader to
    /// different screens.
    NoCostBasis,
}

impl PlannedAssignment {
    /// What an hour of this person costs, or which figure is missing.
    ///
    /// One accessor rather than a branch at each call site: the project total and
    /// the per-person total must never disagree about what somebody costs, and
    /// two copies of this rule is how they would. Returning the *reason* rather
    /// than a bare `None` is what lets the screen name the field to go and fill.
    pub fn hourly_cost_cents(&self) -> Result<i32, MissingCost> {
        hourly_cost_cents_from(
            self.is_salaried,
            self.hourly_rate_cents,
            self.monthly_cost_cents,
            self.weekly_contract_minutes,
        )
    }
}

/// The shared rule behind [`PlannedAssignment::hourly_cost_cents`] and
/// [`service::cost_basis_hourly_cost_cents`]: an assignment and a cost basis
/// version carry the same four fields, and the arithmetic over them must not
/// be written twice to drift apart.
pub(crate) fn hourly_cost_cents_from(
    is_salaried: bool,
    hourly_rate_cents: Option<i32>,
    monthly_cost_cents: Option<i32>,
    weekly_contract_minutes: i32,
) -> Result<i32, MissingCost> {
    if !is_salaried {
        return hourly_rate_cents.ok_or(MissingCost::HourlyRate);
    }

    let monthly_cost_cents = monthly_cost_cents.ok_or(MissingCost::MonthlyCost)?;

    salaried_hourly_rate_cents(Some(monthly_cost_cents), weekly_contract_minutes)
        .ok_or(MissingCost::ContractedHours)
}

/// Money a task cost beyond somebody's time.
#[derive(Debug, Clone, PartialEq)]
pub struct TaskExpense {
    pub project_id: ProjectId,
    pub task_id: TaskId,
    pub expenses_cents: i32,
    /// The task's effective start. An expense is a discrete event rather than
    /// something that accrues, so it belongs whole to the period the task
    /// starts in — never split across two.
    pub starts_at: DateTime<Utc>,
}

/// A machine on a project, with what an hour of it costs.
#[derive(Debug, Clone, PartialEq)]
pub struct AssignedEquipment {
    pub project_id: ProjectId,
    pub equipment_id: EquipmentId,
    pub hourly_rate_cents: i32,
}

/// One project's share of a confirmed supplier invoice line, in scope for
/// the report's period — see #338.
///
/// Filtered by the adapter on the invoice's own `issued_on` falling in the
/// period, and on the invoice's `status` being `Confirmed`: a `Received`
/// document is a proposal and must never move a number, and a project's own
/// work dates or the invoice's reception date were both considered and
/// rejected as the period key, because either can differ from `issued_on`
/// by months — see [`service::supplier_cost_cents`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SupplierCostAllocation {
    pub project_id: ProjectId,
    /// Net of VAT, a slice of its line's own `line_total_cents` — never the
    /// gross figure. Turned into a real cost by
    /// [`service::supplier_cost_cents`], which adds the rate back in when
    /// the organization cannot recover it.
    pub net_amount_cents: i32,
    /// The rate to add back in when VAT is not recoverable. `None` for a
    /// line the supplier printed no rate on, read as 0 bp: a rateless line
    /// is exactly as costly net as gross.
    pub vat_rate_basis_points: Option<i32>,
}

/// A project as the calculation needs to know it, before any arithmetic.
#[derive(Debug, Clone, PartialEq)]
pub struct ProjectHeader {
    pub project_id: ProjectId,
    pub name: String,
    /// `None` for an internal project. Not missing data: a recurring meeting
    /// bills nobody and still costs money.
    pub customer_id: Option<CustomerId>,
    /// The quote's total, when the project carries a `quote_id`. Absent means
    /// no margin can be stated, not a margin of zero.
    pub quoted_cents: Option<i32>,
}

/// Everything the calculation reads from the profitability adapter.
///
/// A flat set of facts rather than a nested tree: the adapter fetches each list
/// with one query, which is what keeps a hundred projects from becoming three
/// hundred round trips.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ProfitabilityFacts {
    pub projects: Vec<ProjectHeader>,
    pub assignments: Vec<PlannedAssignment>,
    pub expenses: Vec<TaskExpense>,
    pub equipment: Vec<AssignedEquipment>,
    /// Cost basis versions for every employee with an all-day assignment in
    /// scope, overlapping the report window. A timed task needs none of
    /// these — its one date already resolved a single version into
    /// [`PlannedAssignment`]'s own fields — but an all-day task's days can
    /// straddle a version boundary its own start date does not see, so
    /// [`service::build_report`] resolves each covered day against this list
    /// instead. Not part of [`PlannedAssignment`]'s frozen shape on purpose:
    /// this is history for a lookup, not a fact carried per assignment.
    pub cost_bases: Vec<EmployeeCostBasis>,
    /// Every allocation of a confirmed supplier invoice line falling in the
    /// report's period — see [`SupplierCostAllocation`].
    pub supplier_costs: Vec<SupplierCostAllocation>,
    /// Whether the organization can recover VAT on what it buys — read from
    /// the same `VatStatus` an invoice's own totals are computed from (see
    /// `invoice::service::calculate_totals`), reused here rather than a
    /// second source of truth. `true` only for `VatStatus::Subject`; `None`
    /// (legal identity incomplete) and `VatStatus::NotSubject` both mean
    /// `false` — the organization cannot recover the VAT it pays, so that
    /// VAT is exactly as real a cost as the net price.
    pub organization_vat_subject: bool,
}

/// What all-day tasks need in order to have a duration at all.
///
/// Loaded through `PlanningRepository`'s existing organization-wide queries
/// rather than a second copy of them in the profitability adapter — see
/// [`service::ProfitabilityService`]. Kept out of [`ProfitabilityFacts`] so
/// neither struct is ever half-filled.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct WorkTime {
    pub rhythms: Vec<EmployeeRhythm>,
    pub work_slots: Vec<WorkSlot>,
}

/// What one project is planned to cost, and what it earns.
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct ProjectProfitability {
    pub project_id: ProjectId,
    pub name: String,
    pub customer_id: Option<CustomerId>,
    pub quoted_cents: Option<i32>,
    pub labour_cost_cents: i64,
    pub equipment_cost_cents: i64,
    pub expenses_cents: i64,
    /// What confirmed supplier invoices allocated to this project cost in
    /// the period — net of VAT when the organization can recover it, gross
    /// when it cannot. See [`SupplierCostAllocation`] and
    /// [`service::supplier_cost_cents`].
    pub supplier_cost_cents: i64,
    /// Sum of everyone's planned minutes. Two people for an hour is 120.
    pub planned_minutes: i64,
    /// Wall-clock minutes at least one person is booked on the project. Two
    /// people over the same hour is 60, which is how long the machines run.
    pub occupied_minutes: i64,
    /// Minutes billed twice because one person is booked on two of this
    /// project's tasks at once.
    ///
    /// Reported rather than deduplicated. `detect_conflicts` already warns about
    /// overlaps and never refuses them, so they happen; silently collapsing them
    /// would make the cost impossible to reconcile by hand, and this screen
    /// exists to be trusted. Already included in [`Self::planned_minutes`] and
    /// [`Self::labour_cost_cents`].
    pub overlapping_minutes: i64,
    /// `None` when the project has no quote, or when a rate is missing: a
    /// margin computed from a floor reads as fact and is not one.
    pub margin_cents: Option<i64>,
    /// People booked on this project whose hourly cost cannot be stated: an
    /// hourly rate nobody entered, no contract at all, or a salary with no
    /// contracted hours to spread it over. While this is not empty the cost is a
    /// lower bound, and no margin is stated.
    pub members_without_rate: Vec<MemberId>,
}

impl ProjectProfitability {
    pub fn planned_cost_cents(&self) -> i64 {
        self.labour_cost_cents
            + self.equipment_cost_cents
            + self.expenses_cents
            + self.supplier_cost_cents
    }

    /// Whether every figure here rests on complete data.
    ///
    /// An overlap does not make a project incomplete: the minutes are known,
    /// they are simply booked twice, and that is a planning problem rather than
    /// a missing input.
    pub fn is_complete(&self) -> bool {
        self.members_without_rate.is_empty()
    }
}

/// What one person is planned to cost over the period, and how long they work.
///
/// Counts every assignment, including tasks attached to no project: payroll
/// cares about the hours whether or not somebody decided which subject they
/// belong to.
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct MemberProfitability {
    pub member_id: MemberId,
    pub planned_minutes: i64,
    pub labour_cost_cents: i64,
    /// Which figure is missing, when the cost could not be computed. `None`
    /// means it was. The cost above is zero because it is unknown, never because
    /// the time is free.
    pub missing_cost: Option<MissingCost>,
}

/// The whole answer for one organization over one period.
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct ProfitabilityReport {
    pub projects: Vec<ProjectProfitability>,
    pub members: Vec<MemberProfitability>,
}
