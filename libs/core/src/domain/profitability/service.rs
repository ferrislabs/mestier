use std::collections::BTreeMap;

use chrono::{DateTime, Utc};
use common::CoreError;

use crate::{
    EmployeeId, EmployeeProfitability, OrganizationId, ProfitabilityReport, TaskProfitability,
    domain::profitability::{
        AssignedEquipment, ClockedTime, JobHeader, ProfitabilityFacts,
        ports::ProfitabilityRepository,
    },
};

pub struct ProfitabilityService<R>
where
    R: ProfitabilityRepository,
{
    repo: R,
}

impl<R> ProfitabilityService<R>
where
    R: ProfitabilityRepository,
{
    pub fn new(repo: R) -> Self {
        Self { repo }
    }

    pub async fn report(
        &mut self,
        organization_id: OrganizationId,
        from: DateTime<Utc>,
        to: DateTime<Utc>,
    ) -> Result<ProfitabilityReport, CoreError> {
        let facts = self.repo.load(organization_id, from, to).await?;

        Ok(build_report(facts))
    }
}

/// The whole calculation, pure and separately testable.
pub fn build_report(facts: ProfitabilityFacts) -> ProfitabilityReport {
    let ProfitabilityFacts {
        jobs,
        clocked,
        equipment,
    } = facts;

    ProfitabilityReport {
        jobs: jobs
            .iter()
            .map(|job| job_profitability(job, &clocked, &equipment))
            .collect(),
        employees: employee_profitability(&clocked),
    }
}

fn job_profitability(
    job: &JobHeader,
    clocked: &[ClockedTime],
    equipment: &[AssignedEquipment],
) -> TaskProfitability {
    let entries: Vec<&ClockedTime> = clocked
        .iter()
        .filter(|entry| entry.task_id == job.task_id)
        .collect();

    let mut labour_cost_cents = 0_i64;
    let mut worked_minutes = 0_i64;
    let mut recollected_minutes = 0_i64;
    let mut open_entries = 0_u32;
    let mut employees_without_rate: Vec<EmployeeId> = Vec::new();

    for entry in &entries {
        let Some(minutes) = closed_minutes(entry) else {
            open_entries += 1;
            continue;
        };

        worked_minutes += minutes;
        if entry.closed_after_the_fact {
            recollected_minutes += minutes;
        }
        match entry.hourly_rate_cents {
            Some(rate) => labour_cost_cents += cost_of(minutes, i64::from(rate)),
            // Recorded once per employee, not once per entry: the reader needs
            // to know who to go and set a rate for, not how many times it hurt.
            None if !employees_without_rate.contains(&entry.employee_id) => {
                employees_without_rate.push(entry.employee_id);
            }
            None => {}
        }
    }

    // The machines ran for as long as somebody was on site, which is the union
    // of the clocked stretches rather than their sum: two people working the
    // same hour is one hour of mower, not two.
    let occupied_minutes = occupied_minutes(&entries);
    let equipment_rate_cents: i64 = equipment
        .iter()
        .filter(|item| item.task_id == job.task_id)
        .map(|item| i64::from(item.hourly_rate_cents))
        .sum();
    let equipment_cost_cents = cost_of(occupied_minutes, equipment_rate_cents);

    // Withheld while anything is missing. A margin built on a floor reads as
    // fact and is not one, and this screen exists to be trusted.
    let is_complete = employees_without_rate.is_empty() && open_entries == 0;
    let margin_cents = job
        .quoted_cents
        .filter(|_| is_complete)
        .map(|quoted| i64::from(quoted) - (labour_cost_cents + equipment_cost_cents));

    TaskProfitability {
        task_id: job.task_id,
        title: job.title.clone(),
        customer_id: job.customer_id,
        quoted_cents: job.quoted_cents,
        labour_cost_cents,
        equipment_cost_cents,
        worked_minutes,
        occupied_minutes,
        margin_cents,
        employees_without_rate,
        open_entries,
        recollected_minutes,
    }
}

