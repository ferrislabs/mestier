use std::collections::{BTreeMap, HashMap};

use chrono::{DateTime, NaiveDate, TimeZone, Timelike, Utc};
use common::CoreError;

use crate::{
    Absence, DateRange, Employee, EmployeeRhythm, MemberId, MinuteInterval, OrganizationId, Task,
    WorkSlot,
    domain::{
        employee::ports::EmployeeRepository,
        member::ports::MemberRepository,
        organization::ports::OrganizationRepository,
        planning::{
            AvailabilityReport, Conflict, ConflictKind, MemberWorkTime, PlanningEntry,
            PlanningResource, PlanningTask, PlanningView, TimeRange, Tz, ports::PlanningRepository,
        },
        work_time::service::expand_work_slots,
    },
};

// ---------------------------------------------------------------------------
// detect_conflicts — pure, I/O-free. See the planning module design doc:
// this function is the one W4 piece worth a disproportionate amount of
// testing, on par with `expand_work_slots` in W3.
// ---------------------------------------------------------------------------

/// For one resource's window, reports every reason it might be unavailable:
/// an overlapping absence, a candidate slot that is not fully covered by
/// the resource's work-time intervals, or an already-assigned task that
/// overlaps and blocks availability. Never refuses anything — see
/// invariant 1 in the planning module design doc: the API reports, the
/// caller decides.
///
/// `busy` may contain tasks with `blocks_availability = false` (a reminder,
/// say) — they are skipped entirely: only a task that carries
/// `blocks_availability = true` can produce an `OverlappingTask` conflict
/// (invariant 9).
pub fn detect_conflicts(
    window: TimeRange,
    absences: &[Absence],
    work_time: &BTreeMap<NaiveDate, Vec<MinuteInterval>>,
    busy: &[Task],
    tz: Tz,
) -> Vec<Conflict> {
    let mut conflicts = Vec::new();

    for absence in absences {
        if overlaps(
            absence.starts_at,
            absence.ends_at,
            window.starts_at,
            window.ends_at,
        ) {
            conflicts.push(Conflict {
                kind: ConflictKind::Absence {
                    kind: absence.kind,
                    note: absence.note.clone(),
                },
                starts_at: absence.starts_at,
                ends_at: absence.ends_at,
            });
        }
    }

    for task in busy {
        if !task.blocks_availability {
            continue;
        }

        let (Some(starts_at), Some(ends_at)) = (task.starts_at, task.ends_at) else {
            continue;
        };

        if overlaps(starts_at, ends_at, window.starts_at, window.ends_at) {
            conflicts.push(Conflict {
                kind: ConflictKind::OverlappingTask { task_id: task.id },
                starts_at,
                ends_at,
            });
        }
    }

    conflicts.extend(outside_work_hours_conflicts(window, work_time, tz));

    conflicts
}

/// Half-open interval overlap: touching endpoints do not count as a
/// conflict — an absence or work order that ends exactly when the checked
/// window starts (or the reverse) is adjacent, not overlapping.
fn overlaps(
    a_start: DateTime<Utc>,
    a_end: DateTime<Utc>,
    b_start: DateTime<Utc>,
    b_end: DateTime<Utc>,
) -> bool {
    a_start < b_end && a_end > b_start
}

/// Walks every local calendar day `window` touches (in `tz`) and, for each
/// one, checks whether that day's slice of the window is fully covered by
/// the resource's work-time intervals. A day absent from `work_time` counts
/// as zero coverage — `expand_work_slots`'s sparse map means "not worked",
/// never "worked all day" (see the planning module design doc).
fn outside_work_hours_conflicts(
    window: TimeRange,
    work_time: &BTreeMap<NaiveDate, Vec<MinuteInterval>>,
    tz: Tz,
) -> Vec<Conflict> {
    let mut conflicts = Vec::new();

    let local_start = window.starts_at.with_timezone(&tz);
    let local_end = window.ends_at.with_timezone(&tz);
    let first_date = local_start.date_naive();
    let last_date = local_end.date_naive();

    let mut date = first_date;
    loop {
        let day_start_minute: i16 = if date == first_date {
            (local_start.hour() * 60 + local_start.minute()) as i16
        } else {
            0
        };
        let day_end_minute: i16 = if date == last_date {
            (local_end.hour() * 60 + local_end.minute()) as i16
        } else {
            1440
        };

        if day_start_minute < day_end_minute {
            let covered = work_time.get(&date).is_some_and(|intervals| {
                is_fully_covered(day_start_minute, day_end_minute, intervals)
            });

            if !covered {
                conflicts.push(Conflict {
                    kind: ConflictKind::OutsideWorkHours,
                    starts_at: local_minute_to_utc(date, day_start_minute, tz),
                    ends_at: local_minute_to_utc(date, day_end_minute, tz),
                });
            }
        }

        if date == last_date {
            break;
        }
        date = date
            .succ_opt()
            .expect("window is bounded, so this stays within NaiveDate's representable range");
    }

    conflicts
}

/// Whether every minute in `[range_start, range_end)` is covered by the
/// union of `intervals`. A candidate slot straddling the edge of a work
/// slot, or landing entirely between two of them, is *not* fully covered.
fn is_fully_covered(range_start: i16, range_end: i16, intervals: &[MinuteInterval]) -> bool {
    let mut clipped: Vec<(i16, i16)> = intervals
        .iter()
        .filter_map(|interval| {
            let start = interval.starts_minute.max(range_start);
            let end = interval.ends_minute.min(range_end);
            (start < end).then_some((start, end))
        })
        .collect();
    clipped.sort_unstable();

    let mut cursor = range_start;
    for (start, end) in clipped {
        if start > cursor {
            return false;
        }
        cursor = cursor.max(end);
        if cursor >= range_end {
            return true;
        }
    }

    cursor >= range_end
}

