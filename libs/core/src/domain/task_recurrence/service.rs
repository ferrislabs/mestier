use chrono::{DateTime, Datelike, Duration, LocalResult, NaiveDate, TimeZone, Utc};
use common::{CoreError, generate_uuid_v7};

use crate::{
    OrganizationId, Task, TaskAssignment, TaskAssignmentId, TaskId, TaskStatus,
    domain::{
        member::ports::MemberRepository,
        task::ports::TaskRepository,
        task_recurrence::{
            RecurrenceRule, TaskRecurrence, TaskRecurrenceId,
            commands::{CreateTaskRecurrenceCommand, PatchTaskRecurrenceCommand},
            ports::TaskRecurrenceRepository,
        },
    },
};

/// How far ahead a recurrence is materialized, in days, absent an
/// organization-level override (`task_recurrence_horizon_settings.horizon_days`,
/// added by #293's migration — a setting, not a constant, because a shop
/// with a lot of monthly recurrences and a slow horizon-extension worker
/// needs more margin than the default gives).
///
/// 60 days was picked over, say, 365: it comfortably covers a monthly
/// recurrence's next occurrence (at most 31 days out, with margin for a
/// worker pass that runs late) without materializing a year of a daily
/// stand-up nobody has looked at yet — each materialized occurrence is a
/// real `tasks` row, not a projection, so a longer default has a real
/// storage and query cost for every organization that never touches the
/// setting.
pub const DEFAULT_HORIZON_DAYS: i64 = 60;

// ---------------------------------------------------------------------------
// Pure functions — no I/O. `dates_in_range` and `expand_occurrences` are
// this workstream's `expand_work_slots`/`detect_conflicts`: disproportionate
// test coverage relative to their size, because everything downstream
// (materialization, the horizon-extension worker) trusts them completely.
// ---------------------------------------------------------------------------

/// Every date `rule` occurs on within `[from, to]`, further clipped to
/// `[starts_on, ends_on]` — both ends inclusive throughout.
///
/// Walks day by day rather than jumping by frequency: the ranges this is
/// ever called with are bounded by a horizon in the hundreds of days at
/// most, so the simplicity is worth far more than the (unmeasurable) cost.
pub fn dates_in_range(
    rule: &RecurrenceRule,
    starts_on: NaiveDate,
    ends_on: Option<NaiveDate>,
    from: NaiveDate,
    to: NaiveDate,
) -> Vec<NaiveDate> {
    let lower = from.max(starts_on);
    let upper = match ends_on {
        Some(e) => to.min(e),
        None => to,
    };
    if lower > upper {
        return Vec::new();
    }

    let mut dates = Vec::new();
    let mut date = lower;
    while date <= upper {
        if occurs_on(rule, date) {
            dates.push(date);
        }
        date += Duration::days(1);
    }
    dates
}

fn occurs_on(rule: &RecurrenceRule, date: NaiveDate) -> bool {
    match rule {
        RecurrenceRule::Daily => true,
        RecurrenceRule::Weekly { weekdays } => weekdays.contains(&date.weekday()),
        RecurrenceRule::Monthly { day_of_month } => {
            date.day() == clamped_day_of_month(date.year(), date.month(), *day_of_month)
        }
    }
}

/// `day_of_month`, or the month's own last day when it is shorter — so
/// "monthly on the 31st" still produces exactly one occurrence in February,
/// on the 28th (or 29th), instead of silently skipping the month.
fn clamped_day_of_month(year: i32, month: u32, day_of_month: u8) -> u32 {
    (day_of_month as u32).min(days_in_month(year, month))
}

fn days_in_month(year: i32, month: u32) -> u32 {
    let (next_year, next_month) = if month == 12 {
        (year + 1, 1)
    } else {
        (year, month + 1)
    };
    let first_of_next = NaiveDate::from_ymd_opt(next_year, next_month, 1)
        .expect("every (year, month) pair produced here is a valid calendar month");
    let first_of_this = NaiveDate::from_ymd_opt(year, month, 1)
        .expect("every (year, month) pair produced here is a valid calendar month");
    (first_of_next - first_of_this).num_days() as u32
}

/// One calendar date resolved to the UTC instant window a materialized task
/// needs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecurrenceOccurrence {
    pub date: NaiveDate,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
}