/// Per-employee totals across every job in the period.
///
/// Ordered by id so the answer is stable: a ranking that reshuffles between two
/// identical requests is a ranking nobody believes.
fn employee_profitability(clocked: &[ClockedTime]) -> Vec<EmployeeProfitability> {
    let mut by_employee: BTreeMap<uuid::Uuid, EmployeeProfitability> = BTreeMap::new();

    for entry in clocked {
        let row = by_employee
            .entry(entry.employee_id.0)
            .or_insert_with(|| EmployeeProfitability {
                employee_id: entry.employee_id,
                worked_minutes: 0,
                labour_cost_cents: 0,
                rate_missing: entry.hourly_rate_cents.is_none(),
                open_entries: 0,
            });

        row.rate_missing = row.rate_missing || entry.hourly_rate_cents.is_none();

        match closed_minutes(entry) {
            Some(minutes) => {
                row.worked_minutes += minutes;
                if let Some(rate) = entry.hourly_rate_cents {
                    row.labour_cost_cents += cost_of(minutes, i64::from(rate));
                }
            }
            None => row.open_entries += 1,
        }
    }

    by_employee.into_values().collect()
}

/// Minutes of a closed entry, or `None` while it is still running.
///
/// Truncating, as `TimeEntry::worked_minutes` does: a cost must never claim
/// more time than was recorded.
fn closed_minutes(entry: &ClockedTime) -> Option<i64> {
    entry
        .ended_at
        .map(|ended_at| (ended_at - entry.started_at).num_minutes().max(0))
}

/// Wall-clock minutes covered by at least one entry.
///
/// Overlaps merged rather than summed, which is what makes this the time the
/// site was occupied rather than the man-hours spent on it.
fn occupied_minutes(entries: &[&ClockedTime]) -> i64 {
    let mut spans: Vec<(DateTime<Utc>, DateTime<Utc>)> = entries
        .iter()
        .filter_map(|entry| entry.ended_at.map(|ended_at| (entry.started_at, ended_at)))
        .filter(|(start, end)| end > start)
        .collect();
    spans.sort_by_key(|(start, _)| *start);

    let mut total = 0_i64;
    let mut current: Option<(DateTime<Utc>, DateTime<Utc>)> = None;

    for (start, end) in spans {
        match current {
            Some((open_start, open_end)) if start <= open_end => {
                current = Some((open_start, open_end.max(end)));
            }
            Some((open_start, open_end)) => {
                total += (open_end - open_start).num_minutes();
                current = Some((start, end));
            }
            None => current = Some((start, end)),
        }
    }

    if let Some((open_start, open_end)) = current {
        total += (open_end - open_start).num_minutes();
    }

    total
}

