use std::collections::{BTreeMap, HashMap};

use chrono::{DateTime, Duration, NaiveDate, TimeZone, Utc};
use common::CoreError;

use crate::{
    DateRange, EmployeeCostBasis, EmployeeId, EmployeeRhythm, MemberId, MinuteInterval,
    MissingCost, OrganizationId, Tz, WorkSlot,
    domain::{
        planning::ports::PlanningRepository,
        profitability::{
            AssignedEquipment, MemberProfitability, PlannedAssignment, ProfitabilityFacts,
            ProfitabilityReport, ProjectHeader, ProjectProfitability, ReportPeriod,
            SupplierCostAllocation, TaskExpense, WorkTime, hourly_cost_cents_from,
            ports::ProfitabilityRepository,
        },
        work_time::service::expand_work_slots,
    },
};

/// Composes the profitability adapter with the planning one.
///
/// The second is there only for all-day tasks, whose duration comes from the
/// member's work slots. `PlanningRepository` already loads rhythms and slots for
/// a whole organization in a fixed number of queries, and reimplementing that in
/// the profitability adapter would be a second copy of the same SQL, free to
/// drift. Cross-aggregate composition in the domain service that owns the use
/// case is the same shape `PlanningService` already uses.
pub struct ProfitabilityService<R, P>
where
    R: ProfitabilityRepository,
    P: PlanningRepository,
{
    repo: R,
    planning: P,
}

impl<R, P> ProfitabilityService<R, P>
where
    R: ProfitabilityRepository,
    P: PlanningRepository,
{
    pub fn new(repo: R, planning: P) -> Self {
        Self { repo, planning }
    }

    pub async fn report(
        &mut self,
        organization_id: OrganizationId,
        period: ReportPeriod,
    ) -> Result<ProfitabilityReport, CoreError> {
        let facts = self
            .repo
            .load(organization_id, period.from, period.to)
            .await?;

        // Only all-day tasks need work time, and most periods contain none.
        // Two queries skipped is worth the branch on a report that fans out
        // over every project in the organization.
        let work_time = if facts.assignments.iter().any(|entry| entry.all_day) {
            let range = local_date_range(period);

            WorkTime {
                rhythms: self
                    .planning
                    .list_rhythms_for_organization(organization_id, range.from, range.to)
                    .await?,
                work_slots: self
                    .planning
                    .list_work_slots_for_organization(organization_id, range.from, range.to)
                    .await?,
            }
        } else {
            WorkTime::default()
        };

        Ok(build_report(facts, work_time, period))
    }
}

/// The whole calculation, pure and separately testable.
pub fn build_report(
    facts: ProfitabilityFacts,
    work_time: WorkTime,
    period: ReportPeriod,
) -> ProfitabilityReport {
    let ProfitabilityFacts {
        projects,
        assignments,
        expenses,
        equipment,
        cost_bases,
        supplier_costs,
        organization_vat_subject,
    } = facts;

    let expanded = expand_for_members(&assignments, &work_time, period);
    // Resolved once per assignment and indexed alongside it: every project and
    // every member below reads the same spans, and expanding an all-day task
    // twice would be both slower and a chance for the two answers to disagree.
    let spans: Vec<Vec<Span>> = assignments
        .iter()
        .map(|assignment| spans_of(assignment, &expanded, period))
        .collect();
    // Same reasoning as `spans`: the cost of an all-day task depends on which
    // day each of its spans falls on, so it is resolved once here rather than
    // inside both `project_profitability` and `member_profitability`, which
    // would otherwise have to agree on it independently.
    let costs: Vec<Result<i64, MissingCost>> = assignments
        .iter()
        .zip(&spans)
        .map(|(assignment, assignment_spans)| {
            cost_of_assignment(assignment, assignment_spans, &cost_bases, period.timezone)
        })
        .collect();

    ProfitabilityReport {
        projects: projects
            .iter()
            .map(|project| {
                project_profitability(
                    project,
                    &assignments,
                    &spans,
                    &costs,
                    &expenses,
                    &equipment,
                    &supplier_costs,
                    organization_vat_subject,
                    period,
                )
            })
            .collect(),
        members: member_profitability(&assignments, &spans, &costs),
    }
}

/// The real cost of one supplier cost allocation: net when the organization
/// can recover the VAT, gross — the rate added back in — when it cannot.
///
/// The sharpest call #338 makes: VAT paid and never recovered is exactly as
/// much a cost as the net price it sits on top of, so a non-subject
/// organization's number has to include it, while a subject one's number
/// would overstate the true cost by folding in VAT that comes straight back
/// as a credit. Rounded the same half-even way `SupplierInvoiceService`
/// computes VAT on a line, `net.abs()` so a credit line's negative amount
/// grosses up correctly instead of flipping sign.
fn supplier_cost_of(allocation: &SupplierCostAllocation, vat_subject: bool) -> i64 {
    let net = i64::from(allocation.net_amount_cents);
    if vat_subject {
        return net;
    }

    let rate_bp = i64::from(allocation.vat_rate_basis_points.unwrap_or(0));
    let vat_magnitude = div_round_half_even(net.abs() * rate_bp, 10_000);

    net + net.signum() * vat_magnitude
}

/// The version of `employee_id`'s cost basis in effect on `date`, if any.
fn cost_basis_for(
    cost_bases: &[EmployeeCostBasis],
    employee_id: EmployeeId,
    date: NaiveDate,
) -> Option<&EmployeeCostBasis> {
    cost_bases.iter().find(|basis| {
        basis.employee_id == employee_id
            && basis.effective_from <= date
            && basis
                .effective_to
                .is_none_or(|effective_to| effective_to > date)
    })
}

