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
    CustomerId, EmployeeId, EmployeeRhythm, EquipmentId, MemberId, ProjectId, TaskId, Tz, WorkSlot,
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
/// rate can change, and a cost has to be built from what was true.
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
    pub hourly_rate_cents: Option<i32>,
    /// True when this person is not costed by the hour at all. Their time still
    /// counts as planned, at zero labour cost, and it is never mistaken for a
    /// missing rate.
    pub is_salaried: bool,
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
    /// People booked on this project whose hourly rate is not set, either
    /// because their contract leaves it blank or because they have no contract.
    /// While this is not empty the cost is a lower bound, and no margin is
    /// stated.
    pub members_without_rate: Vec<MemberId>,
}

impl ProjectProfitability {
    pub fn planned_cost_cents(&self) -> i64 {
        self.labour_cost_cents + self.equipment_cost_cents + self.expenses_cents
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
    /// True when no rate is set, in which case the cost is zero because it is
    /// unknown, not because the time is free.
    pub rate_missing: bool,
}

/// The whole answer for one organization over one period.
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct ProfitabilityReport {
    pub projects: Vec<ProjectProfitability>,
    pub members: Vec<MemberProfitability>,
}