/// Converts a local calendar minute-of-day back to a UTC instant, via `tz`.
/// A UTC instant always maps to *some* local wall-clock time going the
/// other way (which is all `outside_work_hours_conflicts` does when reading
/// `window`), but constructing a local wall-clock time from scratch can, in
/// principle, land in a DST gap. Real-world day boundaries (midnight) and
/// zone transitions (typically 2am–3am local) essentially never coincide,
/// so `LocalResult::None` is unreachable in practice; it is still handled
/// rather than unwrapped.
fn local_minute_to_utc(date: NaiveDate, minute: i16, tz: Tz) -> DateTime<Utc> {
    let naive = date
        .and_hms_opt(0, 0, 0)
        .expect("midnight is always a valid time")
        + chrono::Duration::minutes(minute as i64);

    match tz.from_local_datetime(&naive) {
        chrono::LocalResult::Single(dt) => dt.with_timezone(&Utc),
        // Ambiguous (a fall-back DST transition): pick the earlier of the
        // two instants — the conflict's bounds stay a conservative,
        // deterministic choice rather than an arbitrary one.
        chrono::LocalResult::Ambiguous(earliest, _latest) => earliest.with_timezone(&Utc),
        // No such local time (a spring-forward DST gap): fall back to a
        // literal UTC interpretation rather than panicking. This never
        // happens for a real midnight-anchored day boundary in practice.
        chrono::LocalResult::None => Utc.from_utc_datetime(&naive),
    }
}

/// Converts the local `[from, to]` calendar window (inclusive) into the
/// `[start-of-from, start-of-day-after-to)` UTC instant range used to query
/// `TIMESTAMPTZ` entries (absences, work orders) — the counterpart of
/// [`local_date_span`].
fn local_date_range_to_utc(range: DateRange, tz: Tz) -> Result<TimeRange, CoreError> {
    let starts_at = local_minute_to_utc(range.from, 0, tz);
    let day_after_to = range.to.succ_opt().ok_or_else(|| {
        CoreError::Internal("planning window `to` is at the maximum representable date".to_owned())
    })?;
    let ends_at = local_minute_to_utc(day_after_to, 0, tz);

    TimeRange::new(starts_at, ends_at)
}

/// The local calendar dates `window` touches, in `tz` — the counterpart of
/// [`local_date_range_to_utc`], used to bound the rhythm/work-slot queries
/// for a single availability check.
fn local_date_span(window: TimeRange, tz: Tz) -> DateRange {
    let from = window.starts_at.with_timezone(&tz).date_naive();
    let to = window.ends_at.with_timezone(&tz).date_naive().max(from);

    DateRange::new(from, to).expect("`to` is clamped to be at least `from`")
}

// ---------------------------------------------------------------------------
// PlanningService — orchestrates the read model's assembly. I/O-bound,
// unlike `detect_conflicts` above: this is where the batched loads happen
// and get grouped in memory (see the planning module design doc's N+1
// warning).
// ---------------------------------------------------------------------------

/// Assembles the planning read model. Composes the dedicated
/// `PlanningRepository` (tasks, absences, rhythms, work slots) with
/// the existing employee/member/organization/user repositories — mirroring
/// how `TaskService` composes cross-aggregate repositories directly in
/// the domain service that owns the use case.
pub struct PlanningService<PR, ER, MR, OR>
where
    PR: PlanningRepository,
    ER: EmployeeRepository,
    MR: MemberRepository,
    OR: OrganizationRepository,
{
    planning_repository: PR,
    employee_repository: ER,
    member_repository: MR,
    organization_repository: OR,
}

