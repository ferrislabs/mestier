use mestier_core::{MemberProfitability, ProfitabilityReport, ProjectProfitability};
use serde::Serialize;
use utoipa::ToSchema;

/// Projects ordered so the answer is directly usable, plus the two ends of the
/// ranking the dashboards ask for.
///
/// Ranked here rather than in the browser: "least profitable" has to mean the
/// same thing on every screen, and a project whose margin is unknown must not be
/// ranked at all rather than sorted as if it were zero.
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct ProfitabilityResponse {
    pub projects: Vec<ProjectProfitability>,
    pub members: Vec<MemberProfitability>,
    /// Projects with a known margin, best first. Absent margins are left out.
    pub most_profitable: Vec<ProjectProfitability>,
    /// The same list from the other end, worst first.
    pub least_profitable: Vec<ProjectProfitability>,
    /// Projects whose figures rest on incomplete data, so the reader knows which
    /// numbers to distrust and why. Since the cost comes from the plan rather
    /// than from a clock, the only thing that lands a project here is a rate
    /// nobody has set.
    pub incomplete: Vec<ProjectProfitability>,
    /// Projects where somebody is booked on two tasks at once, so their time is
    /// billed twice. Not "incomplete": the minutes are known, they are just
    /// double booked, and that is a planning problem to go and fix.
    pub double_booked: Vec<ProjectProfitability>,
}

/// How many of each end of the ranking to return.
///
/// A fixed handful rather than a parameter: these feed a dashboard tile, and a
/// caller wanting the whole list already has `projects`.
const RANKING_SIZE: usize = 5;

impl From<ProfitabilityReport> for ProfitabilityResponse {
    fn from(value: ProfitabilityReport) -> Self {
        let mut ranked: Vec<ProjectProfitability> = value
            .projects
            .iter()
            .filter(|project| project.margin_cents.is_some())
            .cloned()
            .collect();
        ranked.sort_by_key(|project| project.margin_cents.unwrap_or(0));

        let least_profitable = ranked.iter().take(RANKING_SIZE).cloned().collect();
        let most_profitable = ranked.iter().rev().take(RANKING_SIZE).cloned().collect();
        let incomplete = value
            .projects
            .iter()
            .filter(|project| !project.is_complete())
            .cloned()
            .collect();
        let double_booked = value
            .projects
            .iter()
            .filter(|project| project.overlapping_minutes > 0)
            .cloned()
            .collect();

        Self {
            projects: value.projects,
            members: value.members,
            most_profitable,
            least_profitable,
            incomplete,
            double_booked,
        }
    }
}

/// One person's hours over the period, for payroll.
///
/// These are *planned* hours. Nobody clocks in any more, so there is no measured
/// number to pay on: the plan is the record, and a correction to it is a
/// management act rather than a missing punch card. See
/// `docs/adr/0002-planned-cost-model.md`.
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct WorkedHoursRow {
    pub member_id: mestier_core::MemberId,
    pub planned_minutes: i64,
    /// True when this person has no hourly rate set, so their cost is zero
    /// because it is unknown rather than because their time is free. Payroll
    /// needs to know that before it pays on the number.
    pub rate_missing: bool,
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct WorkedHoursResponse {
    pub members: Vec<WorkedHoursRow>,
    pub total_planned_minutes: i64,
}

impl From<ProfitabilityReport> for WorkedHoursResponse {
    fn from(value: ProfitabilityReport) -> Self {
        let total_planned_minutes = value
            .members
            .iter()
            .map(|member| member.planned_minutes)
            .sum();

        Self {
            members: value
                .members
                .into_iter()
                .map(|member| WorkedHoursRow {
                    member_id: member.member_id,
                    planned_minutes: member.planned_minutes,
                    rate_missing: member.rate_missing,
                })
                .collect(),
            total_planned_minutes,
        }
    }
}