/// `minutes × rate_per_hour`, in cents.
///
/// Exact integers, halves to even, the same rule the quote total uses. Seven
/// minutes at 35 €/h is 408.33 cents, and a product where every screen rounds
/// differently is a product whose figures never quite add up.
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
    use chrono::TimeZone;
    use uuid::Uuid;

    use super::*;
    use crate::{CustomerId, EquipmentId, TaskId};

    fn at(hour: u32, minute: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 8, 20, hour, minute, 0)
            .single()
            .expect("a valid test instant")
    }

    fn job(quoted_cents: Option<i32>) -> JobHeader {
        JobHeader {
            task_id: TaskId(Uuid::new_v4()),
            title: "Jardin Duval".to_owned(),
            customer_id: CustomerId(Uuid::new_v4()),
            quoted_cents,
        }
    }

    fn clocked(
        task_id: TaskId,
        employee_id: EmployeeId,
        rate: Option<i32>,
        from: (u32, u32),
        to: Option<(u32, u32)>,
    ) -> ClockedTime {
        ClockedTime {
            task_id,
            employee_id,
            hourly_rate_cents: rate,
            started_at: at(from.0, from.1),
            ended_at: to.map(|(h, m)| at(h, m)),
            closed_after_the_fact: false,
        }
    }

    /// A stretch recovered the next day: closed on a declared end time rather
    /// than a measured one.
    fn recollected(
        task_id: TaskId,
        employee_id: EmployeeId,
        rate: Option<i32>,
        from: (u32, u32),
        to: (u32, u32),
    ) -> ClockedTime {
        ClockedTime {
            closed_after_the_fact: true,
            ..clocked(task_id, employee_id, rate, from, Some(to))
        }
    }

    fn employee() -> EmployeeId {
        EmployeeId(Uuid::new_v4())
    }

    #[test]
    fn labour_cost_is_time_times_the_rate_that_applied() {
        let job = job(None);
        let facts = ProfitabilityFacts {
            jobs: vec![job.clone()],
            // 3 h at 35 €/h, then 2 h at 28 €/h.
            clocked: vec![
                clocked(job.task_id, employee(), Some(3500), (8, 0), Some((11, 0))),
                clocked(job.task_id, employee(), Some(2800), (13, 0), Some((15, 0))),
            ],
            equipment: vec![],
        };

        let report = build_report(facts);
        let result = &report.jobs[0];

        assert_eq!(result.worked_minutes, 300);
        assert_eq!(result.labour_cost_cents, 3500 * 3 + 2800 * 2);
        assert_eq!(result.equipment_cost_cents, 0);
    }

    /// The reason equipment time is a union rather than a sum. Two people on the
    /// same hour is one hour of mower, and summing would bill it twice.
    #[test]
    fn equipment_is_charged_for_the_time_the_site_was_occupied_not_the_man_hours() {
        let job = job(None);
        let facts = ProfitabilityFacts {
            jobs: vec![job.clone()],
            clocked: vec![
                clocked(job.task_id, employee(), Some(0), (9, 0), Some((12, 0))),
                clocked(job.task_id, employee(), Some(0), (10, 0), Some((11, 0))),
            ],
            equipment: vec![AssignedEquipment {
                task_id: job.task_id,
                equipment_id: EquipmentId(Uuid::new_v4()),
                hourly_rate_cents: 1200,
            }],
        };

        let result = &build_report(facts).jobs[0];

        assert_eq!(result.worked_minutes, 240, "four man-hours were worked");
        assert_eq!(
            result.occupied_minutes, 180,
            "but the site was busy for three"
        );
        assert_eq!(result.equipment_cost_cents, 3600, "three hours of mower");
    }

    #[test]
    fn two_separate_stretches_are_both_counted() {
        let job = job(None);
        let worker = employee();
        let facts = ProfitabilityFacts {
            jobs: vec![job.clone()],
            clocked: vec![
                clocked(job.task_id, worker, Some(0), (8, 0), Some((10, 0))),
                clocked(job.task_id, worker, Some(0), (14, 0), Some((16, 0))),
            ],
            equipment: vec![],
        };

        assert_eq!(build_report(facts).jobs[0].occupied_minutes, 240);
    }

    #[test]
    fn margin_is_the_quote_less_what_it_cost() {
        let job = job(Some(420_000));
        let facts = ProfitabilityFacts {
            jobs: vec![job.clone()],
            clocked: vec![clocked(
                job.task_id,
                employee(),
                Some(3500),
                (8, 0),
                Some((13, 0)),
            )],
            equipment: vec![AssignedEquipment {
                task_id: job.task_id,
                equipment_id: EquipmentId(Uuid::new_v4()),
                hourly_rate_cents: 1200,
            }],
        };

        let result = &build_report(facts).jobs[0];

        assert_eq!(result.labour_cost_cents, 17_500);
        assert_eq!(result.equipment_cost_cents, 6_000);
        assert_eq!(result.real_cost_cents(), 23_500);
        assert_eq!(result.margin_cents, Some(420_000 - 23_500));
    }

    /// The invariant `Employee::hourly_rate_cents` documents: a cost must refuse
    /// to produce a figure for someone whose rate was never entered, rather
    /// than silently sum it as zero.
    #[test]
    fn an_employee_without_a_rate_makes_the_cost_a_floor_and_withholds_the_margin() {
        let job = job(Some(420_000));
        let unrated = employee();
        let facts = ProfitabilityFacts {
            jobs: vec![job.clone()],
            clocked: vec![
                clocked(job.task_id, employee(), Some(3500), (8, 0), Some((10, 0))),
                clocked(job.task_id, unrated, None, (8, 0), Some((10, 0))),
            ],
            equipment: vec![],
        };

        let result = &build_report(facts).jobs[0];

        assert_eq!(
            result.labour_cost_cents, 7_000,
            "only the rated hours count"
        );
        assert_eq!(result.employees_without_rate, vec![unrated]);
        assert_eq!(
            result.margin_cents, None,
            "a margin built on a floor reads as fact and is not one"
        );
        assert!(!result.is_complete());
    }

    #[test]
    fn the_same_unrated_employee_is_reported_once_however_many_stretches() {
        let job = job(None);
        let unrated = employee();
        let facts = ProfitabilityFacts {
            jobs: vec![job.clone()],
            clocked: vec![
                clocked(job.task_id, unrated, None, (8, 0), Some((10, 0))),
                clocked(job.task_id, unrated, None, (11, 0), Some((12, 0))),
            ],
            equipment: vec![],
        };

        assert_eq!(
            build_report(facts).jobs[0].employees_without_rate,
            vec![unrated]
        );
    }

    /// An entry nobody clocked off has no duration, so it cannot be costed. It
    /// is reported rather than ignored: the figure is missing time, which is not
    /// the same as the time having been free.
    #[test]
    fn an_entry_never_clocked_off_is_reported_rather_than_costed() {
        let job = job(Some(100_000));
        let facts = ProfitabilityFacts {
            jobs: vec![job.clone()],
            clocked: vec![
                clocked(job.task_id, employee(), Some(3500), (8, 0), Some((10, 0))),
                clocked(job.task_id, employee(), Some(3500), (14, 0), None),
            ],
            equipment: vec![],
        };

        let result = &build_report(facts).jobs[0];

        assert_eq!(result.open_entries, 1);
        assert_eq!(result.worked_minutes, 120, "the open stretch adds nothing");
        assert_eq!(result.margin_cents, None);
        assert!(!result.is_complete());
    }

    #[test]
    fn a_job_with_no_quote_has_a_cost_but_no_margin() {
        let job = job(None);
        let facts = ProfitabilityFacts {
            jobs: vec![job.clone()],
            clocked: vec![clocked(
                job.task_id,
                employee(),
                Some(3500),
                (8, 0),
                Some((10, 0)),
            )],
            equipment: vec![],
        };

        let result = &build_report(facts).jobs[0];

        assert_eq!(result.labour_cost_cents, 7_000);
        assert_eq!(result.margin_cents, None);
        assert!(
            result.is_complete(),
            "nothing is missing, there is just no quote"
        );
    }

    #[test]
    fn one_job_never_borrows_another_s_time_or_equipment() {
        let mine = job(None);
        let theirs = job(None);
        let facts = ProfitabilityFacts {
            jobs: vec![mine.clone(), theirs.clone()],
            clocked: vec![clocked(
                theirs.task_id,
                employee(),
                Some(3500),
                (8, 0),
                Some((10, 0)),
            )],
            equipment: vec![AssignedEquipment {
                task_id: theirs.task_id,
                equipment_id: EquipmentId(Uuid::new_v4()),
                hourly_rate_cents: 1200,
            }],
        };

        let report = build_report(facts);
        let mine = report
            .jobs
            .iter()
            .find(|row| row.task_id == mine.task_id)
            .expect("both jobs are reported");

        assert_eq!(mine.labour_cost_cents, 0);
        assert_eq!(mine.equipment_cost_cents, 0);
        assert_eq!(mine.worked_minutes, 0);
    }

    /// Seven minutes at 35 €/h is 408.33 cents. The rule is the one the quote
    /// total uses, so no two screens round the same money differently.
    #[test]
    fn a_part_hour_rounds_the_way_the_quote_total_does() {
        assert_eq!(cost_of(7, 3500), 408);
        assert_eq!(cost_of(1, 3500), 58, "58.33 rounds down");
        assert_eq!(cost_of(1, 3600), 60, "an exact minute needs no rounding");
        // Exact halves land on the even neighbour, both directions.
        assert_eq!(cost_of(1, 30), 0, "0.5 goes to 0");
        assert_eq!(cost_of(3, 30), 2, "1.5 goes to 2");
        assert_eq!(cost_of(5, 30), 2, "2.5 goes to 2");
        assert_eq!(cost_of(7, 30), 4, "3.5 goes to 4");
    }

    #[test]
    fn employee_totals_span_every_job_and_stay_in_a_stable_order() {
        let first = job(None);
        let second = job(None);
        let worker = employee();
        let facts = ProfitabilityFacts {
            jobs: vec![first.clone(), second.clone()],
            clocked: vec![
                clocked(first.task_id, worker, Some(3500), (8, 0), Some((10, 0))),
                clocked(second.task_id, worker, Some(3500), (14, 0), Some((15, 0))),
            ],
            equipment: vec![],
        };

        let report = build_report(facts);

        assert_eq!(report.employees.len(), 1, "one person, two jobs");
        assert_eq!(report.employees[0].worked_minutes, 180);
        assert_eq!(report.employees[0].labour_cost_cents, 3500 * 3);
        assert!(!report.employees[0].rate_missing);
    }

    #[test]
    fn an_employee_is_flagged_when_any_of_their_stretches_has_no_rate() {
        let job = job(None);
        let worker = employee();
        let facts = ProfitabilityFacts {
            jobs: vec![job.clone()],
            clocked: vec![
                clocked(job.task_id, worker, Some(3500), (8, 0), Some((10, 0))),
                clocked(job.task_id, worker, None, (14, 0), Some((15, 0))),
            ],
            equipment: vec![],
        };

        let row = &build_report(facts).employees[0];

        assert!(row.rate_missing);
        assert_eq!(row.labour_cost_cents, 7_000, "only the rated stretch costs");
        assert_eq!(row.worked_minutes, 180, "but both were worked");
    }

    /// A stretch closed on a declared time rather than a measured one still
    /// costs and margins exactly like any other: the KPI is not withheld for
    /// it, only annotated.
    #[test]
    fn recollected_time_counts_normally_but_is_reported_separately() {
        let job = job(Some(100_000));
        let facts = ProfitabilityFacts {
            jobs: vec![job.clone()],
            clocked: vec![
                clocked(job.task_id, employee(), Some(3500), (8, 0), Some((10, 0))),
                recollected(job.task_id, employee(), Some(3500), (14, 0), (15, 0)),
            ],
            equipment: vec![],
        };

        let result = &build_report(facts).jobs[0];

        assert_eq!(result.worked_minutes, 180, "both stretches were worked");
        assert_eq!(
            result.labour_cost_cents,
            3500 * 3,
            "the recollected hour costs exactly as much as a measured one"
        );
        assert_eq!(
            result.recollected_minutes, 60,
            "only the declared stretch is flagged"
        );
        assert_eq!(
            result.margin_cents,
            Some(100_000 - 3500 * 3),
            "recollected time never withholds the margin"
        );
        assert!(
            result.is_complete(),
            "recollection is not the same kind of gap as a missing rate or an open entry"
        );
    }

    #[test]
    fn a_job_with_no_recollected_time_reports_zero() {
        let job = job(None);
        let facts = ProfitabilityFacts {
            jobs: vec![job.clone()],
            clocked: vec![clocked(
                job.task_id,
                employee(),
                Some(3500),
                (8, 0),
                Some((10, 0)),
            )],
            equipment: vec![],
        };

        assert_eq!(build_report(facts).jobs[0].recollected_minutes, 0);
    }

    #[test]
    fn an_empty_period_reports_nothing_rather_than_failing() {
        let report = build_report(ProfitabilityFacts::default());

        assert!(report.jobs.is_empty());
        assert!(report.employees.is_empty());
    }
}