impl<PR, ER, MR, OR> PlanningService<PR, ER, MR, OR>
where
    PR: PlanningRepository,
    ER: EmployeeRepository,
    MR: MemberRepository,
    OR: OrganizationRepository,
{
    pub fn new(
        planning_repository: PR,
        employee_repository: ER,
        member_repository: MR,
        organization_repository: OR,
    ) -> Self {
        Self {
            planning_repository,
            employee_repository,
            member_repository,
            organization_repository,
        }
    }

    /// Assembles `GET /planning`'s response: resources, entries (tasks +
    /// absences) and work time, all over `range`. Never computes conflicts
    /// — that is `get_availability`'s job.
    pub async fn get_planning(
        &mut self,
        organization_id: OrganizationId,
        range: DateRange,
    ) -> Result<PlanningView, CoreError> {
        let timezone = self.find_timezone(organization_id).await?;
        let tz = parse_tz(&timezone)?;

        let resources = self.load_resources(organization_id).await?;
        let window = local_date_range_to_utc(range, tz)?;

        let tasks = self
            .planning_repository
            .list_tasks_in_window(organization_id, window.starts_at, window.ends_at)
            .await?;
        let absences = self
            .planning_repository
            .list_absences_in_window(organization_id, window.starts_at, window.ends_at)
            .await?;
        let rhythms = self
            .planning_repository
            .list_rhythms_for_organization(organization_id, range.from, range.to)
            .await?;
        let work_slots = self
            .planning_repository
            .list_work_slots_for_organization(organization_id, range.from, range.to)
            .await?;

        let entries = build_entries(&tasks, &absences);
        let work_time = build_work_time(&resources, &rhythms, &work_slots, range);

        Ok(PlanningView {
            timezone,
            resources,
            entries,
            work_time,
        })
    }

    /// Assembles `GET /planning/availability`'s response: every resource of
    /// the organization, annotated with why it is or is not available for
    /// `window`.
    pub async fn get_availability(
        &mut self,
        organization_id: OrganizationId,
        window: TimeRange,
    ) -> Result<Vec<AvailabilityReport>, CoreError> {
        let timezone = self.find_timezone(organization_id).await?;
        let tz = parse_tz(&timezone)?;

        let resources = self.load_resources(organization_id).await?;
        let local_span = local_date_span(window, tz);

        let tasks = self
            .planning_repository
            .list_tasks_in_window(organization_id, window.starts_at, window.ends_at)
            .await?;
        let absences = self
            .planning_repository
            .list_absences_in_window(organization_id, window.starts_at, window.ends_at)
            .await?;
        let rhythms = self
            .planning_repository
            .list_rhythms_for_organization(organization_id, local_span.from, local_span.to)
            .await?;
        let work_slots = self
            .planning_repository
            .list_work_slots_for_organization(organization_id, local_span.from, local_span.to)
            .await?;

        let mut reports = Vec::with_capacity(resources.len());
        for resource in resources {
            // Everything that concerns the person is filtered by the member;
            // only the rhythm — the translation of a contract into recurring
            // slots — is filtered by the profile, and a member without one
            // simply has no rhythm to expand.
            let member_absences: Vec<Absence> = absences
                .iter()
                .filter(|absence| absence.member_id == resource.member_id)
                .cloned()
                .collect();
            let member_busy: Vec<Task> = tasks
                .iter()
                .filter(|planning_task| {
                    planning_task
                        .task
                        .assignments
                        .iter()
                        .any(|assignment| assignment.member_id == resource.member_id)
                })
                .map(|planning_task| planning_task.task.clone())
                .collect();
            let member_rhythms: Vec<EmployeeRhythm> = match resource.employee_id {
                Some(employee_id) => rhythms
                    .iter()
                    .filter(|rhythm| rhythm.employee_id == employee_id)
                    .cloned()
                    .collect(),
                None => Vec::new(),
            };
            let member_work_slots: Vec<WorkSlot> = work_slots
                .iter()
                .filter(|slot| slot.member_id == resource.member_id)
                .cloned()
                .collect();

            let work_time = expand_work_slots(&member_rhythms, &member_work_slots, local_span);

            // Someone who has declared no working time at all — no rhythm, no
            // dated slot — cannot be "outside" it. Reporting them unavailable
            // everywhere would make every member without a contract unplannable,
            // which is precisely what a seat existing on its own is meant to
            // allow. Absences and overlapping tasks still count: those are facts
            // about the person, not about a contract they do not have.
            let declares_working_time = !member_rhythms.is_empty() || !member_work_slots.is_empty();

            let mut conflicts =
                detect_conflicts(window, &member_absences, &work_time, &member_busy, tz);
            if !declares_working_time {
                conflicts
                    .retain(|conflict| !matches!(conflict.kind, ConflictKind::OutsideWorkHours));
            }

            reports.push(AvailabilityReport {
                resource,
                conflicts,
            });
        }

        Ok(reports)
    }

    async fn find_timezone(
        &mut self,
        organization_id: OrganizationId,
    ) -> Result<String, CoreError> {
        self.organization_repository
            .find_timezone(organization_id)
            .await?
            .ok_or(CoreError::NotFound)
    }

    /// One row per active member, with its contractual profile attached when
    /// there is one.
    ///
    /// This used to reconcile two rosters by `user_id`: employees on one side,
    /// members on the other, deduplicated into a single list, with a best-effort
    /// user lookup and an `"Unknown"` fallback for the name. None of that is
    /// needed now — a member carries its own name and is plannable on its own,
    /// so the roster *is* the member list and the profile is a lookup on the
    /// side.
    async fn load_resources(
        &mut self,
        organization_id: OrganizationId,
    ) -> Result<Vec<PlanningResource>, CoreError> {
        let members = self
            .member_repository
            .list_active_by_organization(organization_id)
            .await?;
        let employees = self
            .employee_repository
            .list_active_by_organization(organization_id)
            .await?;

        let profiles: HashMap<MemberId, Employee> = employees
            .into_iter()
            .map(|employee| (employee.member_id, employee))
            .collect();

        Ok(members
            .into_iter()
            .map(|member| {
                let profile = profiles.get(&member.id);

                PlanningResource {
                    member_id: member.id,
                    display_name: member.display_name(),
                    employee_id: profile.map(|p| p.id),
                    hourly_rate_cents: profile.and_then(|p| p.hourly_rate_cents),
                    weekly_contract_minutes: profile
                        .map(|p| p.weekly_contract_minutes)
                        .unwrap_or(0),
                }
            })
            .collect())
    }
}

fn parse_tz(timezone: &str) -> Result<Tz, CoreError> {
    timezone
        .parse()
        .map_err(|_| CoreError::Internal(format!("invalid organization timezone `{timezone}`")))
}