/// Resolves a wall-clock date + time in `tz` to the UTC instant it names.
///
/// "Every Tuesday at 9am" is a wall-clock claim, so this is where a DST
/// change actually matters:
/// - A repeated local time (the DST *fall-back* hour, which happens twice)
///   resolves to the **earlier** of the two instants — the first "9am" the
///   clock shows, matching how a person reading a calendar would read it.
/// - A local time that never happens (the DST *spring-forward* gap, e.g.
///   `02:00`–`03:00` skipped outright) is shifted forward by an hour and
///   resolved again, so "every day at 2:30am" still produces something on
///   the gap day instead of silently vanishing. Real IANA zones never gap by
///   more than an hour, so one shift is always enough; the final `None` arm
///   exists only so this is total rather than a possible panic.
fn local_datetime_to_utc(
    date: NaiveDate,
    time: chrono::NaiveTime,
    tz: chrono_tz::Tz,
) -> DateTime<Utc> {
    let naive = date.and_time(time);
    match tz.from_local_datetime(&naive) {
        LocalResult::Single(dt) => dt.with_timezone(&Utc),
        LocalResult::Ambiguous(earliest, _latest) => earliest.with_timezone(&Utc),
        LocalResult::None => {
            let shifted = naive + Duration::hours(1);
            match tz.from_local_datetime(&shifted) {
                LocalResult::Single(dt) => dt.with_timezone(&Utc),
                LocalResult::Ambiguous(earliest, _latest) => earliest.with_timezone(&Utc),
                LocalResult::None => Utc.from_utc_datetime(&naive),
            }
        }
    }
}

/// Every occurrence `recurrence`'s rule produces in `[from, to]`, resolved to
/// UTC instants — the function `TaskRecurrenceService` calls to know what to
/// materialize, both at creation and on each horizon extension.
pub fn expand_occurrences(
    recurrence: &TaskRecurrence,
    from: NaiveDate,
    to: NaiveDate,
) -> Vec<RecurrenceOccurrence> {
    dates_in_range(
        &recurrence.rule,
        recurrence.starts_on,
        recurrence.ends_on,
        from,
        to,
    )
    .into_iter()
    .map(|date| {
        let starts_at = local_datetime_to_utc(date, recurrence.start_time, recurrence.timezone);
        let ends_at = starts_at + Duration::minutes(recurrence.duration_minutes as i64);
        RecurrenceOccurrence {
            date,
            starts_at,
            ends_at,
        }
    })
    .collect()
}

/// The horizon date a recurrence should be filled to right now: `horizon_days`
/// out from today (in the recurrence's own timezone) or its own `starts_on`,
/// whichever is later, clipped to `ends_on` when the series has one.
pub fn target_horizon(
    now: DateTime<Utc>,
    tz: chrono_tz::Tz,
    starts_on: NaiveDate,
    ends_on: Option<NaiveDate>,
    horizon_days: i64,
) -> NaiveDate {
    let today = now.with_timezone(&tz).date_naive();
    let base = today.max(starts_on);
    let target = base + Duration::days(horizon_days);
    match ends_on {
        Some(end) => target.min(end),
        None => target,
    }
}

fn validate_title(title: &str) -> Result<(), CoreError> {
    if title.trim().is_empty() {
        return Err(CoreError::Conflict(
            "recurrence title cannot be blank".to_owned(),
        ));
    }
    Ok(())
}

fn validate_rule(rule: &RecurrenceRule) -> Result<(), CoreError> {
    match rule {
        RecurrenceRule::Daily => Ok(()),
        RecurrenceRule::Weekly { weekdays } => {
            if weekdays.is_empty() {
                return Err(CoreError::Conflict(
                    "a weekly recurrence needs at least one weekday".to_owned(),
                ));
            }
            Ok(())
        }
        RecurrenceRule::Monthly { day_of_month } => {
            if !(1..=31).contains(day_of_month) {
                return Err(CoreError::Conflict(
                    "a monthly recurrence's day must be between 1 and 31".to_owned(),
                ));
            }
            Ok(())
        }
    }
}

fn validate_customer_pairing(
    customer_id: Option<crate::CustomerId>,
    customer_context_id: Option<crate::CustomerContextId>,
) -> Result<(), CoreError> {
    if customer_context_id.is_some() && customer_id.is_none() {
        return Err(CoreError::Conflict(
            "a recurrence's customer_context_id requires a customer_id".to_owned(),
        ));
    }
    Ok(())
}

// ---------------------------------------------------------------------------
// TaskRecurrenceService — I/O-bound orchestration.
// ---------------------------------------------------------------------------

/// Orchestrates a recurrence together with the task and member repositories
/// materializing needs: `task_repository` to write the occurrence rows
/// (through `insert_occurrence_if_absent`, which is what makes a fill
/// idempotent), `member_repository` to check the template's assignees
/// belong to the organization, the same check `TaskService::resolve_assignee`
/// makes for an ordinary `PATCH`.
pub struct TaskRecurrenceService<RR, TR, MR>
where
    RR: TaskRecurrenceRepository,
    TR: TaskRepository,
    MR: MemberRepository,
{
    recurrence_repository: RR,
    task_repository: TR,
    member_repository: MR,
}