/// What an hour under this cost basis version costs, or which figure is
/// missing — the same rule [`PlannedAssignment::hourly_cost_cents`] applies,
/// read from a version instead of from the assignment's own anchor fields.
pub(crate) fn cost_basis_hourly_cost_cents(basis: &EmployeeCostBasis) -> Result<i32, MissingCost> {
    hourly_cost_cents_from(
        basis.is_salaried,
        basis.hourly_rate_cents,
        basis.monthly_cost_cents,
        basis.weekly_contract_minutes,
    )
}

/// The whole cost of one assignment's spans.
///
/// A timed task gives exactly one span, already anchored to the version
/// covering its own start date by the adapter — one rate for the whole
/// assignment, same as before this issue. An all-day task can give one span
/// per day it covers, and those days can straddle a version boundary the same
/// way a report period can, so each is resolved against `cost_bases`
/// individually rather than all being costed at the assignment's single
/// anchor rate.
fn cost_of_assignment(
    assignment: &PlannedAssignment,
    assignment_spans: &[Span],
    cost_bases: &[EmployeeCostBasis],
    timezone: Tz,
) -> Result<i64, MissingCost> {
    if !assignment.all_day {
        let minutes: i64 = assignment_spans.iter().copied().map(Span::minutes).sum();
        return assignment
            .hourly_cost_cents()
            .map(|rate| cost_of(minutes, i64::from(rate)));
    }

    // No employee profile at all is the same gap a timed task reports as
    // `HourlyRate` by default — there is no basis to look up per day either
    // way.
    let employee_id = assignment.employee_id.ok_or(MissingCost::HourlyRate)?;

    let mut total = 0_i64;
    for span in assignment_spans {
        let date = span.starts_at.with_timezone(&timezone).date_naive();
        let basis =
            cost_basis_for(cost_bases, employee_id, date).ok_or(MissingCost::NoCostBasis)?;
        let rate = cost_basis_hourly_cost_cents(basis)?;
        total += cost_of(span.minutes(), i64::from(rate));
    }

    Ok(total)
}

/// A stretch of real time somebody is booked for.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
struct Span {
    starts_at: DateTime<Utc>,
    ends_at: DateTime<Utc>,
}

impl Span {
    fn minutes(self) -> i64 {
        (self.ends_at - self.starts_at).num_minutes()
    }
}

/// The calendar days the period touches, in the organization's timezone.
///
/// `to` is exclusive as an instant, so the last day is the one containing the
/// instant just before it. Asking for `to`'s own day would pull in a day the
/// period does not cover.
fn local_date_range(period: ReportPeriod) -> DateRange {
    DateRange {
        from: period.from.with_timezone(&period.timezone).date_naive(),
        to: (period.to - Duration::milliseconds(1))
            .with_timezone(&period.timezone)
            .date_naive(),
    }
}

/// Work time expanded per member, for the members who need it.
///
/// Filtered per member the way `build_work_time` does it in the planning
/// service: rhythms belong to an employee record, dated slots to a member, and
/// someone with no contract has no rhythm but may still have slots.
fn expand_for_members(
    assignments: &[PlannedAssignment],
    work_time: &WorkTime,
    period: ReportPeriod,
) -> HashMap<MemberId, BTreeMap<NaiveDate, Vec<MinuteInterval>>> {
    let range = local_date_range(period);
    let mut expanded = HashMap::new();

    for assignment in assignments.iter().filter(|entry| entry.all_day) {
        if expanded.contains_key(&assignment.member_id) {
            continue;
        }

        let rhythms: Vec<EmployeeRhythm> = match assignment.employee_id {
            Some(employee_id) => work_time
                .rhythms
                .iter()
                .filter(|rhythm| rhythm.employee_id == employee_id)
                .cloned()
                .collect(),
            None => Vec::new(),
        };
        let slots: Vec<WorkSlot> = work_time
            .work_slots
            .iter()
            .filter(|slot| slot.member_id == assignment.member_id)
            .cloned()
            .collect();

        expanded.insert(
            assignment.member_id,
            expand_work_slots(&rhythms, &slots, range),
        );
    }

    expanded
}

/// Every stretch one assignment contributes, clipped to the period.
///
/// A timed task gives one span. An all-day task gives one per work slot on each
/// day it covers, which is why a 9-12 / 13-17 day costs 420 minutes and the
/// lunch break is never billed: `expand_work_slots` returns the intervals, not
/// the amplitude.
fn spans_of(
    assignment: &PlannedAssignment,
    expanded: &HashMap<MemberId, BTreeMap<NaiveDate, Vec<MinuteInterval>>>,
    period: ReportPeriod,
) -> Vec<Span> {
    if !assignment.all_day {
        return clip(
            Span {
                starts_at: assignment.starts_at,
                ends_at: assignment.ends_at,
            },
            period,
        )
        .into_iter()
        .collect();
    }

    // No expanded day means no contract and no dated slot: nothing is planned
    // against a week this person does not have, so the task costs nothing rather
    // than a guessed seven hours.
    let Some(days) = expanded.get(&assignment.member_id) else {
        return Vec::new();
    };

    let first = assignment
        .starts_at
        .with_timezone(&period.timezone)
        .date_naive();
    let last = (assignment.ends_at - Duration::milliseconds(1))
        .with_timezone(&period.timezone)
        .date_naive();

    let mut spans = Vec::new();
    for (date, intervals) in days.range(first..=last) {
        for interval in intervals {
            if let Some(span) =
                slot_span(*date, *interval, period.timezone).and_then(|span| clip(span, period))
            {
                spans.push(span);
            }
        }
    }

    spans
}