fn build_entries(tasks: &[PlanningTask], absences: &[Absence]) -> Vec<PlanningEntry> {
    let mut entries = Vec::with_capacity(tasks.len() + absences.len());

    for planning_task in tasks {
        let task = &planning_task.task;
        // `PlanningRepository::list_tasks_in_window` resolves a dateless
        // subtask's window to its parent's before returning it (see that
        // trait method's doc comment and `resolve_task_window`), so both
        // fields are `Some` in practice for every row reaching this point.
        // Skipped defensively rather than trusted blindly: a future
        // repository implementation, or a directly-constructed
        // `PlanningTask` in a test, could still hand back an unresolved
        // one.
        let (Some(starts_at), Some(ends_at)) = (task.starts_at, task.ends_at) else {
            continue;
        };

        entries.push(PlanningEntry::Task {
            id: task.id,
            parent_task_id: task.parent_task_id,
            starts_at,
            ends_at,
            all_day: task.all_day,
            status: task.status,
            blocks_availability: task.blocks_availability,
            child_count: planning_task.child_count,
            title: task.title.clone(),
            customer_name: planning_task.customer_name.clone(),
            context_label: planning_task.context_label.clone(),
            description: task.description.clone(),
            member_ids: task
                .assignments
                .iter()
                .map(|assignment| assignment.member_id)
                .collect(),
            // Populated by `MestierUseCase::get_planning`, after this
            // service returns — see `PlanningEntry::Task::labels`'s doc
            // comment.
            labels: Vec::new(),
        });
    }

    for absence in absences {
        entries.push(PlanningEntry::Absence {
            id: absence.id,
            starts_at: absence.starts_at,
            ends_at: absence.ends_at,
            all_day: absence.all_day,
            absence_kind: absence.kind,
            note: absence.note.clone(),
            member_id: absence.member_id,
        });
    }

    entries
}

