use mestier_core::{EmployeeProfitability, ProfitabilityReport, TaskProfitability};
use serde::Serialize;
use utoipa::ToSchema;

/// Jobs ordered so the answer is directly usable, plus the two ends of the
/// ranking the dashboards ask for.
///
/// Ranked here rather than in the browser: "least profitable" has to mean the
/// same thing on every screen, and a job whose margin is unknown must not be
/// ranked at all rather than sorted as if it were zero.
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct ProfitabilityResponse {
    pub jobs: Vec<TaskProfitability>,
    pub employees: Vec<EmployeeProfitability>,
    /// Jobs with a known margin, best first. Absent margins are left out.
    pub most_profitable: Vec<TaskProfitability>,
    /// The same list from the other end, worst first.
    pub least_profitable: Vec<TaskProfitability>,
    /// Jobs whose figures rest on incomplete data, so the reader knows which
    /// numbers to distrust and why.
    pub incomplete: Vec<TaskProfitability>,
}

/// How many of each end of the ranking to return.
///
/// A fixed handful rather than a parameter: these feed a dashboard tile, and a
/// caller wanting the whole list already has `jobs`.
const RANKING_SIZE: usize = 5;

impl From<ProfitabilityReport> for ProfitabilityResponse {
    fn from(value: ProfitabilityReport) -> Self {
        let mut ranked: Vec<TaskProfitability> = value
            .jobs
            .iter()
            .filter(|job| job.margin_cents.is_some())
            .cloned()
            .collect();
        ranked.sort_by_key(|job| job.margin_cents.unwrap_or(0));

        let least_profitable = ranked.iter().take(RANKING_SIZE).cloned().collect();
        let most_profitable = ranked.iter().rev().take(RANKING_SIZE).cloned().collect();
        let incomplete = value
            .jobs
            .iter()
            .filter(|job| !job.is_complete())
            .cloned()
            .collect();

        Self {
            jobs: value.jobs,
            employees: value.employees,
            most_profitable,
            least_profitable,
            incomplete,
        }
    }
}

/// One employee's hours over the period, for payroll.
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct WorkedHoursRow {
    pub employee_id: mestier_core::EmployeeId,
    pub worked_minutes: i64,
    /// Entries never clocked off, whose time is missing from the total. Payroll
    /// needs to know the number is short before it pays on it.
    pub open_entries: u32,
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct WorkedHoursResponse {
    pub employees: Vec<WorkedHoursRow>,
    pub total_worked_minutes: i64,
}

impl From<ProfitabilityReport> for WorkedHoursResponse {
    fn from(value: ProfitabilityReport) -> Self {
        let total_worked_minutes = value
            .employees
            .iter()
            .map(|employee| employee.worked_minutes)
            .sum();

        Self {
            employees: value
                .employees
                .into_iter()
                .map(|employee| WorkedHoursRow {
                    employee_id: employee.employee_id,
                    worked_minutes: employee.worked_minutes,
                    open_entries: employee.open_entries,
                })
                .collect(),
            total_worked_minutes,
        }
    }
}