impl<RR, TR, MR> TaskRecurrenceService<RR, TR, MR>
where
    RR: TaskRecurrenceRepository,
    TR: TaskRepository,
    MR: MemberRepository,
{
    pub fn new(recurrence_repository: RR, task_repository: TR, member_repository: MR) -> Self {
        Self {
            recurrence_repository,
            task_repository,
            member_repository,
        }
    }

    /// Creates the recurrence and materializes every occurrence up to
    /// `horizon_days` out, assignments included. Every consumer of a task —
    /// planning, profitability, conflict detection, worked hours — reads the
    /// resulting rows as ordinary tasks and needs no change; that is the
    /// whole point of materializing rather than expanding the rule at read
    /// time.
    pub async fn create_recurrence(
        &mut self,
        command: CreateTaskRecurrenceCommand,
        horizon_days: i64,
    ) -> Result<TaskRecurrence, CoreError> {
        validate_title(&command.title)?;
        validate_rule(&command.rule)?;
        validate_customer_pairing(command.customer_id, command.customer_context_id)?;
        if command.duration_minutes <= 0 {
            return Err(CoreError::Conflict(
                "a recurrence's duration must be positive".to_owned(),
            ));
        }
        if let Some(ends_on) = command.ends_on
            && ends_on < command.starts_on
        {
            return Err(CoreError::Conflict(
                "a recurrence's ends_on cannot be before its starts_on".to_owned(),
            ));
        }

        for member_id in &command.assignee_member_ids {
            let member = self
                .member_repository
                .find_by_id(*member_id)
                .await?
                .ok_or(CoreError::NotFound)?;
            if member.organization_id != command.organization_id {
                return Err(CoreError::NotFound);
            }
        }

        let now = Utc::now();
        let horizon = target_horizon(
            now,
            command.timezone,
            command.starts_on,
            command.ends_on,
            horizon_days,
        );

        let recurrence = TaskRecurrence {
            id: TaskRecurrenceId(generate_uuid_v7()),
            organization_id: command.organization_id,
            rule: command.rule,
            starts_on: command.starts_on,
            ends_on: command.ends_on,
            horizon_filled_to: horizon,
            timezone: command.timezone,
            start_time: command.start_time,
            duration_minutes: command.duration_minutes,
            all_day: command.all_day,
            title: command.title,
            description: command.description,
            blocks_availability: command.blocks_availability,
            customer_id: command.customer_id,
            customer_context_id: command.customer_context_id,
            project_id: command.project_id,
            assignee_member_ids: command.assignee_member_ids,
            deleted_at: None,
            created_at: now,
            updated_at: now,
        };

        let created = self.recurrence_repository.insert(&recurrence).await?;
        self.materialize_range(&created, created.starts_on, horizon)
            .await?;

        Ok(created)
    }

    pub async fn get_recurrence(
        &mut self,
        id: TaskRecurrenceId,
    ) -> Result<TaskRecurrence, CoreError> {
        self.recurrence_repository
            .find_by_id(id)
            .await?
            .ok_or(CoreError::NotFound)
    }

    pub async fn list_recurrences(
        &mut self,
        organization_id: OrganizationId,
    ) -> Result<Vec<TaskRecurrence>, CoreError> {
        self.recurrence_repository
            .list_by_organization(organization_id)
            .await
    }

    /// Applies a `PATCH` to the rule/template. Every field is optional and
    /// only the ones present are applied — same "leave unset fields alone"
    /// contract as `task::commands::PatchTaskCommand`. Never touches
    /// `horizon_filled_to` or already-materialized occurrences: this changes
    /// what gets materialized *from here on*, not what already exists (see
    /// `TaskRecurrence`'s own doc).
    pub async fn patch_recurrence(
        &mut self,
        command: PatchTaskRecurrenceCommand,
    ) -> Result<TaskRecurrence, CoreError> {
        let mut recurrence = self.get_recurrence(command.id).await?;

        if let Some(rule) = command.rule {
            validate_rule(&rule)?;
            recurrence.rule = rule;
        }
        if let Some(ends_on) = command.ends_on {
            if let Some(end) = ends_on
                && end < recurrence.starts_on
            {
                return Err(CoreError::Conflict(
                    "a recurrence's ends_on cannot be before its starts_on".to_owned(),
                ));
            }
            recurrence.ends_on = ends_on;
        }
        if let Some(start_time) = command.start_time {
            recurrence.start_time = start_time;
        }
        if let Some(duration_minutes) = command.duration_minutes {
            if duration_minutes <= 0 {
                return Err(CoreError::Conflict(
                    "a recurrence's duration must be positive".to_owned(),
                ));
            }
            recurrence.duration_minutes = duration_minutes;
        }
        if let Some(all_day) = command.all_day {
            recurrence.all_day = all_day;
        }
        if let Some(title) = command.title {
            validate_title(&title)?;
            recurrence.title = title;
        }
        if let Some(description) = command.description {
            recurrence.description = description;
        }
        if let Some(blocks_availability) = command.blocks_availability {
            recurrence.blocks_availability = blocks_availability;
        }
        if let Some(project_id) = command.project_id {
            recurrence.project_id = project_id;
        }
        if let Some(assignee_member_ids) = command.assignee_member_ids {
            for member_id in &assignee_member_ids {
                let member = self
                    .member_repository
                    .find_by_id(*member_id)
                    .await?
                    .ok_or(CoreError::NotFound)?;
                if member.organization_id != recurrence.organization_id {
                    return Err(CoreError::NotFound);
                }
            }
            recurrence.assignee_member_ids = assignee_member_ids;
        }
        recurrence.updated_at = Utc::now();

        self.recurrence_repository.update(&recurrence).await
    }

    /// Soft-deletes the recurrence and every one of its future occurrences —
    /// `occurrence_date >= today` — leaving past ones untouched, exactly the
    /// "deleting the recurrence takes future occurrences with it and leaves
    /// past ones alone" contract. Today itself counts as future: it has not
    /// happened yet from the moment this runs.
    pub async fn delete_recurrence(&mut self, id: TaskRecurrenceId) -> Result<(), CoreError> {
        // Existence is checked first so a deleted or foreign id reads back
        // `NotFound` rather than a silent no-op affecting zero rows.
        self.get_recurrence(id).await?;

        let now = Utc::now();
        let today = now.date_naive();
        self.task_repository
            .soft_delete_recurrence_occurrences_from(id, today, now)
            .await?;
        self.recurrence_repository.soft_delete(id, now).await
    }

    /// Materializes every occurrence of `recurrence` in `[from, to]`, one
    /// `tasks` row each, assignments included. Idempotent: a date already
    /// filled (`uq_tasks_recurrence_occurrence`) is silently skipped rather
    /// than erroring, which is what lets both `create_recurrence` and a
    /// horizon-extension retry call this without tracking what already
    /// exists.
    pub async fn materialize_range(
        &mut self,
        recurrence: &TaskRecurrence,
        from: NaiveDate,
        to: NaiveDate,
    ) -> Result<u64, CoreError> {
        if from > to {
            return Ok(0);
        }

        let occurrences = expand_occurrences(recurrence, from, to);
        let now = Utc::now();
        let mut materialized = 0u64;

        for occurrence in occurrences {
            let task_id = TaskId(generate_uuid_v7());
            let assignments: Vec<TaskAssignment> = recurrence
                .assignee_member_ids
                .iter()
                .map(|member_id| TaskAssignment {
                    id: TaskAssignmentId(generate_uuid_v7()),
                    organization_id: recurrence.organization_id,
                    task_id,
                    member_id: *member_id,
                    created_at: now,
                })
                .collect();

            let task = Task {
                id: task_id,
                organization_id: recurrence.organization_id,
                parent_task_id: None,
                title: recurrence.title.clone(),
                description: recurrence.description.clone(),
                starts_at: Some(occurrence.starts_at),
                ends_at: Some(occurrence.ends_at),
                all_day: recurrence.all_day,
                status: TaskStatus::Planned,
                blocks_availability: recurrence.blocks_availability,
                customer_id: recurrence.customer_id,
                customer_context_id: recurrence.customer_context_id,
                quote_id: None,
                project_id: recurrence.project_id,
                expenses_cents: 0,
                expenses_label: None,
                assignments,
                recurrence_id: Some(recurrence.id),
                occurrence_date: Some(occurrence.date),
                deleted_at: None,
                created_at: now,
                updated_at: now,
            };

            if self
                .task_repository
                .insert_occurrence_if_absent(&task)
                .await?
            {
                materialized += 1;
            }
        }

        Ok(materialized)
    }

    /// Moves `recurrence`'s watermark to `horizon_filled_to` — never called
    /// except right after [`Self::materialize_range`] accounted for
    /// everything up to that date, and always in the same transaction as
    /// that call, which is what keeps the watermark from ever running ahead
    /// of what was actually persisted.
    pub async fn advance_horizon(
        &mut self,
        id: TaskRecurrenceId,
        horizon_filled_to: NaiveDate,
    ) -> Result<(), CoreError> {
        self.recurrence_repository
            .advance_horizon(id, horizon_filled_to)
            .await
    }

    /// The organization's configured horizon, in days, falling back to
    /// [`DEFAULT_HORIZON_DAYS`] when it has never set one.
    pub async fn horizon_days_for(
        &mut self,
        organization_id: OrganizationId,
    ) -> Result<i64, CoreError> {
        Ok(self
            .recurrence_repository
            .horizon_days_for_organization(organization_id)
            .await?
            .map(i64::from)
            .unwrap_or(DEFAULT_HORIZON_DAYS))
    }

    /// Every organization with at least one recurrence whose watermark is
    /// getting close to today — what the horizon-extension pass (#293)
    /// schedules a run for. `today` is a coarse, UTC-calendar approximation
    /// (this only decides *whether* a pass is due soon, never the actual
    /// fill target, which `extend_organization_horizons` computes precisely
    /// per recurrence in its own timezone) — see
    /// [`RECURRENCE_HORIZON_REFILL_TRIGGER_DAYS`]'s own doc for why that
    /// approximation is safe.
    pub async fn organizations_needing_extension(
        &mut self,
        today: NaiveDate,
    ) -> Result<Vec<OrganizationId>, CoreError> {
        let threshold = today + Duration::days(RECURRENCE_HORIZON_REFILL_TRIGGER_DAYS);
        self.recurrence_repository
            .organizations_needing_horizon_extension(today, threshold)
            .await
    }

    /// Extends every one of `organization_id`'s recurrences whose horizon
    /// needs pushing forward, and moves each one's watermark in the same
    /// transaction as the rows it accounts for — a failure partway through
    /// (an unlikely but possible I/O error) rolls the whole call back, so
    /// the next pass redoes it rather than resuming from a half-moved
    /// watermark. A recurrence whose `ends_on` has already passed, or whose
    /// horizon is already filled past the target, is skipped.
    ///
    /// Returns how many occurrences were newly materialized, across every
    /// recurrence — `0` is a legitimate answer (every recurrence was
    /// already filled, or none needed visiting), never a caller error.
    pub async fn extend_organization_horizons(
        &mut self,
        organization_id: OrganizationId,
    ) -> Result<u64, CoreError> {
        let horizon_days = self.horizon_days_for(organization_id).await?;
        let recurrences = self.list_recurrences(organization_id).await?;
        let now = Utc::now();
        let today = now.date_naive();
        let mut materialized = 0u64;

        for recurrence in recurrences {
            if let Some(ends_on) = recurrence.ends_on
                && ends_on < today
            {
                continue;
            }

            let target = target_horizon(
                now,
                recurrence.timezone,
                recurrence.starts_on,
                recurrence.ends_on,
                horizon_days,
            );
            if target <= recurrence.horizon_filled_to {
                continue;
            }

            let from = recurrence.horizon_filled_to + Duration::days(1);
            materialized += self.materialize_range(&recurrence, from, target).await?;
            self.advance_horizon(recurrence.id, target).await?;
        }

        Ok(materialized)
    }
}