/// One `MemberWorkTime` per employee resource — always, even when
/// `expand_work_slots` returns an empty map for them — mirroring how
/// `resources` lists every employee regardless of whether they have any
/// entries. Member-only resources carry no work-time concept and are
/// skipped.
fn build_work_time(
    resources: &[PlanningResource],
    rhythms: &[EmployeeRhythm],
    work_slots: &[WorkSlot],
    range: DateRange,
) -> Vec<MemberWorkTime> {
    // Every member gets a row, contract or not: someone with no rhythm still
    // has dated work slots, and a grid that skipped them would show them as
    // never available rather than as simply unscheduled.
    resources
        .iter()
        .map(|resource| {
            let member_rhythms: Vec<EmployeeRhythm> = match resource.employee_id {
                Some(employee_id) => rhythms
                    .iter()
                    .filter(|rhythm| rhythm.employee_id == employee_id)
                    .cloned()
                    .collect(),
                None => Vec::new(),
            };
            let member_work_slots: Vec<WorkSlot> = work_slots
                .iter()
                .filter(|slot| slot.member_id == resource.member_id)
                .cloned()
                .collect();

            MemberWorkTime {
                member_id: resource.member_id,
                days: expand_work_slots(&member_rhythms, &member_work_slots, range),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        AbsenceId, AbsenceKind, CustomerContextId, CustomerId, EmployeeId, Member, MemberId,
        QuoteId, RhythmSlot, RhythmSlotId, TaskAssignment, TaskAssignmentId, TaskId, TaskStatus,
        UserId,
        domain::{
            employee::ports::MockEmployeeRepository, member::ports::MockMemberRepository,
            organization::ports::MockOrganizationRepository,
            planning::ports::MockPlanningRepository,
        },
    };
    use chrono_tz::Europe;
    use mockall::predicate::eq;
    use uuid::Uuid;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    fn utc(y: i32, m: u32, d: u32, h: u32, min: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(y, m, d, h, min, 0).unwrap()
    }

    fn window(y: i32, m: u32, d: u32, from_h: u32, to_h: u32) -> TimeRange {
        TimeRange::new(utc(y, m, d, from_h, 0), utc(y, m, d, to_h, 0)).unwrap()
    }

    fn absence(
        member_id: MemberId,
        kind: AbsenceKind,
        starts_at: DateTime<Utc>,
        ends_at: DateTime<Utc>,
    ) -> Absence {
        Absence {
            id: AbsenceId(Uuid::new_v4()),
            organization_id: OrganizationId(Uuid::new_v4()),
            member_id,
            kind,
            starts_at,
            ends_at,
            all_day: false,
            note: Some("note".to_owned()),
            deleted_at: None,
            created_at: starts_at,
            updated_at: starts_at,
        }
    }

    fn seat(id: MemberId, organization_id: OrganizationId, last_name: &str) -> Member {
        Member {
            id,
            organization_id,
            user_id: None,
            last_name: last_name.to_owned(),
            first_name: None,
            joined_at: None,
            created_at: Utc::now(),
            deleted_at: None,
        }
    }

    /// A grid row takes its name from the seat, so there is nothing left for
    /// the profile to format. Two tests here used to check that
    /// `employee_resource` rendered "last first" — that function is gone with
    /// the two-variant resource, and `Member::display_name` is now the one
    /// place the format lives.
    #[test]
    fn resource_takes_its_display_name_from_the_seat() {
        let member_id = MemberId(Uuid::new_v4());
        let resource = PlanningResource {
            member_id,
            display_name: "Bonnal Baptiste".to_owned(),
            employee_id: None,
            hourly_rate_cents: None,
            weekly_contract_minutes: 0,
        };

        assert_eq!(resource.display_name, "Bonnal Baptiste");
        assert_eq!(resource.resource_id(), format!("member:{member_id}"));
    }

    fn task(starts_at: DateTime<Utc>, ends_at: DateTime<Utc>) -> Task {
        Task {
            id: TaskId(Uuid::new_v4()),
            organization_id: OrganizationId(Uuid::new_v4()),
            parent_task_id: None,
            title: "Toiture".to_owned(),
            description: None,
            starts_at: Some(starts_at),
            ends_at: Some(ends_at),
            all_day: false,
            status: TaskStatus::Planned,
            blocks_availability: true,
            customer_id: Some(CustomerId(Uuid::new_v4())),
            customer_context_id: Some(CustomerContextId(Uuid::new_v4())),
            quote_id: None::<QuoteId>,
            project_id: None,
            expenses_cents: 0,
            expenses_label: None,
            assignments: Vec::new(),
            deleted_at: None,
            created_at: starts_at,
            updated_at: starts_at,
        }
    }

    fn interval(starts_minute: i16, ends_minute: i16) -> MinuteInterval {
        MinuteInterval {
            starts_minute,
            ends_minute,
        }
    }

    // -- detect_conflicts: absences ---------------------------------------

    #[test]
    fn absence_overlapping_the_window_is_a_conflict() {
        let member_id = MemberId(Uuid::new_v4());
        let win = window(2026, 8, 10, 9, 12);
        let absences = vec![absence(
            member_id,
            AbsenceKind::Leave,
            utc(2026, 8, 10, 8, 0),
            utc(2026, 8, 10, 10, 0),
        )];
        let work_time = BTreeMap::from([(date(2026, 8, 10), vec![interval(0, 1440)])]);

        let conflicts = detect_conflicts(win, &absences, &work_time, &[], Europe::Paris);

        assert_eq!(conflicts.len(), 1);
        assert!(matches!(
            &conflicts[0].kind,
            ConflictKind::Absence { kind: AbsenceKind::Leave, note: Some(n) } if n == "note"
        ));
    }

    #[test]
    fn absence_adjacent_to_the_window_is_not_a_conflict() {
        let member_id = MemberId(Uuid::new_v4());
        let win = window(2026, 8, 10, 9, 12);
        // Ends exactly when the window starts: touching, not overlapping.
        let absences = vec![absence(
            member_id,
            AbsenceKind::Leave,
            utc(2026, 8, 10, 7, 0),
            utc(2026, 8, 10, 9, 0),
        )];
        let work_time = BTreeMap::from([(date(2026, 8, 10), vec![interval(0, 1440)])]);

        let conflicts = detect_conflicts(win, &absences, &work_time, &[], Europe::Paris);

        assert!(conflicts.is_empty());
    }

    // -- detect_conflicts: work hours --------------------------------------

    #[test]
    fn a_slot_entirely_outside_work_hours_is_a_conflict() {
        // Local Europe/Paris in August is UTC+2; work hours are 08:00-12:00
        // local (window uses UTC hours directly, so pick a UTC window that
        // maps outside that local range).
        let win = window(2026, 8, 10, 18, 19); // 20:00-21:00 local
        let work_time = BTreeMap::from([(date(2026, 8, 10), vec![interval(480, 720)])]); // 08:00-12:00 local

        let conflicts = detect_conflicts(win, &[], &work_time, &[], Europe::Paris);

        assert_eq!(conflicts.len(), 1);
        assert!(matches!(conflicts[0].kind, ConflictKind::OutsideWorkHours));
    }

    #[test]
    fn a_slot_straddling_the_edge_of_a_work_slot_is_a_conflict() {
        // Work hours 08:00-12:00 local (480-720); window 11:00-13:00 local
        // straddles the end of the work slot.
        let win = TimeRange::new(utc(2026, 8, 10, 9, 0), utc(2026, 8, 10, 11, 0)).unwrap();
        let work_time = BTreeMap::from([(date(2026, 8, 10), vec![interval(480, 720)])]);

        let conflicts = detect_conflicts(win, &[], &work_time, &[], Europe::Paris);

        assert_eq!(conflicts.len(), 1);
        assert!(matches!(conflicts[0].kind, ConflictKind::OutsideWorkHours));
    }

    #[test]
    fn a_slot_fully_covered_by_a_work_slot_is_not_a_conflict() {
        // 09:00-10:00 local, fully inside 08:00-12:00 local.
        let win = TimeRange::new(utc(2026, 8, 10, 7, 0), utc(2026, 8, 10, 8, 0)).unwrap();
        let work_time = BTreeMap::from([(date(2026, 8, 10), vec![interval(480, 720)])]);

        let conflicts = detect_conflicts(win, &[], &work_time, &[], Europe::Paris);

        assert!(conflicts.is_empty());
    }

    #[test]
    fn a_day_absent_from_the_work_time_map_is_a_conflict() {
        let win = window(2026, 8, 10, 7, 8); // 09:00-10:00 local
        let work_time: BTreeMap<NaiveDate, Vec<MinuteInterval>> = BTreeMap::new();

        let conflicts = detect_conflicts(win, &[], &work_time, &[], Europe::Paris);

        assert_eq!(conflicts.len(), 1);
        assert!(matches!(conflicts[0].kind, ConflictKind::OutsideWorkHours));
    }

    // -- detect_conflicts: overlapping tasks --------------------------------

    #[test]
    fn an_overlapping_blocking_task_is_a_conflict() {
        let win = window(2026, 8, 10, 9, 12);
        let busy = vec![task(utc(2026, 8, 10, 10, 0), utc(2026, 8, 10, 11, 0))];
        let work_time = BTreeMap::from([(date(2026, 8, 10), vec![interval(0, 1440)])]);

        let conflicts = detect_conflicts(win, &[], &work_time, &busy, Europe::Paris);

        assert_eq!(conflicts.len(), 1);
        assert!(matches!(
            conflicts[0].kind,
            ConflictKind::OverlappingTask { .. }
        ));
    }

    #[test]
    fn an_overlapping_task_that_does_not_block_availability_is_not_a_conflict() {
        let win = window(2026, 8, 10, 9, 12);
        let busy = vec![Task {
            blocks_availability: false,
            ..task(utc(2026, 8, 10, 10, 0), utc(2026, 8, 10, 11, 0))
        }];
        let work_time = BTreeMap::from([(date(2026, 8, 10), vec![interval(0, 1440)])]);

        let conflicts = detect_conflicts(win, &[], &work_time, &busy, Europe::Paris);

        assert!(
            conflicts.is_empty(),
            "a reminder-like task (`blocks_availability = false`) must never produce a conflict, \
             even while it overlaps the checked window"
        );
    }

    #[test]
    fn an_adjacent_task_is_not_a_conflict() {
        let win = window(2026, 8, 10, 9, 12);
        // Starts exactly when the window ends: touching, not overlapping.
        let busy = vec![task(utc(2026, 8, 10, 12, 0), utc(2026, 8, 10, 13, 0))];
        let work_time = BTreeMap::from([(date(2026, 8, 10), vec![interval(0, 1440)])]);

        let conflicts = detect_conflicts(win, &[], &work_time, &busy, Europe::Paris);

        assert!(conflicts.is_empty());
    }

    #[test]
    fn a_task_with_no_dates_of_its_own_is_skipped_rather_than_panicking() {
        // A subtask inheriting its parent's window carries `None` for both
        // — `detect_conflicts` must not choke on it (resolving it is
        // `resolve_task_window`'s job, upstream of this function).
        let win = window(2026, 8, 10, 9, 12);
        let inheriting_subtask = Task {
            parent_task_id: Some(TaskId(Uuid::new_v4())),
            starts_at: None,
            ends_at: None,
            ..task(utc(2026, 8, 10, 10, 0), utc(2026, 8, 10, 11, 0))
        };
        let work_time = BTreeMap::from([(date(2026, 8, 10), vec![interval(0, 1440)])]);

        let conflicts =
            detect_conflicts(win, &[], &work_time, &[inheriting_subtask], Europe::Paris);

        assert!(conflicts.is_empty());
    }

    // -- detect_conflicts: multiple conflicts --------------------------------

    #[test]
    fn several_simultaneous_conflicts_are_all_reported() {
        let member_id = MemberId(Uuid::new_v4());
        let win = window(2026, 8, 10, 18, 19); // 20:00-21:00 local, outside work hours
        let absences = vec![absence(
            member_id,
            AbsenceKind::Sick,
            utc(2026, 8, 10, 17, 0),
            utc(2026, 8, 10, 19, 0),
        )];
        let busy = vec![task(utc(2026, 8, 10, 18, 30), utc(2026, 8, 10, 19, 30))];
        let work_time = BTreeMap::from([(date(2026, 8, 10), vec![interval(480, 720)])]);

        let conflicts = detect_conflicts(win, &absences, &work_time, &busy, Europe::Paris);

        assert_eq!(conflicts.len(), 3);
        assert!(
            conflicts
                .iter()
                .any(|c| matches!(c.kind, ConflictKind::Absence { .. }))
        );
        assert!(
            conflicts
                .iter()
                .any(|c| matches!(c.kind, ConflictKind::OutsideWorkHours))
        );
        assert!(
            conflicts
                .iter()
                .any(|c| matches!(c.kind, ConflictKind::OverlappingTask { .. }))
        );
    }

    #[test]
    fn a_fully_covered_window_with_nothing_else_has_no_conflicts() {
        let win = window(2026, 8, 10, 6, 7); // 08:00-09:00 local
        let work_time = BTreeMap::from([(date(2026, 8, 10), vec![interval(480, 720)])]);

        let conflicts = detect_conflicts(win, &[], &work_time, &[], Europe::Paris);

        assert!(conflicts.is_empty());
    }

    // -- detect_conflicts: DST ------------------------------------------------

    /// 2026-03-29 is Europe/Paris's spring-forward DST transition (01:00
    /// UTC, clocks jump 02:00 -> 03:00 local). The same UTC wall-clock hour
    /// therefore maps to a *different* local hour the day before (UTC+1)
    /// than the day of/after (UTC+2) — proving the conversion is zoned, not
    /// a fixed offset.
    #[test]
    fn dst_spring_forward_shifts_the_same_utc_hour_into_work_hours() {
        let work_time_pre = BTreeMap::from([(date(2026, 3, 28), vec![interval(540, 600)])]); // 09:00-10:00 local
        let work_time_post = BTreeMap::from([(date(2026, 3, 29), vec![interval(540, 600)])]);

        // 07:00-07:30 UTC both days.
        let win_pre = TimeRange::new(utc(2026, 3, 28, 7, 0), utc(2026, 3, 28, 7, 30)).unwrap();
        let win_post = TimeRange::new(utc(2026, 3, 29, 7, 0), utc(2026, 3, 29, 7, 30)).unwrap();

        // Pre-transition: UTC+1, 07:00 UTC = 08:00 local, outside 09:00-10:00.
        let conflicts_pre = detect_conflicts(win_pre, &[], &work_time_pre, &[], Europe::Paris);
        assert_eq!(
            conflicts_pre.len(),
            1,
            "07:00 UTC on 2026-03-28 is 08:00 CET, outside the 09:00-10:00 work slot"
        );
        assert!(matches!(
            conflicts_pre[0].kind,
            ConflictKind::OutsideWorkHours
        ));

        // Post-transition: UTC+2, 07:00 UTC = 09:00 local, inside 09:00-10:00.
        let conflicts_post = detect_conflicts(win_post, &[], &work_time_post, &[], Europe::Paris);
        assert!(
            conflicts_post.is_empty(),
            "07:00 UTC on 2026-03-29 is 09:00 CEST, inside the 09:00-10:00 work slot"
        );
    }

    // -- PlanningService: resource dedup (mockall) --------------------------

    fn employee(id: EmployeeId, member_id: MemberId) -> Employee {
        let now = Utc::now();
        Employee {
            id,
            organization_id: OrganizationId(Uuid::new_v4()),
            member_id,
            hourly_rate_cents: Some(3500),
            is_salaried: false,
            weekly_contract_minutes: 2100,
            deleted_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn service(
        planning_repository: MockPlanningRepository,
        employee_repository: MockEmployeeRepository,
        member_repository: MockMemberRepository,
        organization_repository: MockOrganizationRepository,
    ) -> PlanningService<
        MockPlanningRepository,
        MockEmployeeRepository,
        MockMemberRepository,
        MockOrganizationRepository,
    > {
        PlanningService::new(
            planning_repository,
            employee_repository,
            member_repository,
            organization_repository,
        )
    }

    fn empty_repository_expectations(
        planning_repository: &mut MockPlanningRepository,
        organization_id: OrganizationId,
    ) {
        planning_repository
            .expect_list_tasks_in_window()
            .withf(move |org, _, _| *org == organization_id)
            .returning(|_, _, _| Box::pin(async { Ok(Vec::new()) }));
        planning_repository
            .expect_list_absences_in_window()
            .withf(move |org, _, _| *org == organization_id)
            .returning(|_, _, _| Box::pin(async { Ok(Vec::new()) }));
        planning_repository
            .expect_list_rhythms_for_organization()
            .withf(move |org, _, _| *org == organization_id)
            .returning(|_, _, _| Box::pin(async { Ok(Vec::new()) }));
        planning_repository
            .expect_list_work_slots_for_organization()
            .withf(move |org, _, _| *org == organization_id)
            .returning(|_, _, _| Box::pin(async { Ok(Vec::new()) }));
    }

    /// One roster, one row per member — with or without a contract.
    ///
    /// Three tests used to live here: one for the dedup of a person who was
    /// both member and employee, one adding a row for a member with no HR
    /// record, and one skipping a free seat that had no user to name it. All
    /// three described a read model that reconciled two rosters. There is one
    /// roster now, and a seat carries its own name, so what is left to check is
    /// that the contract is attached when it exists and absent when it does not.
    #[tokio::test]
    async fn get_planning_lists_every_member_once_with_its_profile_when_it_has_one() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let with_contract = MemberId(Uuid::new_v4());
        let without_contract = MemberId(Uuid::new_v4());
        let employee_id = EmployeeId(Uuid::new_v4());

        let mut organization_repository = MockOrganizationRepository::new();
        organization_repository
            .expect_find_timezone()
            .with(eq(organization_id))
            .returning(|_| Box::pin(async { Ok(Some("Europe/Paris".to_owned())) }));

        let mut employee_repository = MockEmployeeRepository::new();
        employee_repository
            .expect_list_active_by_organization()
            .with(eq(organization_id))
            .returning(move |_| {
                Box::pin(async move { Ok(vec![employee(employee_id, with_contract)]) })
            });

        let mut member_repository = MockMemberRepository::new();
        member_repository
            .expect_list_active_by_organization()
            .with(eq(organization_id))
            .returning(move |org_id| {
                Box::pin(async move {
                    Ok(vec![
                        seat(with_contract, org_id, "Avec"),
                        seat(without_contract, org_id, "Sans"),
                    ])
                })
            });

        // No `expect_list_by_ids`: names come from the seats themselves, so
        // the user repository is never consulted to build the roster.

        let mut planning_repository = MockPlanningRepository::new();
        empty_repository_expectations(&mut planning_repository, organization_id);

        let mut service = service(
            planning_repository,
            employee_repository,
            member_repository,
            organization_repository,
        );

        let range = DateRange::new(date(2026, 8, 1), date(2026, 8, 7)).unwrap();
        let view = service.get_planning(organization_id, range).await.unwrap();

        assert_eq!(view.resources.len(), 2);

        let contracted = view
            .resources
            .iter()
            .find(|r| r.member_id == with_contract)
            .expect("the member with a contract must appear");
        assert_eq!(contracted.employee_id, Some(employee_id));
        assert_eq!(contracted.hourly_rate_cents, Some(3500));
        assert_eq!(contracted.weekly_contract_minutes, 2100);

        let bare = view
            .resources
            .iter()
            .find(|r| r.member_id == without_contract)
            .expect("the member with no contract must appear all the same");
        assert_eq!(bare.employee_id, None);
        assert_eq!(
            bare.hourly_rate_cents, None,
            "no contract means no rate — never a rate of zero"
        );
        assert_eq!(bare.weekly_contract_minutes, 0);
    }

    #[tokio::test]
    async fn get_availability_reports_a_member_only_resource_as_always_available() {
        let organization_id = OrganizationId(Uuid::new_v4());
        let user_id = UserId(Uuid::new_v4());

        let mut organization_repository = MockOrganizationRepository::new();
        organization_repository
            .expect_find_timezone()
            .with(eq(organization_id))
            .returning(|_| Box::pin(async { Ok(Some("Europe/Paris".to_owned())) }));

        let mut employee_repository = MockEmployeeRepository::new();
        employee_repository
            .expect_list_active_by_organization()
            .with(eq(organization_id))
            .returning(|_| Box::pin(async { Ok(Vec::new()) }));

        let mut member_repository = MockMemberRepository::new();
        member_repository
            .expect_list_active_by_organization()
            .with(eq(organization_id))
            .returning(move |org_id| {
                Box::pin(async move {
                    Ok(vec![Member {
                        id: MemberId(Uuid::new_v4()),
                        organization_id: org_id,
                        user_id: Some(user_id),
                        last_name: "Member".to_owned(),
                        first_name: None,
                        joined_at: Some(Utc::now()),
                        created_at: Utc::now(),
                        deleted_at: None,
                    }])
                })
            });

        let mut planning_repository = MockPlanningRepository::new();
        empty_repository_expectations(&mut planning_repository, organization_id);

        let mut service = service(
            planning_repository,
            employee_repository,
            member_repository,
            organization_repository,
        );

        let window = TimeRange::new(utc(2026, 8, 10, 9, 0), utc(2026, 8, 10, 12, 0)).unwrap();
        let reports = service
            .get_availability(organization_id, window)
            .await
            .unwrap();

        assert_eq!(reports.len(), 1);
        assert!(reports[0].available());
    }

    #[test]
    fn build_entries_flattens_a_task_and_an_absence() {
        let member_id = MemberId(Uuid::new_v4());
        let mut t = task(utc(2026, 8, 10, 8, 0), utc(2026, 8, 10, 12, 0));
        t.assignments.push(TaskAssignment {
            id: TaskAssignmentId(Uuid::new_v4()),
            organization_id: t.organization_id,
            task_id: t.id,
            member_id,
            created_at: t.created_at,
        });
        let planning_tasks = vec![PlanningTask {
            task: t,
            customer_name: Some("Alice Dupont".to_owned()),
            context_label: Some("Chantier principal".to_owned()),
            child_count: 0,
        }];
        let absences = vec![absence(
            member_id,
            AbsenceKind::Sick,
            utc(2026, 8, 11, 0, 0),
            utc(2026, 8, 12, 0, 0),
        )];

        let entries = build_entries(&planning_tasks, &absences);

        assert_eq!(entries.len(), 2);
        assert!(matches!(
            &entries[0],
            PlanningEntry::Task { customer_name: Some(customer_name), context_label: Some(context_label), member_ids, .. }
                if customer_name == "Alice Dupont"
                    && context_label == "Chantier principal"
                    && member_ids == &[member_id]
        ));
        assert!(matches!(
            &entries[1],
            PlanningEntry::Absence {
                absence_kind: AbsenceKind::Sick,
                ..
            }
        ));
    }

    #[test]
    fn build_entries_reports_no_customer_for_a_task_without_one() {
        let t = Task {
            customer_id: None,
            customer_context_id: None,
            ..task(utc(2026, 8, 10, 8, 0), utc(2026, 8, 10, 12, 0))
        };
        let planning_tasks = vec![PlanningTask {
            task: t,
            customer_name: None,
            context_label: None,
            child_count: 0,
        }];

        let entries = build_entries(&planning_tasks, &[]);

        assert_eq!(entries.len(), 1);
        assert!(matches!(
            &entries[0],
            PlanningEntry::Task {
                customer_name: None,
                context_label: None,
                ..
            }
        ));
    }

    #[test]
    fn build_entries_skips_a_task_with_no_dates_of_its_own() {
        let inheriting_subtask = Task {
            parent_task_id: Some(TaskId(Uuid::new_v4())),
            starts_at: None,
            ends_at: None,
            ..task(utc(2026, 8, 10, 8, 0), utc(2026, 8, 10, 12, 0))
        };
        let planning_tasks = vec![PlanningTask {
            task: inheriting_subtask,
            customer_name: None,
            context_label: None,
            child_count: 0,
        }];

        let entries = build_entries(&planning_tasks, &[]);

        assert!(entries.is_empty());
    }

    #[test]
    /// Every member gets a work-time row, contract or not. The version of this
    /// test that skipped bare members described a grid on which someone without
    /// an HR record read as never available rather than simply unscheduled.
    fn build_work_time_covers_every_member_including_those_without_a_contract() {
        let member_id = MemberId(Uuid::new_v4());
        let employee_id = EmployeeId(Uuid::new_v4());
        let resources = vec![
            PlanningResource {
                member_id,
                display_name: "Alice".to_owned(),
                employee_id: Some(employee_id),
                hourly_rate_cents: None,
                weekly_contract_minutes: 0,
            },
            PlanningResource {
                member_id: MemberId(Uuid::new_v4()),
                display_name: "Bob".to_owned(),
                employee_id: None,
                hourly_rate_cents: None,
                weekly_contract_minutes: 0,
            },
        ];
        let rhythm_id = crate::EmployeeRhythmId(Uuid::new_v4());
        let rhythms = vec![EmployeeRhythm {
            id: rhythm_id,
            organization_id: OrganizationId(Uuid::new_v4()),
            employee_id,
            effective_from: date(2026, 1, 1),
            effective_to: None,
            slots: vec![RhythmSlot {
                id: RhythmSlotId(Uuid::new_v4()),
                rhythm_id,
                weekday: 1,
                starts_minute: 480,
                ends_minute: 720,
            }],
            created_at: Utc::now(),
            updated_at: Utc::now(),
        }];
        let range = DateRange::new(date(2026, 8, 10), date(2026, 8, 10)).unwrap(); // a Monday

        let work_time = build_work_time(&resources, &rhythms, &[], range);

        assert_eq!(
            work_time.len(),
            2,
            "every member gets a row — someone without a contract is unscheduled, not skipped"
        );
        assert_eq!(work_time[0].member_id, member_id);
        assert!(work_time[0].days.contains_key(&date(2026, 8, 10)));
        assert!(
            work_time[1].days.is_empty(),
            "a member with no rhythm and no slot declares no working day"
        );
    }
}
