//! What a job actually cost, against what it was quoted.
//!
//! The payoff the whole product is built for: `coût réel = Σ(temps × €/h)`,
//! compared to the quote, giving a margin. Read-only, so no commands and no
//! events: this module computes, it never writes.
//!
//! "Chantier" is not a type here, because it is not one in the domain either.
//! A chantier is a root [`crate::Task`] carrying a customer, and its cost
//! aggregates its own clocked time and its subtasks'.

use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;

use crate::{CustomerId, EmployeeId, EquipmentId, TaskId};

pub mod ports;
pub mod service;

/// One clocked stretch, carrying the rate that applied to whoever worked it.
///
/// The rate travels with the fact rather than being looked up later: an
/// employee's rate can change, and a cost has to be built from what was true,
/// not from what is true now.
#[derive(Debug, Clone, PartialEq)]
pub struct ClockedTime {
    /// The root task the time belongs to, already resolved by the adapter, so
    /// the service never walks the hierarchy.
    pub task_id: TaskId,
    pub employee_id: EmployeeId,
    /// `None` when the employee has no rate yet. Never treated as zero: see
    /// [`TaskProfitability::employees_without_rate`].
    pub hourly_rate_cents: Option<i32>,
    pub started_at: DateTime<Utc>,
    /// `None` for an entry nobody clocked off. It contributes no time, and is
    /// counted in [`TaskProfitability::open_entries`] so the figure can say so.
    pub ended_at: Option<DateTime<Utc>>,
}

/// A machine on a job, with what an hour of it costs.
#[derive(Debug, Clone, PartialEq)]
pub struct AssignedEquipment {
    pub task_id: TaskId,
    pub equipment_id: EquipmentId,
    pub hourly_rate_cents: i32,
}

/// A job as the calculation needs to know it, before any arithmetic.
#[derive(Debug, Clone, PartialEq)]
pub struct JobHeader {
    pub task_id: TaskId,
    pub title: String,
    pub customer_id: CustomerId,
    /// The quote's total, when the root task carries a `quote_id`. Absent means
    /// no margin can be stated, not a margin of zero.
    pub quoted_cents: Option<i32>,
}

/// Everything the calculation reads, gathered in one go.
///
/// A flat set of facts rather than a nested tree: the adapter fetches each list
/// with one query, which is what keeps a hundred jobs from becoming three
/// hundred round trips.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ProfitabilityFacts {
    pub jobs: Vec<JobHeader>,
    pub clocked: Vec<ClockedTime>,
    pub equipment: Vec<AssignedEquipment>,
}

/// What one job cost and what it earned.
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct TaskProfitability {
    pub task_id: TaskId,
    pub title: String,
    pub customer_id: CustomerId,
    pub quoted_cents: Option<i32>,
    pub labour_cost_cents: i64,
    pub equipment_cost_cents: i64,
    /// Sum of everyone's clocked minutes. Two people for an hour is 120.
    pub worked_minutes: i64,
    /// Wall-clock minutes at least one person was clocked on. Two people for
    /// the same hour is 60, which is how long the machines ran.
    pub occupied_minutes: i64,
    /// `None` when the job has no quote, or when the cost is incomplete: a
    /// margin computed from a floor is a number that reads as fact and is not.
    pub margin_cents: Option<i64>,
    /// Employees on this job whose hourly rate was never entered. While this is
    /// not empty the cost is a lower bound, and no margin is stated.
    pub employees_without_rate: Vec<EmployeeId>,
    /// Entries nobody clocked off. They contribute nothing, and their presence
    /// says the figure is missing time rather than that the time was free.
    pub open_entries: u32,
}

impl TaskProfitability {
    pub fn real_cost_cents(&self) -> i64 {
        self.labour_cost_cents + self.equipment_cost_cents
    }

    /// Whether every figure here rests on complete data.
    pub fn is_complete(&self) -> bool {
        self.employees_without_rate.is_empty() && self.open_entries == 0
    }
}

/// What one employee cost over the period, and how long they worked.
///
/// The hours are what payroll needs; the cost is what the job rankings are
/// built from.
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct EmployeeProfitability {
    pub employee_id: EmployeeId,
    pub worked_minutes: i64,
    pub labour_cost_cents: i64,
    /// True when the rate was never entered, in which case the cost is zero
    /// because it is unknown, not because the work was free.
    pub rate_missing: bool,
    pub open_entries: u32,
}

/// The whole answer for one organization over one period.
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct ProfitabilityReport {
    pub jobs: Vec<TaskProfitability>,
    pub employees: Vec<EmployeeProfitability>,
}