/// One minute-of-day interval turned into real instants.
///
/// Built by adding minutes to local midnight rather than by constructing an hour
/// and a minute: a slot may legitimately end at 1440, and `and_hms_opt(24, ..)`
/// has no answer for that.
///
/// `.earliest()` rather than `.single()`: on a spring-forward night the wall
/// clock skips an hour and the exact local time does not exist. Same choice the
/// period bounds already make in `period_bounds`.
fn slot_span(date: NaiveDate, interval: MinuteInterval, timezone: Tz) -> Option<Span> {
    let midnight = date.and_hms_opt(0, 0, 0)?;
    let starts_local = midnight + Duration::minutes(i64::from(interval.starts_minute));
    let ends_local = midnight + Duration::minutes(i64::from(interval.ends_minute));

    let starts_at = timezone
        .from_local_datetime(&starts_local)
        .earliest()?
        .with_timezone(&Utc);
    let ends_at = timezone
        .from_local_datetime(&ends_local)
        .earliest()?
        .with_timezone(&Utc);

    (ends_at > starts_at).then_some(Span { starts_at, ends_at })
}

/// A task straddling the period boundary contributes only the minutes inside
/// it, so consecutive periods add up to the whole task and never double it.
fn clip(span: Span, period: ReportPeriod) -> Option<Span> {
    let starts_at = span.starts_at.max(period.from);
    let ends_at = span.ends_at.min(period.to);

    (ends_at > starts_at).then_some(Span { starts_at, ends_at })
}

/// Wall-clock minutes covered by at least one span: their union, not their sum.
fn union_minutes(spans: &[Span]) -> i64 {
    let mut sorted = spans.to_vec();
    sorted.sort_unstable();

    let mut total = 0_i64;
    let mut open: Option<Span> = None;

    for span in sorted {
        open = match open {
            Some(current) if span.starts_at <= current.ends_at => Some(Span {
                starts_at: current.starts_at,
                ends_at: current.ends_at.max(span.ends_at),
            }),
            Some(current) => {
                total += current.minutes();
                Some(span)
            }
            None => Some(span),
        };
    }

    total + open.map_or(0, Span::minutes)
}

#[allow(clippy::too_many_arguments)]
fn project_profitability(
    project: &ProjectHeader,
    assignments: &[PlannedAssignment],
    spans: &[Vec<Span>],
    costs: &[Result<i64, MissingCost>],
    expenses: &[TaskExpense],
    equipment: &[AssignedEquipment],
    supplier_costs: &[SupplierCostAllocation],
    organization_vat_subject: bool,
    period: ReportPeriod,
) -> ProjectProfitability {
    let mut labour_cost_cents = 0_i64;
    let mut planned_minutes = 0_i64;
    let mut members_without_rate: Vec<MemberId> = Vec::new();
    let mut project_spans: Vec<Span> = Vec::new();
    let mut by_member: HashMap<MemberId, Vec<Span>> = HashMap::new();

    for ((assignment, assignment_spans), cost) in assignments.iter().zip(spans).zip(costs) {
        if assignment.project_id != Some(project.project_id) || assignment_spans.is_empty() {
            continue;
        }

        let minutes: i64 = assignment_spans.iter().map(|span| span.minutes()).sum();
        planned_minutes += minutes;
        project_spans.extend(assignment_spans.iter().copied());
        by_member
            .entry(assignment.member_id)
            .or_default()
            .extend(assignment_spans.iter().copied());

        // No special case for a salaried person: their rate is derived rather
        // than typed, and an unstateable one is an unstateable one whichever
        // basis it came from. This branch used to skip them entirely and count
        // their time at zero.
        match cost {
            Ok(cost_cents) => labour_cost_cents += cost_cents,
            // Recorded once per person, not once per assignment: the reader needs
            // to know who to go and set a figure for, not how many times it hurt.
            // Which figure it is lives on the per-person total, where there is
            // room to say it.
            Err(_) if !members_without_rate.contains(&assignment.member_id) => {
                members_without_rate.push(assignment.member_id);
            }
            Err(_) => {}
        }
    }

    // Per person, the gap between what is billed and the wall clock they
    // actually occupy. Summed across people, because two of them overlapping
    // each other is normal teamwork rather than a double booking.
    let overlapping_minutes: i64 = by_member
        .values()
        .map(|spans| spans.iter().map(|span| span.minutes()).sum::<i64>() - union_minutes(spans))
        .sum();

    // The machines run for as long as somebody is on the project, which is the
    // union of the spans rather than their sum: two people over the same hour is
    // one hour of mower, not two.
    let occupied_minutes = union_minutes(&project_spans);
    let equipment_rate_cents: i64 = equipment
        .iter()
        .filter(|item| item.project_id == project.project_id)
        .map(|item| i64::from(item.hourly_rate_cents))
        .sum();
    let equipment_cost_cents = cost_of(occupied_minutes, equipment_rate_cents);

    let expenses_cents: i64 = expenses
        .iter()
        .filter(|expense| {
            expense.project_id == project.project_id
                && expense.starts_at >= period.from
                && expense.starts_at < period.to
        })
        .map(|expense| i64::from(expense.expenses_cents))
        .sum();

    let supplier_cost_cents: i64 = supplier_costs
        .iter()
        .filter(|allocation| allocation.project_id == project.project_id)
        .map(|allocation| supplier_cost_of(allocation, organization_vat_subject))
        .sum();

    // Withheld while a rate is missing. A margin built on a floor reads as fact
    // and is not one, and this screen exists to be trusted.
    let margin_cents = project
        .quoted_cents
        .filter(|_| members_without_rate.is_empty())
        .map(|quoted| {
            i64::from(quoted)
                - (labour_cost_cents + equipment_cost_cents + expenses_cents + supplier_cost_cents)
        });

    ProjectProfitability {
        project_id: project.project_id,
        name: project.name.clone(),
        customer_id: project.customer_id,
        quoted_cents: project.quoted_cents,
        labour_cost_cents,
        equipment_cost_cents,
        expenses_cents,
        supplier_cost_cents,
        planned_minutes,
        occupied_minutes,
        overlapping_minutes,
        margin_cents,
        members_without_rate,
    }
}

