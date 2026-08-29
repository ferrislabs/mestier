use mestier_core::{CustomerId, MemberId, MissingCost, ProfitabilityReport, ProjectId};
use serde::Serialize;
use utoipa::ToSchema;

/// What one project is planned to cost, and what it earns — redacted for a
/// caller without `VIEW_COST` (#306): every money field becomes `None`
/// rather than a number, which is why they are all `Option` here and not
/// on the domain [`mestier_core::ProjectProfitability`] this is built
/// from. Distinguishing "redacted" from "unknown" is
/// [`ProfitabilityResponse::costs_redacted`]'s job, not this struct's: a
/// caller without the bit and a project with no quote must not render
/// the same for a different reason.
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct ProjectProfitabilityResponse {
    pub project_id: ProjectId,
    pub name: String,
    pub customer_id: Option<CustomerId>,
    pub quoted_cents: Option<i32>,
    pub labour_cost_cents: Option<i64>,
    pub equipment_cost_cents: Option<i64>,
    pub expenses_cents: Option<i64>,
    pub supplier_cost_cents: Option<i64>,
    /// Never redacted: a planning fact, not a cost.
    pub planned_minutes: i64,
    /// Never redacted, same reason.
    pub occupied_minutes: i64,
    /// Never redacted: double-booking is a planning problem to go and fix,
    /// not payroll.
    pub overlapping_minutes: i64,
    pub margin_cents: Option<i64>,
    /// Never redacted: this is the list of people planning must chase for
    /// a missing rate, not the rate itself, and gating it behind
    /// `VIEW_COST` would keep it from the one person whose job is to fix
    /// it and who may not have that bit.
    pub members_without_rate: Vec<MemberId>,
}

impl ProjectProfitabilityResponse {
    fn from_domain(value: mestier_core::ProjectProfitability, costs_redacted: bool) -> Self {
        Self {
            project_id: value.project_id,
            name: value.name,
            customer_id: value.customer_id,
            quoted_cents: if costs_redacted {
                None
            } else {
                value.quoted_cents
            },
            labour_cost_cents: if costs_redacted {
                None
            } else {
                Some(value.labour_cost_cents)
            },
            equipment_cost_cents: if costs_redacted {
                None
            } else {
                Some(value.equipment_cost_cents)
            },
            expenses_cents: if costs_redacted {
                None
            } else {
                Some(value.expenses_cents)
            },
            supplier_cost_cents: if costs_redacted {
                None
            } else {
                Some(value.supplier_cost_cents)
            },
            planned_minutes: value.planned_minutes,
            occupied_minutes: value.occupied_minutes,
            overlapping_minutes: value.overlapping_minutes,
            margin_cents: if costs_redacted {
                None
            } else {
                value.margin_cents
            },
            members_without_rate: value.members_without_rate,
        }
    }
}

/// One person's planned cost — redacted the same way
/// [`ProjectProfitabilityResponse`] is, for the same reason.
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct MemberProfitabilityResponse {
    pub member_id: MemberId,
    pub planned_minutes: i64,
    pub labour_cost_cents: Option<i64>,
    pub missing_cost: Option<MissingCost>,
}

impl MemberProfitabilityResponse {
    fn from_domain(value: mestier_core::MemberProfitability, costs_redacted: bool) -> Self {
        Self {
            member_id: value.member_id,
            planned_minutes: value.planned_minutes,
            labour_cost_cents: if costs_redacted {
                None
            } else {
                Some(value.labour_cost_cents)
            },
            missing_cost: value.missing_cost,
        }
    }
}

/// Projects ordered so the answer is directly usable, plus the two ends of the
/// ranking the dashboards ask for.
///
/// Ranked here rather than in the browser: "least profitable" has to mean the
/// same thing on every screen, and a project whose margin is unknown must not be
/// ranked at all rather than sorted as if it were zero.
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct ProfitabilityResponse {
    pub projects: Vec<ProjectProfitabilityResponse>,
    pub members: Vec<MemberProfitabilityResponse>,
    /// Projects with a known margin, best first. Absent margins are left out.
    /// Empty outright when [`Self::costs_redacted`] is set: a margin ranking
    /// is itself a leak of relative cost, the numbers do not have to be
    /// printed for the ordering to say who costs more (#306).
    pub most_profitable: Vec<ProjectProfitabilityResponse>,
    /// The same list from the other end, worst first. Same redaction rule.
    pub least_profitable: Vec<ProjectProfitabilityResponse>,
    /// Projects whose figures rest on incomplete data, so the reader knows which
    /// numbers to distrust and why. Since the cost comes from the plan rather
    /// than from a clock, the only thing that lands a project here is a rate
    /// nobody has set.
    pub incomplete: Vec<ProjectProfitabilityResponse>,
    /// Projects where somebody is booked on two tasks at once, so their time is
    /// billed twice. Not "incomplete": the minutes are known, they are just
    /// double booked, and that is a planning problem to go and fix.
    pub double_booked: Vec<ProjectProfitabilityResponse>,
    /// `true` when the caller lacks `VIEW_COST`: every money field above is
    /// `None` on that account, not because the figure itself is unknown.
    /// Named on the response rather than left for the client to infer from
    /// nulls, because a project with an incomplete rate also nulls
    /// `margin_cents` and the two situations must read differently (#306).
    pub costs_redacted: bool,
}