/// How close to today a recurrence's `horizon_filled_to` has to be before a
/// run is scheduled to push it forward.
///
/// Deliberately independent from any organization's actual
/// `horizon_days` setting (which can only be read per-recurrence, inside the
/// transaction that visits it) — this is a coarse, cheap, index-friendly
/// trigger for "a pass is due soon", not the fill target itself. Any value
/// comfortably smaller than [`DEFAULT_HORIZON_DAYS`] keeps a refill from
/// ever running the watermark down to zero before a pass catches it; half of
/// the default leaves a wide margin even for an organization whose worker
/// only ticks once a day.
pub const RECURRENCE_HORIZON_REFILL_TRIGGER_DAYS: i64 = DEFAULT_HORIZON_DAYS / 2;

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::{NaiveTime, Timelike, Weekday};
    use chrono_tz::Europe;
    use uuid::Uuid;

    fn recurrence(
        rule: RecurrenceRule,
        starts_on: NaiveDate,
        ends_on: Option<NaiveDate>,
    ) -> TaskRecurrence {
        let now = Utc::now();
        TaskRecurrence {
            id: TaskRecurrenceId(Uuid::new_v4()),
            organization_id: OrganizationId(Uuid::new_v4()),
            rule,
            starts_on,
            ends_on,
            horizon_filled_to: starts_on,
            timezone: Europe::Paris,
            start_time: NaiveTime::from_hms_opt(9, 0, 0).unwrap(),
            duration_minutes: 60,
            all_day: false,
            title: "Réunion hebdo".to_owned(),
            description: None,
            blocks_availability: true,
            customer_id: None,
            customer_context_id: None,
            project_id: None,
            assignee_member_ids: Vec::new(),
            deleted_at: None,
            created_at: now,
            updated_at: now,
        }
    }

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    // -- dates_in_range: daily ------------------------------------------------

    #[test]
    fn daily_produces_every_date_in_range() {
        let dates = dates_in_range(
            &RecurrenceRule::Daily,
            date(2026, 8, 1),
            None,
            date(2026, 8, 10),
            date(2026, 8, 13),
        );

        assert_eq!(
            dates,
            vec![
                date(2026, 8, 10),
                date(2026, 8, 11),
                date(2026, 8, 12),
                date(2026, 8, 13)
            ]
        );
    }

    #[test]
    fn daily_never_produces_a_date_before_starts_on() {
        let dates = dates_in_range(
            &RecurrenceRule::Daily,
            date(2026, 8, 12),
            None,
            date(2026, 8, 1),
            date(2026, 8, 13),
        );

        assert_eq!(dates, vec![date(2026, 8, 12), date(2026, 8, 13)]);
    }

    #[test]
    fn daily_never_produces_a_date_after_ends_on() {
        let dates = dates_in_range(
            &RecurrenceRule::Daily,
            date(2026, 8, 1),
            Some(date(2026, 8, 3)),
            date(2026, 8, 1),
            date(2026, 8, 10),
        );

        assert_eq!(
            dates,
            vec![date(2026, 8, 1), date(2026, 8, 2), date(2026, 8, 3)]
        );
    }

    #[test]
    fn a_window_entirely_before_starts_on_produces_nothing() {
        let dates = dates_in_range(
            &RecurrenceRule::Daily,
            date(2026, 9, 1),
            None,
            date(2026, 8, 1),
            date(2026, 8, 10),
        );

        assert!(dates.is_empty());
    }

    // -- dates_in_range: weekly ------------------------------------------------

    /// 2026-08-04 is a Tuesday; 2026-08-01 is a Saturday.
    #[test]
    fn weekly_on_tuesday_matches_only_tuesdays() {
        let dates = dates_in_range(
            &RecurrenceRule::Weekly {
                weekdays: vec![Weekday::Tue],
            },
            date(2026, 8, 1),
            None,
            date(2026, 8, 1),
            date(2026, 8, 31),
        );

        assert_eq!(
            dates,
            vec![
                date(2026, 8, 4),
                date(2026, 8, 11),
                date(2026, 8, 18),
                date(2026, 8, 25),
            ]
        );
    }

    #[test]
    fn weekly_with_several_weekdays_matches_every_one_of_them() {
        let dates = dates_in_range(
            &RecurrenceRule::Weekly {
                weekdays: vec![Weekday::Mon, Weekday::Thu],
            },
            date(2026, 8, 1),
            None,
            date(2026, 8, 1),
            date(2026, 8, 7),
        );

        // 2026-08-03 is a Monday, 2026-08-06 is a Thursday.
        assert_eq!(dates, vec![date(2026, 8, 3), date(2026, 8, 6)]);
    }

    // -- dates_in_range: monthly -------------------------------------------

    #[test]
    fn monthly_matches_the_same_day_number_every_month() {
        let dates = dates_in_range(
            &RecurrenceRule::Monthly { day_of_month: 15 },
            date(2026, 1, 1),
            None,
            date(2026, 1, 1),
            date(2026, 4, 30),
        );

        assert_eq!(
            dates,
            vec![
                date(2026, 1, 15),
                date(2026, 2, 15),
                date(2026, 3, 15),
                date(2026, 4, 15),
            ]
        );
    }

    /// 2026 is not a leap year: February has 28 days.
    #[test]
    fn monthly_on_the_31st_clamps_to_the_last_day_of_a_shorter_month() {
        let dates = dates_in_range(
            &RecurrenceRule::Monthly { day_of_month: 31 },
            date(2026, 1, 1),
            None,
            date(2026, 1, 1),
            date(2026, 4, 30),
        );

        assert_eq!(
            dates,
            vec![
                date(2026, 1, 31),
                date(2026, 2, 28),
                date(2026, 3, 31),
                date(2026, 4, 30),
            ]
        );
    }

    #[test]
    fn monthly_on_the_31st_lands_on_the_29th_of_a_leap_february() {
        // 2028 is a leap year.
        let dates = dates_in_range(
            &RecurrenceRule::Monthly { day_of_month: 31 },
            date(2028, 2, 1),
            None,
            date(2028, 2, 1),
            date(2028, 2, 29),
        );

        assert_eq!(dates, vec![date(2028, 2, 29)]);
    }

    // -- expand_occurrences / DST --------------------------------------------

    /// Europe/Paris springs forward on the last Sunday of March: 2026-03-29,
    /// 02:00 becomes 03:00. A daily 9am recurrence sits well clear of the
    /// gap, and this is the baseline the next two tests contrast with: the
    /// wall-clock hour must read 9am local on both sides of the change,
    /// which only holds if the UTC offset itself shifted by an hour.
    #[test]
    fn a_daily_9am_recurrence_keeps_the_same_wall_clock_time_across_a_dst_change() {
        let recurrence = recurrence(RecurrenceRule::Daily, date(2026, 3, 28), None);

        let occurrences = expand_occurrences(&recurrence, date(2026, 3, 28), date(2026, 3, 30));

        assert_eq!(occurrences.len(), 3);
        for occurrence in &occurrences {
            let local = occurrence.starts_at.with_timezone(&Europe::Paris);
            assert_eq!(local.time(), NaiveTime::from_hms_opt(9, 0, 0).unwrap());
        }
        // Before the change (CET, UTC+1) 9am local is 8am UTC; after it
        // (CEST, UTC+2) the same 9am local is 7am UTC — the offset moved,
        // proving this is not just comparing two identical instants.
        assert_eq!(occurrences[0].starts_at.hour(), 8, "before the change: CET");
        assert_eq!(occurrences[2].starts_at.hour(), 7, "after the change: CEST");
    }

    /// The spring-forward gap itself: 2026-03-29 02:30 local never happens in
    /// Europe/Paris. A daily recurrence at that wall-clock time must still
    /// produce something for that date rather than silently dropping it.
    #[test]
    fn a_recurrence_at_the_spring_forward_gap_still_produces_an_occurrence() {
        let mut recurrence = recurrence(RecurrenceRule::Daily, date(2026, 3, 28), None);
        recurrence.start_time = NaiveTime::from_hms_opt(2, 30, 0).unwrap();

        let occurrences = expand_occurrences(&recurrence, date(2026, 3, 29), date(2026, 3, 29));

        assert_eq!(occurrences.len(), 1, "the gap day is not silently dropped");
    }

    /// The fall-back hour: 2026-10-25 02:30 local happens twice in
    /// Europe/Paris (once at CEST, once at CET). The occurrence resolves to
    /// the earlier of the two instants.
    #[test]
    fn a_recurrence_at_the_fall_back_hour_resolves_to_the_earlier_instant() {
        let mut recurrence = recurrence(RecurrenceRule::Daily, date(2026, 10, 24), None);
        recurrence.start_time = NaiveTime::from_hms_opt(2, 30, 0).unwrap();

        let occurrences = expand_occurrences(&recurrence, date(2026, 10, 25), date(2026, 10, 25));

        assert_eq!(occurrences.len(), 1);
        // CEST (UTC+2) is the earlier of the two 2:30am local instants that
        // day, so this must be 00:30 UTC, not 01:30 UTC (the later, CET one).
        assert_eq!(occurrences[0].starts_at.hour(), 0);
        assert_eq!(occurrences[0].starts_at.minute(), 30);
    }

    #[test]
    fn occurrence_ends_at_is_starts_at_plus_the_duration() {
        let mut recurrence = recurrence(RecurrenceRule::Daily, date(2026, 8, 1), None);
        recurrence.duration_minutes = 90;

        let occurrences = expand_occurrences(&recurrence, date(2026, 8, 1), date(2026, 8, 1));

        assert_eq!(
            occurrences[0].ends_at - occurrences[0].starts_at,
            Duration::minutes(90)
        );
    }

    // -- target_horizon -------------------------------------------------------

    #[test]
    fn target_horizon_is_horizon_days_out_from_today() {
        let now = Europe::Paris
            .with_ymd_and_hms(2026, 8, 1, 12, 0, 0)
            .unwrap()
            .with_timezone(&Utc);

        let horizon = target_horizon(now, Europe::Paris, date(2026, 1, 1), None, 60);

        assert_eq!(horizon, date(2026, 9, 30));
    }

    #[test]
    fn target_horizon_never_goes_before_starts_on() {
        let now = Europe::Paris
            .with_ymd_and_hms(2026, 1, 1, 12, 0, 0)
            .unwrap()
            .with_timezone(&Utc);

        let horizon = target_horizon(now, Europe::Paris, date(2026, 8, 1), None, 60);

        assert_eq!(horizon, date(2026, 9, 30));
    }

    #[test]
    fn target_horizon_never_goes_past_ends_on() {
        let now = Europe::Paris
            .with_ymd_and_hms(2026, 8, 1, 12, 0, 0)
            .unwrap()
            .with_timezone(&Utc);

        let horizon = target_horizon(
            now,
            Europe::Paris,
            date(2026, 1, 1),
            Some(date(2026, 8, 15)),
            60,
        );

        assert_eq!(horizon, date(2026, 8, 15));
    }

    // -- validate_rule ----------------------------------------------------

    #[test]
    fn a_weekly_rule_with_no_weekday_is_rejected() {
        let err = validate_rule(&RecurrenceRule::Weekly { weekdays: vec![] }).unwrap_err();
        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[test]
    fn a_monthly_rule_with_an_out_of_range_day_is_rejected() {
        let err = validate_rule(&RecurrenceRule::Monthly { day_of_month: 0 }).unwrap_err();
        assert!(matches!(err, CoreError::Conflict(_)));

        let err = validate_rule(&RecurrenceRule::Monthly { day_of_month: 32 }).unwrap_err();
        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[test]
    fn a_daily_rule_is_always_valid() {
        assert!(validate_rule(&RecurrenceRule::Daily).is_ok());
    }

    // -- TaskRecurrenceService::extend_organization_horizons -----------------

    mod extend_organization_horizons_tests {
        use super::*;
        use crate::domain::member::ports::MockMemberRepository;
        use crate::domain::task::ports::MockTaskRepository;
        use crate::domain::task_recurrence::ports::MockTaskRecurrenceRepository;

        fn service(
            recurrence_repository: MockTaskRecurrenceRepository,
        ) -> TaskRecurrenceService<
            MockTaskRecurrenceRepository,
            MockTaskRepository,
            MockMemberRepository,
        > {
            TaskRecurrenceService::new(
                recurrence_repository,
                MockTaskRepository::new(),
                MockMemberRepository::new(),
            )
        }

        #[tokio::test]
        async fn a_recurrence_whose_ends_on_has_passed_is_skipped_entirely() {
            let organization_id = OrganizationId(Uuid::new_v4());
            let today = Utc::now().date_naive();
            let finished = TaskRecurrence {
                ends_on: Some(today - Duration::days(1)),
                horizon_filled_to: today - Duration::days(1),
                ..recurrence(RecurrenceRule::Daily, today - Duration::days(100), None)
            };

            let mut recurrence_repository = MockTaskRecurrenceRepository::new();
            recurrence_repository
                .expect_horizon_days_for_organization()
                .returning(|_| Box::pin(async { Ok(None) }));
            recurrence_repository
                .expect_list_by_organization()
                .returning(move |_| {
                    let finished = finished.clone();
                    Box::pin(async move { Ok(vec![finished]) })
                });
            // No `expect_advance_horizon`: mockall panics on the unexpected
            // call, which is exactly the assertion that a finished series
            // is never visited.

            let mut service = service(recurrence_repository);

            let materialized = service
                .extend_organization_horizons(organization_id)
                .await
                .unwrap();

            assert_eq!(materialized, 0);
        }

        #[tokio::test]
        async fn a_recurrence_already_filled_past_the_target_is_skipped() {
            let organization_id = OrganizationId(Uuid::new_v4());
            let today = Utc::now().date_naive();
            // Filled far beyond even a generous horizon: nothing to do.
            let already_filled = TaskRecurrence {
                horizon_filled_to: today + Duration::days(400),
                ..recurrence(RecurrenceRule::Daily, today - Duration::days(10), None)
            };

            let mut recurrence_repository = MockTaskRecurrenceRepository::new();
            recurrence_repository
                .expect_horizon_days_for_organization()
                .returning(|_| Box::pin(async { Ok(None) }));
            recurrence_repository
                .expect_list_by_organization()
                .returning(move |_| {
                    let already_filled = already_filled.clone();
                    Box::pin(async move { Ok(vec![already_filled]) })
                });
            // No `expect_advance_horizon`: an already-filled recurrence must
            // never move its own watermark backward or redundantly forward.

            let mut service = service(recurrence_repository);

            let materialized = service
                .extend_organization_horizons(organization_id)
                .await
                .unwrap();

            assert_eq!(materialized, 0);
        }

        /// The organization-level setting is honored, not just read: with a
        /// configured 10-day override, a recurrence filled only up to today
        /// is pushed exactly 10 days forward — proven by asserting on the
        /// horizon `advance_horizon` is actually called with, not merely on
        /// the returned count.
        #[tokio::test]
        async fn a_configured_horizon_override_changes_the_target() {
            let organization_id = OrganizationId(Uuid::new_v4());
            let today = Utc::now().date_naive();
            let due = TaskRecurrence {
                horizon_filled_to: today,
                ..recurrence(RecurrenceRule::Daily, today - Duration::days(10), None)
            };
            let due_id = due.id;

            let mut recurrence_repository = MockTaskRecurrenceRepository::new();
            recurrence_repository
                .expect_horizon_days_for_organization()
                .returning(|_| Box::pin(async { Ok(Some(10)) }));
            recurrence_repository
                .expect_list_by_organization()
                .returning(move |_| {
                    let due = due.clone();
                    Box::pin(async move { Ok(vec![due]) })
                });
            recurrence_repository
                .expect_advance_horizon()
                .withf(move |id, horizon_filled_to| {
                    *id == due_id && *horizon_filled_to == today + Duration::days(10)
                })
                .returning(|_, _| Box::pin(async { Ok(()) }));

            // Ten days of gap (`horizon_filled_to + 1` through the new
            // target, inclusive): one `insert_occurrence_if_absent` call per
            // date, every one reporting a fresh insert.
            let mut task_repository = MockTaskRepository::new();
            task_repository
                .expect_insert_occurrence_if_absent()
                .times(10)
                .returning(|_| Box::pin(async { Ok(true) }));

            let mut service = TaskRecurrenceService::new(
                recurrence_repository,
                task_repository,
                MockMemberRepository::new(),
            );

            let materialized = service
                .extend_organization_horizons(organization_id)
                .await
                .unwrap();

            assert_eq!(materialized, 10);
        }
    }
}