/// Per-person totals across every assignment in the period, project or not.
///
/// Ordered by id so the answer is stable: a ranking that reshuffles between two
/// identical requests is a ranking nobody trusts.
fn member_profitability(
    assignments: &[PlannedAssignment],
    spans: &[Vec<Span>],
    costs: &[Result<i64, MissingCost>],
) -> Vec<MemberProfitability> {
    let mut totals: HashMap<MemberId, MemberProfitability> = HashMap::new();

    for ((assignment, assignment_spans), cost) in assignments.iter().zip(spans).zip(costs) {
        if assignment_spans.is_empty() {
            continue;
        }

        let minutes: i64 = assignment_spans.iter().map(|span| span.minutes()).sum();
        let total = totals
            .entry(assignment.member_id)
            .or_insert_with(|| MemberProfitability {
                member_id: assignment.member_id,
                planned_minutes: 0,
                labour_cost_cents: 0,
                missing_cost: None,
            });

        total.planned_minutes += minutes;

        match cost {
            Ok(cost_cents) => total.labour_cost_cents += cost_cents,
            // Kept if already set: one gap named is enough to act on, and the
            // first one found is as good as any.
            Err(missing) => total.missing_cost = total.missing_cost.or(Some(*missing)),
        }
    }

    let mut members: Vec<MemberProfitability> = totals.into_values().collect();
    members.sort_by_key(|member| member.member_id.0);

    members
}

fn cost_of(minutes: i64, hourly_rate_cents: i64) -> i64 {
    div_round_half_even(minutes * hourly_rate_cents, 60)
}

/// Divides, rounding a half to the even neighbour.
///
/// Both arguments are non-negative here (minutes and rates cannot be negative),
/// so no sign handling: a negative would be a bug upstream rather than
/// something to interpret.
fn div_round_half_even(numerator: i64, denominator: i64) -> i64 {
    let quotient = numerator / denominator;
    let doubled_remainder = (numerator % denominator) * 2;

    if doubled_remainder > denominator {
        return quotient + 1;
    }
    if doubled_remainder < denominator {
        return quotient;
    }

    if quotient % 2 == 0 {
        quotient
    } else {
        quotient + 1
    }
}

#[cfg(test)]
mod tests {
    use chrono::NaiveDate;
    use uuid::Uuid;

    use super::*;
    use crate::{
        CustomerId, EmployeeCostBasisId, EmployeeId, EmployeeRhythmId, EquipmentId, MissingCost,
        ProjectId, RhythmSlot, RhythmSlotId, TaskId, WorkSlotId,
    };

    const PARIS: &str = "Europe/Paris";

    fn timezone() -> Tz {
        PARIS.parse().expect("Europe/Paris is an IANA zone")
    }

    /// Monday 2026-06-01, 09:00 Paris, as a UTC instant.
    fn monday(hour: u32, minute: u32) -> DateTime<Utc> {
        NaiveDate::from_ymd_opt(2026, 6, 1)
            .and_then(|date| date.and_hms_opt(hour, minute, 0))
            .map(|naive| {
                timezone()
                    .from_local_datetime(&naive)
                    .earliest()
                    .expect("a valid local time")
                    .with_timezone(&Utc)
            })
            .expect("a representable instant")
    }

    fn june() -> ReportPeriod {
        ReportPeriod {
            from: monday(0, 0),
            to: monday(0, 0) + Duration::days(7),
            timezone: timezone(),
        }
    }

    fn project(quoted_cents: Option<i32>, customer: bool) -> ProjectHeader {
        ProjectHeader {
            project_id: ProjectId(Uuid::from_u128(1)),
            name: "Entretien".to_owned(),
            customer_id: customer.then(|| CustomerId(Uuid::from_u128(9))),
            quoted_cents,
        }
    }