/// How many of each end of the ranking to return.
///
/// A fixed handful rather than a parameter: these feed a dashboard tile, and a
/// caller wanting the whole list already has `projects`.
const RANKING_SIZE: usize = 5;

impl ProfitabilityResponse {
    pub fn from_report(value: ProfitabilityReport, costs_redacted: bool) -> Self {
        let mut ranked: Vec<mestier_core::ProjectProfitability> = value
            .projects
            .iter()
            .filter(|project| project.margin_cents.is_some())
            .cloned()
            .collect();
        ranked.sort_by_key(|project| project.margin_cents.unwrap_or(0));

        // Empty outright when redacted (see the field's own doc comment),
        // not populated with redacted-shape entries: the ranking itself is
        // what leaks.
        let (least_profitable, most_profitable) = if costs_redacted {
            (Vec::new(), Vec::new())
        } else {
            (
                ranked
                    .iter()
                    .take(RANKING_SIZE)
                    .cloned()
                    .map(|p| ProjectProfitabilityResponse::from_domain(p, false))
                    .collect(),
                ranked
                    .iter()
                    .rev()
                    .take(RANKING_SIZE)
                    .cloned()
                    .map(|p| ProjectProfitabilityResponse::from_domain(p, false))
                    .collect(),
            )
        };

        let incomplete = value
            .projects
            .iter()
            .filter(|project| !project.is_complete())
            .cloned()
            .map(|p| ProjectProfitabilityResponse::from_domain(p, costs_redacted))
            .collect();
        let double_booked = value
            .projects
            .iter()
            .filter(|project| project.overlapping_minutes > 0)
            .cloned()
            .map(|p| ProjectProfitabilityResponse::from_domain(p, costs_redacted))
            .collect();

        Self {
            projects: value
                .projects
                .into_iter()
                .map(|p| ProjectProfitabilityResponse::from_domain(p, costs_redacted))
                .collect(),
            members: value
                .members
                .into_iter()
                .map(|m| MemberProfitabilityResponse::from_domain(m, costs_redacted))
                .collect(),
            most_profitable,
            least_profitable,
            incomplete,
            double_booked,
            costs_redacted,
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
    /// Which figure is missing when this person's cost could not be computed:
    /// an hourly rate, a monthly amount, or the contracted hours to divide it by.
    /// `null` when the cost is known.
    ///
    /// The reason rather than a flag, because the three are fixed in three
    /// different places and a screen that guesses between them sends somebody to
    /// a field they already filled in.
    ///
    /// Kept even when [`WorkedHoursResponse::costs_redacted`] is set (#306):
    /// this names a gap in somebody's payroll *setup*, not their pay, the
    /// same way `members_without_rate` stays visible on the profitability
    /// report — it is what tells whoever manages payroll that a colleague
    /// needs a rate entered, without itself stating an amount.
    pub missing_cost: Option<MissingCost>,
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct WorkedHoursResponse {
    pub members: Vec<WorkedHoursRow>,
    pub total_planned_minutes: i64,
    /// Reflects the caller's `VIEW_COST` status for parity with
    /// [`ProfitabilityResponse::costs_redacted`], even though this response
    /// has no money field to null on that account (see
    /// [`WorkedHoursRow::missing_cost`]'s own doc comment) — a client
    /// reading both reports should not need a special case for the one
    /// that never redacts anything.
    pub costs_redacted: bool,
}

impl WorkedHoursResponse {
    pub fn from_report(value: ProfitabilityReport, costs_redacted: bool) -> Self {
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
                    missing_cost: member.missing_cost,
                })
                .collect(),
            total_planned_minutes,
            costs_redacted,
        }
    }
}