    fn assignment(member: u128, rate: Option<i32>) -> PlannedAssignment {
        PlannedAssignment {
            project_id: Some(ProjectId(Uuid::from_u128(1))),
            task_id: TaskId(Uuid::from_u128(100)),
            member_id: MemberId(Uuid::from_u128(member)),
            employee_id: Some(EmployeeId(Uuid::from_u128(member + 1000))),
            hourly_rate_cents: rate,
            is_salaried: false,
            monthly_cost_cents: None,
            weekly_contract_minutes: 2_100,
            starts_at: monday(9, 0),
            ends_at: monday(11, 0),
            all_day: false,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn cost_basis(
        employee_id: EmployeeId,
        effective_from: NaiveDate,
        effective_to: Option<NaiveDate>,
        is_salaried: bool,
        hourly_rate_cents: Option<i32>,
        monthly_cost_cents: Option<i32>,
        weekly_contract_minutes: i32,
    ) -> EmployeeCostBasis {
        let now = Utc::now();
        EmployeeCostBasis {
            id: EmployeeCostBasisId(Uuid::new_v4()),
            organization_id: crate::OrganizationId(Uuid::from_u128(50)),
            employee_id,
            effective_from,
            effective_to,
            is_salaried,
            hourly_rate_cents,
            monthly_cost_cents,
            weekly_contract_minutes,
            created_at: now,
            updated_at: now,
        }
    }

    fn facts(
        assignments: Vec<PlannedAssignment>,
        projects: Vec<ProjectHeader>,
    ) -> ProfitabilityFacts {
        ProfitabilityFacts {
            projects,
            assignments,
            expenses: Vec::new(),
            equipment: Vec::new(),
            cost_bases: Vec::new(),
            supplier_costs: Vec::new(),
            // Irrelevant while `supplier_costs` is empty; the dedicated
            // supplier-cost tests below set both explicitly.
            organization_vat_subject: true,
        }
    }

    fn only_project(report: &ProfitabilityReport) -> &ProjectProfitability {
        report.projects.first().expect("one project")
    }

    /// The example the whole change exists for: a two-hour meeting with three
    /// people costs six person-hours.
    #[test]
    fn three_people_for_two_hours_cost_six_person_hours() {
        let assignments = vec![
            assignment(1, Some(3_000)),
            assignment(2, Some(3_000)),
            assignment(3, Some(3_000)),
        ];

        let report = build_report(
            facts(assignments, vec![project(None, false)]),
            WorkTime::default(),
            june(),
        );
        let project = only_project(&report);

        assert_eq!(project.planned_minutes, 360);
        assert_eq!(project.labour_cost_cents, 18_000);
        assert_eq!(
            project.occupied_minutes, 120,
            "three people over the same two hours occupy two hours"
        );
        assert_eq!(
            project.overlapping_minutes, 0,
            "different people sharing a slot is teamwork, not a double booking"
        );
    }

    /// 3 500 € a month on a 35 h contract is 23,08 € an hour, so two hours cost
    /// 46,16 €. This used to be 0,00 €, and nothing on the screen said so.
    #[test]
    fn a_salaried_person_costs_their_derived_hourly_rate() {
        let mut salaried = assignment(2, None);
        salaried.is_salaried = true;
        salaried.monthly_cost_cents = Some(350_000);

        let report = build_report(
            facts(
                vec![assignment(1, Some(3_000)), salaried],
                vec![project(Some(100_000), true)],
            ),
            WorkTime::default(),
            june(),
        );
        let project = only_project(&report);

        assert_eq!(project.planned_minutes, 240);
        assert_eq!(project.labour_cost_cents, 6_000 + 4_616);
        assert!(
            project.members_without_rate.is_empty(),
            "a salary is a cost basis, not a missing one"
        );
        assert_eq!(project.margin_cents, Some(100_000 - 6_000 - 4_616));
    }

    /// The regression this fixes. A salaried person with no amount entered was
    /// costed at zero and deliberately kept out of `members_without_rate`, so an
    /// hour of their time read as free and the margin was stated as fact.
    #[test]
    fn a_salaried_person_with_no_salary_is_a_missing_cost_not_a_free_one() {
        let mut salaried = assignment(2, None);
        salaried.is_salaried = true;

        let report = build_report(
            facts(
                vec![assignment(1, Some(3_000)), salaried],
                vec![project(Some(100_000), true)],
            ),
            WorkTime::default(),
            june(),
        );
        let project = only_project(&report);

        assert_eq!(
            project.members_without_rate,
            vec![MemberId(Uuid::from_u128(2))]
        );
        assert_eq!(project.margin_cents, None);
        assert_eq!(
            project.planned_minutes, 240,
            "their time is still planned, it is only the cost that is unknown"
        );

        let salaried_total = report
            .members
            .iter()
            .find(|member| member.member_id == MemberId(Uuid::from_u128(2)))
            .expect("the person appears");
        assert_eq!(
            salaried_total.missing_cost,
            Some(MissingCost::MonthlyCost),
            "the screen has to name the amount, not guess between two fields"
        );
    }

    /// A salary with no contracted hours has no hourly equivalent, so it lands in
    /// the same place as an amount nobody entered.
    #[test]
    fn a_salary_with_no_contract_cannot_be_costed_either() {
        let mut salaried = assignment(2, None);
        salaried.is_salaried = true;
        salaried.monthly_cost_cents = Some(350_000);
        salaried.weekly_contract_minutes = 0;

        let report = build_report(
            facts(vec![salaried], vec![project(Some(100_000), true)]),
            WorkTime::default(),
            june(),
        );

        assert_eq!(
            only_project(&report).members_without_rate,
            vec![MemberId(Uuid::from_u128(2))]
        );
        // The gap the report used to describe as a missing salary, which sent
        // somebody looking at a field they had already filled in.
        assert_eq!(
            report.members[0].missing_cost,
            Some(MissingCost::ContractedHours)
        );
    }

    /// The per-person section and the project total read the same rule, so they
    /// cannot disagree about what a salaried person costs.
    #[test]
    fn a_salaried_person_costs_the_same_in_both_totals() {
        let mut salaried = assignment(2, None);
        salaried.is_salaried = true;
        salaried.monthly_cost_cents = Some(350_000);

        let report = build_report(
            facts(vec![salaried], vec![project(None, false)]),
            WorkTime::default(),
            june(),
        );

        let member = report.members.first().expect("one person");
        assert_eq!(member.labour_cost_cents, 4_616);
        assert_eq!(member.missing_cost, None);
        assert_eq!(only_project(&report).labour_cost_cents, 4_616);
    }

    #[test]
    fn a_missing_rate_withholds_the_margin_and_names_the_person() {
        let report = build_report(
            facts(
                vec![assignment(1, Some(3_000)), assignment(2, None)],
                vec![project(Some(100_000), true)],
            ),
            WorkTime::default(),
            june(),
        );
        let project = only_project(&report);

        assert_eq!(
            project.members_without_rate,
            vec![MemberId(Uuid::from_u128(2))]
        );
        assert!(!project.is_complete());
        assert_eq!(
            project.margin_cents, None,
            "a margin built on a floor reads as fact and is not one"
        );
    }

    #[test]
    fn one_person_booked_twice_over_is_billed_twice_and_reported() {
        let first = assignment(1, Some(3_000));
        let mut second = assignment(1, Some(3_000));
        second.task_id = TaskId(Uuid::from_u128(101));
        second.starts_at = monday(10, 0);
        second.ends_at = monday(12, 0);

        let report = build_report(
            facts(vec![first, second], vec![project(None, false)]),
            WorkTime::default(),
            june(),
        );
        let project = only_project(&report);

        assert_eq!(project.planned_minutes, 240, "both bookings are billed");
        assert_eq!(project.occupied_minutes, 180, "09:00 to 12:00 in the round");
        assert_eq!(
            project.overlapping_minutes, 60,
            "the hour billed twice is surfaced rather than silently collapsed"
        );
        assert!(
            project.is_complete(),
            "an overlap is a planning problem, not missing data"
        );
    }

    #[test]
    fn an_all_day_task_costs_the_work_slots_and_never_the_lunch_break() {
        let employee_id = EmployeeId(Uuid::from_u128(1001));
        let rhythm_id = EmployeeRhythmId(Uuid::from_u128(7));

        let mut all_day = assignment(1, Some(3_000));
        all_day.all_day = true;
        all_day.starts_at = monday(0, 0);
        all_day.ends_at = monday(0, 0) + Duration::days(1);

        // 09:00-12:00 then 13:00-17:00 on a Monday: seven hours worked, one hour
        // of lunch that nobody bills.
        let work_time = WorkTime {
            rhythms: vec![EmployeeRhythm {
                id: rhythm_id,
                organization_id: crate::OrganizationId(Uuid::from_u128(50)),
                employee_id,
                effective_from: NaiveDate::from_ymd_opt(2026, 1, 1).expect("a date"),
                effective_to: None,
                slots: vec![
                    RhythmSlot {
                        id: RhythmSlotId(Uuid::from_u128(11)),
                        rhythm_id,
                        weekday: 1,
                        starts_minute: 9 * 60,
                        ends_minute: 12 * 60,
                    },
                    RhythmSlot {
                        id: RhythmSlotId(Uuid::from_u128(12)),
                        rhythm_id,
                        weekday: 1,
                        starts_minute: 13 * 60,
                        ends_minute: 17 * 60,
                    },
                ],
                created_at: monday(0, 0),
                updated_at: monday(0, 0),
            }],
            work_slots: Vec::new(),
        };

        let mut all_day_facts = facts(vec![all_day], vec![project(None, false)]);
        all_day_facts.cost_bases = vec![cost_basis(
            employee_id,
            NaiveDate::from_ymd_opt(2026, 1, 1).expect("a date"),
            None,
            false,
            Some(3_000),
            None,
            2_100,
        )];

        let report = build_report(all_day_facts, work_time, june());
        let project = only_project(&report);

        assert_eq!(
            project.planned_minutes, 420,
            "seven hours, not the eight a naive amplitude would give"
        );
        assert_eq!(project.labour_cost_cents, 21_000);
    }

    /// A dated slot beats the rhythm for that day, which is what makes a
    /// one-off schedule change costable.
    #[test]
    fn a_dated_work_slot_overrides_the_rhythm_for_that_day() {
        let mut all_day = assignment(1, Some(3_000));
        all_day.all_day = true;
        all_day.starts_at = monday(0, 0);
        all_day.ends_at = monday(0, 0) + Duration::days(1);

        let work_time = WorkTime {
            rhythms: Vec::new(),
            work_slots: vec![WorkSlot {
                id: WorkSlotId(Uuid::from_u128(21)),
                organization_id: crate::OrganizationId(Uuid::from_u128(50)),
                member_id: MemberId(Uuid::from_u128(1)),
                work_date: NaiveDate::from_ymd_opt(2026, 6, 1).expect("a date"),
                starts_minute: 8 * 60,
                ends_minute: 10 * 60,
            }],
        };

        let report = build_report(
            facts(vec![all_day], vec![project(None, false)]),
            work_time,
            june(),
        );

        assert_eq!(only_project(&report).planned_minutes, 120);
    }

    #[test]
    fn an_all_day_task_for_someone_with_no_schedule_costs_nothing() {
        let mut all_day = assignment(1, Some(3_000));
        all_day.all_day = true;
        all_day.starts_at = monday(0, 0);
        all_day.ends_at = monday(0, 0) + Duration::days(1);

        let report = build_report(
            facts(vec![all_day], vec![project(None, false)]),
            WorkTime::default(),
            june(),
        );
        let project = only_project(&report);

        assert_eq!(
            project.planned_minutes, 0,
            "nothing is planned against a week this person does not have"
        );
        assert_eq!(project.labour_cost_cents, 0);
    }

    #[test]
    fn a_task_straddling_the_period_boundary_contributes_only_what_is_inside() {
        let mut straddling = assignment(1, Some(3_000));
        straddling.starts_at = monday(0, 0) - Duration::hours(2);
        straddling.ends_at = monday(0, 0) + Duration::hours(3);

        let report = build_report(
            facts(vec![straddling], vec![project(None, false)]),
            WorkTime::default(),
            june(),
        );

        assert_eq!(
            only_project(&report).planned_minutes,
            180,
            "the two hours before the period belong to the period before it"
        );
    }

    #[test]
    fn an_internal_project_reports_a_cost_and_no_margin() {
        let report = build_report(
            facts(vec![assignment(1, Some(3_000))], vec![project(None, false)]),
            WorkTime::default(),
            june(),
        );
        let project = only_project(&report);

        assert_eq!(project.customer_id, None);
        assert_eq!(project.labour_cost_cents, 6_000);
        assert_eq!(project.quoted_cents, None);
        assert_eq!(project.margin_cents, None);
        assert!(
            project.is_complete(),
            "having no quote is not incomplete data"
        );
    }

    #[test]
    fn expenses_and_equipment_join_the_cost_and_the_margin() {
        let mut facts = facts(
            vec![assignment(1, Some(3_000))],
            vec![project(Some(100_000), true)],
        );
        facts.expenses = vec![TaskExpense {
            project_id: ProjectId(Uuid::from_u128(1)),
            task_id: TaskId(Uuid::from_u128(100)),
            expenses_cents: 4_500,
            starts_at: monday(9, 0),
        }];
        facts.equipment = vec![AssignedEquipment {
            project_id: ProjectId(Uuid::from_u128(1)),
            equipment_id: EquipmentId(Uuid::from_u128(30)),
            hourly_rate_cents: 1_200,
        }];

        let report = build_report(facts, WorkTime::default(), june());
        let project = only_project(&report);

        assert_eq!(project.expenses_cents, 4_500);
        // The machine ran as long as somebody was there: two hours at 12 euros.
        assert_eq!(project.equipment_cost_cents, 2_400);
        assert_eq!(project.planned_cost_cents(), 6_000 + 2_400 + 4_500);
        assert_eq!(project.margin_cents, Some(100_000 - 6_000 - 2_400 - 4_500));
    }

    #[test]
    fn an_expense_on_a_task_starting_outside_the_period_is_left_out() {
        let mut facts = facts(
            vec![assignment(1, Some(3_000))],
            vec![project(Some(100_000), true)],
        );
        facts.expenses = vec![TaskExpense {
            project_id: ProjectId(Uuid::from_u128(1)),
            task_id: TaskId(Uuid::from_u128(100)),
            expenses_cents: 4_500,
            starts_at: monday(0, 0) - Duration::days(1),
        }];

        let report = build_report(facts, WorkTime::default(), june());

        assert_eq!(only_project(&report).expenses_cents, 0);
    }

    /// A task attached to no project still costs the person's time. Payroll
    /// cares about the hours whether or not somebody filed them under a subject.
    #[test]
    fn a_task_with_no_project_counts_for_the_person_and_for_nobody_else() {
        let mut orphan = assignment(1, Some(3_000));
        orphan.project_id = None;

        let report = build_report(
            facts(
                vec![assignment(2, Some(3_000)), orphan],
                vec![project(None, false)],
            ),
            WorkTime::default(),
            june(),
        );

        assert_eq!(
            only_project(&report).planned_minutes,
            120,
            "only the assignment naming the project is charged to it"
        );
        let orphan_member = report
            .members
            .iter()
            .find(|member| member.member_id == MemberId(Uuid::from_u128(1)))
            .expect("the person still appears");
        assert_eq!(orphan_member.planned_minutes, 120);
        assert_eq!(orphan_member.labour_cost_cents, 6_000);
    }

    #[test]
    fn member_totals_are_ordered_by_id_so_the_answer_is_stable() {
        let report = build_report(
            facts(
                vec![assignment(3, Some(3_000)), assignment(1, None)],
                vec![project(None, false)],
            ),
            WorkTime::default(),
            june(),
        );

        let ids: Vec<Uuid> = report
            .members
            .iter()
            .map(|member| member.member_id.0)
            .collect();
        assert_eq!(ids, vec![Uuid::from_u128(1), Uuid::from_u128(3)]);
        assert_eq!(
            report.members[0].missing_cost,
            Some(MissingCost::HourlyRate)
        );
        assert_eq!(report.members[1].missing_cost, None);
    }

    // -- Per-day costing against `employee_cost_bases` ---------------------

    fn all_day_rhythm(employee_id: EmployeeId, weekdays: &[i16]) -> WorkTime {
        let rhythm_id = EmployeeRhythmId(Uuid::from_u128(7));
        WorkTime {
            rhythms: vec![EmployeeRhythm {
                id: rhythm_id,
                organization_id: crate::OrganizationId(Uuid::from_u128(50)),
                employee_id,
                effective_from: NaiveDate::from_ymd_opt(2026, 1, 1).expect("a date"),
                effective_to: None,
                slots: weekdays
                    .iter()
                    .map(|&weekday| RhythmSlot {
                        id: RhythmSlotId(Uuid::new_v4()),
                        rhythm_id,
                        weekday,
                        starts_minute: 9 * 60,
                        ends_minute: 17 * 60,
                    })
                    .collect(),
                created_at: monday(0, 0),
                updated_at: monday(0, 0),
            }],
            work_slots: Vec::new(),
        }
    }

    /// The bug this issue exists to fix, at the all-day boundary: a raise
    /// taking effect partway through a two-day all-day task must cost the
    /// first day at the old rate and the second at the new one, never one
    /// rate blended or wrongly applied to both.
    #[test]
    fn an_all_day_task_straddling_a_rate_change_costs_each_day_at_its_own_rate() {
        let employee_id = EmployeeId(Uuid::from_u128(1001));
        let raise_day = NaiveDate::from_ymd_opt(2026, 6, 2).expect("a date");

        let mut all_day = assignment(1, None);
        all_day.all_day = true;
        all_day.starts_at = monday(0, 0);
        all_day.ends_at = monday(0, 0) + Duration::days(2); // Monday and Tuesday.

        let mut facts = facts(vec![all_day], vec![project(None, false)]);
        facts.cost_bases = vec![
            cost_basis(
                employee_id,
                NaiveDate::from_ymd_opt(2026, 1, 1).expect("a date"),
                Some(raise_day),
                false,
                Some(3_000),
                None,
                2_100,
            ),
            cost_basis(
                employee_id,
                raise_day,
                None,
                false,
                Some(4_000),
                None,
                2_100,
            ),
        ];

        let report = build_report(facts, all_day_rhythm(employee_id, &[1, 2]), june());
        let project = only_project(&report);

        // Monday: 8h at 30,00 €/h = 240,00 €. Tuesday: 8h at 40,00 €/h = 320,00 €.
        assert_eq!(project.labour_cost_cents, 24_000 + 32_000);
        assert!(
            project.members_without_rate.is_empty(),
            "both days had a version to cost from"
        );
    }

    /// A task planned before the employee's very first cost basis version
    /// finds no version to read on that day, which is a different gap from a
    /// version that exists with a blank figure — the reader has to be sent to
    /// a different place for each.
    #[test]
    fn an_all_day_task_on_a_date_no_version_covers_is_a_distinct_missing_cost() {
        let employee_id = EmployeeId(Uuid::from_u128(1001));

        let mut all_day = assignment(1, None);
        all_day.all_day = true;
        all_day.starts_at = monday(0, 0);
        all_day.ends_at = monday(0, 0) + Duration::days(1);

        let mut facts = facts(vec![all_day], vec![project(None, false)]);
        // The only version on file starts after the task's own day.
        facts.cost_bases = vec![cost_basis(
            employee_id,
            NaiveDate::from_ymd_opt(2026, 12, 1).expect("a date"),
            None,
            false,
            Some(3_000),
            None,
            2_100,
        )];

        let report = build_report(facts, all_day_rhythm(employee_id, &[1]), june());

        assert_eq!(
            report.members[0].missing_cost,
            Some(MissingCost::NoCostBasis),
            "conflating this with `HourlyRate` would send somebody to check a \
             figure that was never the problem"
        );
    }

    // -- Supplier cost (#338) -----------------------------------------------

    #[test]
    fn supplier_cost_is_net_when_the_organization_can_recover_vat() {
        let mut facts = facts(vec![], vec![project(Some(100_000), true)]);
        facts.organization_vat_subject = true;
        facts.supplier_costs = vec![SupplierCostAllocation {
            project_id: ProjectId(Uuid::from_u128(1)),
            net_amount_cents: 20_000,
            vat_rate_basis_points: Some(2000),
        }];

        let report = build_report(facts, WorkTime::default(), june());
        let project = only_project(&report);

        assert_eq!(
            project.supplier_cost_cents, 20_000,
            "a subject organization recovers the VAT, so only the net price is a real cost"
        );
        assert_eq!(project.margin_cents, Some(100_000 - 20_000));
    }

    #[test]
    fn supplier_cost_is_grossed_up_when_the_organization_cannot_recover_vat() {
        let mut facts = facts(vec![], vec![project(Some(100_000), true)]);
        facts.organization_vat_subject = false;
        facts.supplier_costs = vec![SupplierCostAllocation {
            project_id: ProjectId(Uuid::from_u128(1)),
            net_amount_cents: 20_000,
            vat_rate_basis_points: Some(2000),
        }];

        let report = build_report(facts, WorkTime::default(), june());
        let project = only_project(&report);

        // 20 000 net + 20% VAT that is never coming back = 24 000.
        assert_eq!(
            project.supplier_cost_cents, 24_000,
            "VAT paid and never recovered is exactly as much a cost as the net price"
        );
        assert_eq!(project.margin_cents, Some(100_000 - 24_000));
        assert_eq!(project.planned_cost_cents(), 24_000);
    }

    #[test]
    fn a_rateless_supplier_line_costs_the_same_net_or_gross() {
        let mut facts = facts(vec![], vec![project(None, false)]);
        facts.organization_vat_subject = false;
        facts.supplier_costs = vec![SupplierCostAllocation {
            project_id: ProjectId(Uuid::from_u128(1)),
            net_amount_cents: 5_000,
            vat_rate_basis_points: None,
        }];

        let report = build_report(facts, WorkTime::default(), june());

        assert_eq!(only_project(&report).supplier_cost_cents, 5_000);
    }

    /// A credit/rebate line allocated as a credit must gross up towards a
    /// larger *debt* to the organization, never flip sign into a positive
    /// cost.
    #[test]
    fn a_credit_allocation_grosses_up_without_flipping_sign() {
        let mut facts = facts(vec![], vec![project(None, false)]);
        facts.organization_vat_subject = false;
        facts.supplier_costs = vec![SupplierCostAllocation {
            project_id: ProjectId(Uuid::from_u128(1)),
            net_amount_cents: -10_000,
            vat_rate_basis_points: Some(2000),
        }];

        let report = build_report(facts, WorkTime::default(), june());

        assert_eq!(only_project(&report).supplier_cost_cents, -12_000);
    }

    #[test]
    fn supplier_cost_on_another_project_does_not_leak_in() {
        let mut facts = facts(vec![], vec![project(None, false)]);
        facts.organization_vat_subject = true;
        facts.supplier_costs = vec![SupplierCostAllocation {
            project_id: ProjectId(Uuid::from_u128(999)),
            net_amount_cents: 20_000,
            vat_rate_basis_points: None,
        }];

        let report = build_report(facts, WorkTime::default(), june());

        assert_eq!(only_project(&report).supplier_cost_cents, 0);
    }
}
