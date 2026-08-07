use std::collections::BTreeMap;

use chrono::{Datelike, NaiveDate, Utc};
use common::{CoreError, generate_uuid_v7};

use crate::{
    EmployeeId, EmployeeRhythm, EmployeeRhythmId, EmployeeWorkSlot, EmployeeWorkSlotId, RhythmSlot,
    RhythmSlotId,
    domain::work_time::{
        DateRange, MinuteInterval, Tz,
        commands::{ReplaceRhythmCommand, ReplaceWorkSlotsCommand, RhythmSlotInput, WorkSlotInput},
        ports::{RhythmRepository, WorkSlotRepository},
    },
};

/// Expands the two sources of truth — the recurring weekly rhythm and the
/// dated work slots — into the concrete minute intervals worked on each day
/// of `range`.
///
/// For a given day: if `work_slots` has entries for that date, they win
/// outright; otherwise the rhythm version in effect on that date supplies
/// its slots for that weekday; otherwise the day carries no entry at all —
/// omitted from the map, never an empty `Vec`, so callers don't have to
/// distinguish "no data" from "a rhythm with a zero-length day."
///
/// Pure and I/O-free by design: the API's reading window is capped at 92
/// days, so the work this function does is bounded by construction and
/// nothing needs to be materialized ahead of time in the database (see the
/// planning module design doc).
pub fn expand_work_slots(
    rhythms: &[EmployeeRhythm],
    work_slots: &[EmployeeWorkSlot],
    range: DateRange,
    _tz: Tz,
) -> BTreeMap<NaiveDate, Vec<MinuteInterval>> {
    let mut expanded = BTreeMap::new();
    let mut date = range.from;

    loop {
        let dated_intervals: Vec<MinuteInterval> = work_slots
            .iter()
            .filter(|slot| slot.work_date == date)
            .map(|slot| MinuteInterval {
                starts_minute: slot.starts_minute,
                ends_minute: slot.ends_minute,
            })
            .collect();

        if !dated_intervals.is_empty() {
            expanded.insert(date, dated_intervals);
        } else if let Some(intervals) = rhythm_intervals_for(rhythms, date) {
            expanded.insert(date, intervals);
        }

        if date == range.to {
            break;
        }
        date = date
            .succ_opt()
            .expect("range.to is reached before NaiveDate's representable range overflows");
    }

    expanded
}

/// The rhythm version in effect on `date`, if any, and its slots for that
/// weekday. Returns `None` — rather than `Some(vec![])` — both when no
/// version covers `date` and when the covering version has no slot for that
/// weekday, so the caller has a single "nothing to show" case to handle.
fn rhythm_intervals_for(
    rhythms: &[EmployeeRhythm],
    date: NaiveDate,
) -> Option<Vec<MinuteInterval>> {
    let rhythm = rhythms.iter().find(|rhythm| {
        rhythm.effective_from <= date && rhythm.effective_to.is_none_or(|to| to > date)
    })?;

    let weekday = date.weekday().number_from_monday() as i16;
    let intervals: Vec<MinuteInterval> = rhythm
        .slots
        .iter()
        .filter(|slot| slot.weekday == weekday)
        .map(|slot| MinuteInterval {
            starts_minute: slot.starts_minute,
            ends_minute: slot.ends_minute,
        })
        .collect();

    if intervals.is_empty() {
        None
    } else {
        Some(intervals)
    }
}

fn validate_slot_minutes(starts_minute: i16, ends_minute: i16) -> Result<(), CoreError> {
    if !(0..=1439).contains(&starts_minute) {
        return Err(CoreError::Conflict(
            "slot starts_minute must be between 0 and 1439".to_owned(),
        ));
    }
    if !(1..=1440).contains(&ends_minute) {
        return Err(CoreError::Conflict(
            "slot ends_minute must be between 1 and 1440".to_owned(),
        ));
    }
    if ends_minute <= starts_minute {
        return Err(CoreError::Conflict(
            "slot ends_minute must be after starts_minute".to_owned(),
        ));
    }

    Ok(())
}

fn validate_weekday(weekday: i16) -> Result<(), CoreError> {
    if !(1..=7).contains(&weekday) {
        return Err(CoreError::Conflict(
            "slot weekday must be between 1 and 7 (ISO, Monday = 1)".to_owned(),
        ));
    }

    Ok(())
}

fn build_rhythm_slots(rhythm_id: EmployeeRhythmId, inputs: &[RhythmSlotInput]) -> Vec<RhythmSlot> {
    inputs
        .iter()
        .map(|input| RhythmSlot {
            id: RhythmSlotId(generate_uuid_v7()),
            rhythm_id,
            weekday: input.weekday,
            starts_minute: input.starts_minute,
            ends_minute: input.ends_minute,
        })
        .collect()
}

/// Orchestrates the rhythm and work-slot repositories behind the
/// `work-time` use cases: reading a window, and the two full-replace `PUT`s.
pub struct WorkTimeService<RR, WR>
where
    RR: RhythmRepository,
    WR: WorkSlotRepository,
{
    rhythm_repository: RR,
    work_slot_repository: WR,
}

impl<RR, WR> WorkTimeService<RR, WR>
where
    RR: RhythmRepository,
    WR: WorkSlotRepository,
{
    pub fn new(rhythm_repository: RR, work_slot_repository: WR) -> Self {
        Self {
            rhythm_repository,
            work_slot_repository,
        }
    }

    pub async fn get_work_time(
        &mut self,
        employee_id: EmployeeId,
        range: DateRange,
    ) -> Result<(Vec<EmployeeRhythm>, Vec<EmployeeWorkSlot>), CoreError> {
        let rhythms = self
            .rhythm_repository
            .find_overlapping(employee_id, range.from, range.to)
            .await?;
        let work_slots = self
            .work_slot_repository
            .list_in_window(employee_id, range.from, range.to)
            .await?;

        Ok((rhythms, work_slots))
    }

    /// Replaces the rhythm currently in effect for `command.employee_id`
    /// wholesale.
    ///
    /// - No open version exists yet: the command becomes the employee's
    ///   first rhythm version.
    /// - An open version exists and shares `command.effective_from`: it is
    ///   the same version being edited — its own fields and its complete
    ///   slot list are replaced in place, so calling this twice in a row
    ///   never accumulates a second row.
    /// - An open version exists and `command.effective_from` is later: a
    ///   genuinely new version starts, so the open one is closed
    ///   (`effective_to` set to the new version's start) rather than
    ///   overwritten — the old slots stay in the database as history.
    /// - An open version exists and `command.effective_from` is earlier:
    ///   rejected — this endpoint has no way to reconcile a version that
    ///   would start before the one currently in effect.
    pub async fn replace_rhythm(
        &mut self,
        command: ReplaceRhythmCommand,
    ) -> Result<EmployeeRhythm, CoreError> {
        if let Some(effective_to) = command.effective_to
            && effective_to <= command.effective_from
        {
            return Err(CoreError::Conflict(
                "rhythm effective_to must be after effective_from".to_owned(),
            ));
        }
        for slot in &command.slots {
            validate_weekday(slot.weekday)?;
            validate_slot_minutes(slot.starts_minute, slot.ends_minute)?;
        }

        let now = Utc::now();
        match self
            .rhythm_repository
            .find_open_by_employee(command.employee_id)
            .await?
        {
            None => {
                let id = EmployeeRhythmId(generate_uuid_v7());
                let rhythm = EmployeeRhythm {
                    id,
                    organization_id: command.organization_id,
                    employee_id: command.employee_id,
                    effective_from: command.effective_from,
                    effective_to: command.effective_to,
                    slots: build_rhythm_slots(id, &command.slots),
                    created_at: now,
                    updated_at: now,
                };
                self.rhythm_repository.insert(&rhythm).await
            }
            Some(mut open) if open.effective_from == command.effective_from => {
                open.effective_to = command.effective_to;
                open.slots = build_rhythm_slots(open.id, &command.slots);
                open.updated_at = now;
                self.rhythm_repository.update(&open).await
            }
            Some(open) if command.effective_from > open.effective_from => {
                self.rhythm_repository
                    .set_effective_to(open.id, command.effective_from)
                    .await?;

                let id = EmployeeRhythmId(generate_uuid_v7());
                let rhythm = EmployeeRhythm {
                    id,
                    organization_id: command.organization_id,
                    employee_id: command.employee_id,
                    effective_from: command.effective_from,
                    effective_to: command.effective_to,
                    slots: build_rhythm_slots(id, &command.slots),
                    created_at: now,
                    updated_at: now,
                };
                self.rhythm_repository.insert(&rhythm).await
            }
            Some(_) => Err(CoreError::Conflict(
                "cannot start a rhythm version before the one currently in effect".to_owned(),
            )),
        }
    }

    /// Replaces the `[command.from, command.to]` work-slot window wholesale.
    pub async fn replace_work_slots(
        &mut self,
        command: ReplaceWorkSlotsCommand,
    ) -> Result<Vec<EmployeeWorkSlot>, CoreError> {
        if command.to < command.from {
            return Err(CoreError::Conflict(
                "work slots window `to` must not be before `from`".to_owned(),
            ));
        }
        for slot in &command.slots {
            validate_slot_minutes(slot.starts_minute, slot.ends_minute)?;
            if slot.work_date < command.from || slot.work_date > command.to {
                return Err(CoreError::Conflict(format!(
                    "work slot date {} falls outside the replaced window [{}, {}]",
                    slot.work_date, command.from, command.to
                )));
            }
        }

        let slots: Vec<EmployeeWorkSlot> = command
            .slots
            .iter()
            .map(|input: &WorkSlotInput| EmployeeWorkSlot {
                id: EmployeeWorkSlotId(generate_uuid_v7()),
                organization_id: command.organization_id,
                employee_id: command.employee_id,
                work_date: input.work_date,
                starts_minute: input.starts_minute,
                ends_minute: input.ends_minute,
            })
            .collect();

        self.work_slot_repository
            .replace_window(
                command.organization_id,
                command.employee_id,
                command.from,
                command.to,
                &slots,
            )
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        OrganizationId,
        domain::work_time::ports::{MockRhythmRepository, MockWorkSlotRepository},
    };
    use mockall::predicate::eq;
    use uuid::Uuid;

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    fn tz() -> Tz {
        Tz("Europe/Paris".to_owned())
    }

    fn work_slot(
        employee_id: EmployeeId,
        work_date: NaiveDate,
        starts: i16,
        ends: i16,
    ) -> EmployeeWorkSlot {
        EmployeeWorkSlot {
            id: EmployeeWorkSlotId(Uuid::new_v4()),
            organization_id: OrganizationId(Uuid::new_v4()),
            employee_id,
            work_date,
            starts_minute: starts,
            ends_minute: ends,
        }
    }

    fn rhythm_slot(
        rhythm_id: EmployeeRhythmId,
        weekday: i16,
        starts: i16,
        ends: i16,
    ) -> RhythmSlot {
        RhythmSlot {
            id: RhythmSlotId(Uuid::new_v4()),
            rhythm_id,
            weekday,
            starts_minute: starts,
            ends_minute: ends,
        }
    }

    fn rhythm(
        employee_id: EmployeeId,
        effective_from: NaiveDate,
        effective_to: Option<NaiveDate>,
        slots: Vec<RhythmSlot>,
    ) -> EmployeeRhythm {
        let now = Utc::now();
        EmployeeRhythm {
            id: slots
                .first()
                .map(|s| s.rhythm_id)
                .unwrap_or(EmployeeRhythmId(Uuid::new_v4())),
            organization_id: OrganizationId(Uuid::new_v4()),
            employee_id,
            effective_from,
            effective_to,
            slots,
            created_at: now,
            updated_at: now,
        }
    }

    // -- expand_work_slots -----------------------------------------------

    #[test]
    fn a_dated_work_slot_wins_over_the_rhythm_for_that_day() {
        let employee_id = EmployeeId(Uuid::new_v4());
        // Monday 2026-08-10.
        let monday = date(2026, 8, 10);
        let rhythm_id = EmployeeRhythmId(Uuid::new_v4());
        let rhythms = vec![rhythm(
            employee_id,
            date(2026, 1, 1),
            None,
            vec![rhythm_slot(rhythm_id, 1, 480, 720)],
        )];
        let work_slots = vec![work_slot(employee_id, monday, 540, 600)];
        let range = DateRange::new(monday, monday).unwrap();

        let expanded = expand_work_slots(&rhythms, &work_slots, range, tz());

        assert_eq!(
            expanded.get(&monday),
            Some(&vec![MinuteInterval {
                starts_minute: 540,
                ends_minute: 600
            }])
        );
    }

    #[test]
    fn the_rhythm_is_used_when_no_dated_slot_exists() {
        let employee_id = EmployeeId(Uuid::new_v4());
        let monday = date(2026, 8, 10);
        let rhythm_id = EmployeeRhythmId(Uuid::new_v4());
        let rhythms = vec![rhythm(
            employee_id,
            date(2026, 1, 1),
            None,
            vec![rhythm_slot(rhythm_id, 1, 480, 720)],
        )];
        let range = DateRange::new(monday, monday).unwrap();

        let expanded = expand_work_slots(&rhythms, &[], range, tz());

        assert_eq!(
            expanded.get(&monday),
            Some(&vec![MinuteInterval {
                starts_minute: 480,
                ends_minute: 720
            }])
        );
    }

    #[test]
    fn a_day_with_no_applicable_rhythm_and_no_dated_slot_has_no_entry() {
        let monday = date(2026, 8, 10);
        let range = DateRange::new(monday, monday).unwrap();

        let expanded = expand_work_slots(&[], &[], range, tz());

        assert!(!expanded.contains_key(&monday));
        assert!(expanded.is_empty());
    }

    #[test]
    fn switching_rhythm_versions_mid_window_uses_each_versions_own_slots() {
        let employee_id = EmployeeId(Uuid::new_v4());
        // Two consecutive Mondays, one week apart, each governed by a
        // different rhythm version.
        let first_monday = date(2026, 8, 3);
        let second_monday = date(2026, 8, 10);

        let old_rhythm_id = EmployeeRhythmId(Uuid::new_v4());
        let new_rhythm_id = EmployeeRhythmId(Uuid::new_v4());
        let rhythms = vec![
            rhythm(
                employee_id,
                date(2026, 1, 1),
                Some(date(2026, 8, 7)),
                vec![rhythm_slot(old_rhythm_id, 1, 480, 720)],
            ),
            rhythm(
                employee_id,
                date(2026, 8, 7),
                None,
                vec![rhythm_slot(new_rhythm_id, 1, 540, 1020)],
            ),
        ];
        let range = DateRange::new(first_monday, second_monday).unwrap();

        let expanded = expand_work_slots(&rhythms, &[], range, tz());

        assert_eq!(
            expanded.get(&first_monday),
            Some(&vec![MinuteInterval {
                starts_minute: 480,
                ends_minute: 720
            }])
        );
        assert_eq!(
            expanded.get(&second_monday),
            Some(&vec![MinuteInterval {
                starts_minute: 540,
                ends_minute: 1020
            }])
        );
    }

    #[test]
    fn a_rhythm_with_no_effective_to_stays_in_effect_indefinitely() {
        let employee_id = EmployeeId(Uuid::new_v4());
        let far_future_monday = date(2030, 1, 7);
        let rhythm_id = EmployeeRhythmId(Uuid::new_v4());
        let rhythms = vec![rhythm(
            employee_id,
            date(2026, 1, 1),
            None,
            vec![rhythm_slot(rhythm_id, 1, 480, 720)],
        )];
        let range = DateRange::new(far_future_monday, far_future_monday).unwrap();

        let expanded = expand_work_slots(&rhythms, &[], range, tz());

        assert!(expanded.contains_key(&far_future_monday));
    }

    #[test]
    fn a_weekday_with_no_slot_in_the_rhythm_has_no_entry() {
        let employee_id = EmployeeId(Uuid::new_v4());
        // Sunday 2026-08-09: the rhythm only defines Monday.
        let sunday = date(2026, 8, 9);
        let rhythm_id = EmployeeRhythmId(Uuid::new_v4());
        let rhythms = vec![rhythm(
            employee_id,
            date(2026, 1, 1),
            None,
            vec![rhythm_slot(rhythm_id, 1, 480, 720)],
        )];
        let range = DateRange::new(sunday, sunday).unwrap();

        let expanded = expand_work_slots(&rhythms, &[], range, tz());

        assert!(!expanded.contains_key(&sunday));
    }

    #[test]
    fn several_slots_the_same_day_are_all_returned() {
        let employee_id = EmployeeId(Uuid::new_v4());
        let monday = date(2026, 8, 10);
        let rhythm_id = EmployeeRhythmId(Uuid::new_v4());
        let rhythms = vec![rhythm(
            employee_id,
            date(2026, 1, 1),
            None,
            vec![
                rhythm_slot(rhythm_id, 1, 480, 720),
                rhythm_slot(rhythm_id, 1, 780, 1020),
            ],
        )];
        let range = DateRange::new(monday, monday).unwrap();

        let expanded = expand_work_slots(&rhythms, &[], range, tz());

        let intervals = expanded.get(&monday).unwrap();
        assert_eq!(intervals.len(), 2);
        assert!(intervals.contains(&MinuteInterval {
            starts_minute: 480,
            ends_minute: 720
        }));
        assert!(intervals.contains(&MinuteInterval {
            starts_minute: 780,
            ends_minute: 1020
        }));
    }

    #[test]
    fn several_dated_slots_the_same_day_are_all_returned() {
        let employee_id = EmployeeId(Uuid::new_v4());
        let monday = date(2026, 8, 10);
        let work_slots = vec![
            work_slot(employee_id, monday, 480, 720),
            work_slot(employee_id, monday, 780, 1020),
        ];
        let range = DateRange::new(monday, monday).unwrap();

        let expanded = expand_work_slots(&[], &work_slots, range, tz());

        assert_eq!(expanded.get(&monday).unwrap().len(), 2);
    }

    #[test]
    fn both_bounds_of_the_window_are_included() {
        let employee_id = EmployeeId(Uuid::new_v4());
        let from = date(2026, 8, 3);
        let to = date(2026, 8, 5);
        let work_slots = vec![
            work_slot(employee_id, from, 480, 720),
            work_slot(employee_id, to, 480, 720),
        ];
        let range = DateRange::new(from, to).unwrap();

        let expanded = expand_work_slots(&[], &work_slots, range, tz());

        assert!(
            expanded.contains_key(&from),
            "the `from` bound must be included"
        );
        assert!(
            expanded.contains_key(&to),
            "the `to` bound must be included"
        );
        assert_eq!(expanded.len(), 2);
    }

    #[test]
    fn dated_work_slots_outside_the_window_are_ignored() {
        let employee_id = EmployeeId(Uuid::new_v4());
        let from = date(2026, 8, 3);
        let to = date(2026, 8, 5);
        let outside = date(2026, 8, 6);
        let work_slots = vec![work_slot(employee_id, outside, 480, 720)];
        let range = DateRange::new(from, to).unwrap();

        let expanded = expand_work_slots(&[], &work_slots, range, tz());

        assert!(expanded.is_empty());
    }

    #[test]
    fn a_work_slot_on_the_boundary_day_a_rhythm_version_closes_still_wins() {
        // The dated slot is authoritative regardless of which rhythm
        // version (if any) would otherwise apply that day.
        let employee_id = EmployeeId(Uuid::new_v4());
        let closing_day = date(2026, 8, 7);
        let old_rhythm_id = EmployeeRhythmId(Uuid::new_v4());
        let rhythms = vec![rhythm(
            employee_id,
            date(2026, 1, 1),
            Some(closing_day),
            vec![rhythm_slot(old_rhythm_id, 5, 480, 720)],
        )];
        let work_slots = vec![work_slot(employee_id, closing_day, 900, 960)];
        let range = DateRange::new(closing_day, closing_day).unwrap();

        let expanded = expand_work_slots(&rhythms, &work_slots, range, tz());

        assert_eq!(
            expanded.get(&closing_day),
            Some(&vec![MinuteInterval {
                starts_minute: 900,
                ends_minute: 960
            }])
        );
    }

    // -- WorkTimeService::replace_rhythm ----------------------------------

    fn service(
        rhythm_repository: MockRhythmRepository,
        work_slot_repository: MockWorkSlotRepository,
    ) -> WorkTimeService<MockRhythmRepository, MockWorkSlotRepository> {
        WorkTimeService::new(rhythm_repository, work_slot_repository)
    }

    fn replace_rhythm_command(employee_id: EmployeeId) -> ReplaceRhythmCommand {
        ReplaceRhythmCommand {
            organization_id: OrganizationId(Uuid::new_v4()),
            employee_id,
            effective_from: date(2026, 1, 1),
            effective_to: None,
            slots: vec![RhythmSlotInput {
                weekday: 1,
                starts_minute: 480,
                ends_minute: 720,
            }],
        }
    }

    #[tokio::test]
    async fn replace_rhythm_inserts_a_first_version_when_none_is_open() {
        let employee_id = EmployeeId(Uuid::new_v4());
        let mut rhythm_repository = MockRhythmRepository::new();
        rhythm_repository
            .expect_find_open_by_employee()
            .with(eq(employee_id))
            .returning(|_| Box::pin(async { Ok(None) }));
        rhythm_repository
            .expect_insert()
            .withf(|r| r.slots.len() == 1)
            .returning(|r| {
                let cloned = r.clone();
                Box::pin(async move { Ok(cloned) })
            });

        let mut service = service(rhythm_repository, MockWorkSlotRepository::new());

        let created = service
            .replace_rhythm(replace_rhythm_command(employee_id))
            .await
            .unwrap();

        assert_eq!(created.employee_id, employee_id);
        assert_eq!(created.slots.len(), 1);
    }

    #[tokio::test]
    async fn replace_rhythm_with_the_same_effective_from_replaces_in_place() {
        let employee_id = EmployeeId(Uuid::new_v4());
        let existing_id = EmployeeRhythmId(Uuid::new_v4());
        let existing = rhythm(
            employee_id,
            date(2026, 1, 1),
            None,
            vec![rhythm_slot(existing_id, 1, 480, 600)],
        );
        let mut existing = existing;
        existing.id = existing_id;

        let mut rhythm_repository = MockRhythmRepository::new();
        rhythm_repository
            .expect_find_open_by_employee()
            .with(eq(employee_id))
            .returning(move |_| {
                let existing = existing.clone();
                Box::pin(async move { Ok(Some(existing)) })
            });
        rhythm_repository
            .expect_update()
            .withf(move |r| {
                r.id == existing_id && r.slots.len() == 1 && r.slots[0].starts_minute == 480
            })
            .returning(|r| {
                let cloned = r.clone();
                Box::pin(async move { Ok(cloned) })
            });
        // No `expect_insert`: the same version must be edited in place, not
        // duplicated.

        let mut service = service(rhythm_repository, MockWorkSlotRepository::new());

        let updated = service
            .replace_rhythm(replace_rhythm_command(employee_id))
            .await
            .unwrap();

        assert_eq!(updated.id, existing_id);
    }

    #[tokio::test]
    async fn replace_rhythm_with_a_later_effective_from_closes_the_open_version_and_opens_a_new_one()
     {
        let employee_id = EmployeeId(Uuid::new_v4());
        let existing_id = EmployeeRhythmId(Uuid::new_v4());
        let mut existing = rhythm(employee_id, date(2026, 1, 1), None, Vec::new());
        existing.id = existing_id;

        let mut command = replace_rhythm_command(employee_id);
        command.effective_from = date(2026, 9, 1);

        let mut rhythm_repository = MockRhythmRepository::new();
        rhythm_repository
            .expect_find_open_by_employee()
            .with(eq(employee_id))
            .returning(move |_| {
                let existing = existing.clone();
                Box::pin(async move { Ok(Some(existing)) })
            });
        rhythm_repository
            .expect_set_effective_to()
            .withf(move |id, effective_to| *id == existing_id && *effective_to == date(2026, 9, 1))
            .returning(|_, _| Box::pin(async { Ok(()) }));
        rhythm_repository
            .expect_insert()
            .withf(|r| r.effective_from == date(2026, 9, 1))
            .returning(|r| {
                let cloned = r.clone();
                Box::pin(async move { Ok(cloned) })
            });

        let mut service = service(rhythm_repository, MockWorkSlotRepository::new());

        let created = service.replace_rhythm(command).await.unwrap();

        assert_eq!(created.effective_from, date(2026, 9, 1));
    }

    #[tokio::test]
    async fn replace_rhythm_rejects_an_effective_from_before_the_open_version() {
        let employee_id = EmployeeId(Uuid::new_v4());
        let existing = rhythm(employee_id, date(2026, 6, 1), None, Vec::new());

        let mut command = replace_rhythm_command(employee_id);
        command.effective_from = date(2026, 1, 1);

        let mut rhythm_repository = MockRhythmRepository::new();
        rhythm_repository
            .expect_find_open_by_employee()
            .with(eq(employee_id))
            .returning(move |_| {
                let existing = existing.clone();
                Box::pin(async move { Ok(Some(existing)) })
            });

        let mut service = service(rhythm_repository, MockWorkSlotRepository::new());

        let err = service.replace_rhythm(command).await.unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn replace_rhythm_rejects_an_out_of_range_weekday() {
        let employee_id = EmployeeId(Uuid::new_v4());
        let mut command = replace_rhythm_command(employee_id);
        command.slots = vec![RhythmSlotInput {
            weekday: 8,
            starts_minute: 480,
            ends_minute: 720,
        }];

        let mut service = service(MockRhythmRepository::new(), MockWorkSlotRepository::new());

        let err = service.replace_rhythm(command).await.unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn replace_rhythm_rejects_ends_minute_before_starts_minute() {
        let employee_id = EmployeeId(Uuid::new_v4());
        let mut command = replace_rhythm_command(employee_id);
        command.slots = vec![RhythmSlotInput {
            weekday: 1,
            starts_minute: 720,
            ends_minute: 480,
        }];

        let mut service = service(MockRhythmRepository::new(), MockWorkSlotRepository::new());

        let err = service.replace_rhythm(command).await.unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn replace_rhythm_rejects_effective_to_not_after_effective_from() {
        let employee_id = EmployeeId(Uuid::new_v4());
        let mut command = replace_rhythm_command(employee_id);
        command.effective_to = Some(command.effective_from);

        let mut service = service(MockRhythmRepository::new(), MockWorkSlotRepository::new());

        let err = service.replace_rhythm(command).await.unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    // -- WorkTimeService::replace_work_slots -------------------------------

    #[tokio::test]
    async fn replace_work_slots_delegates_the_full_replace_to_the_repository() {
        let employee_id = EmployeeId(Uuid::new_v4());
        let organization_id = OrganizationId(Uuid::new_v4());
        let from = date(2026, 8, 1);
        let to = date(2026, 8, 7);

        let mut work_slot_repository = MockWorkSlotRepository::new();
        work_slot_repository
            .expect_replace_window()
            .withf(move |org, emp, f, t, slots| {
                *org == organization_id
                    && *emp == employee_id
                    && *f == from
                    && *t == to
                    && slots.len() == 1
            })
            .returning(|_, _, _, _, slots| {
                let cloned = slots.to_vec();
                Box::pin(async move { Ok(cloned) })
            });

        let mut service = service(MockRhythmRepository::new(), work_slot_repository);

        let command = ReplaceWorkSlotsCommand {
            organization_id,
            employee_id,
            from,
            to,
            slots: vec![WorkSlotInput {
                work_date: date(2026, 8, 3),
                starts_minute: 480,
                ends_minute: 720,
            }],
        };

        let replaced = service.replace_work_slots(command).await.unwrap();

        assert_eq!(replaced.len(), 1);
    }

    #[tokio::test]
    async fn replace_work_slots_rejects_to_before_from() {
        let employee_id = EmployeeId(Uuid::new_v4());
        let mut service = service(MockRhythmRepository::new(), MockWorkSlotRepository::new());

        let command = ReplaceWorkSlotsCommand {
            organization_id: OrganizationId(Uuid::new_v4()),
            employee_id,
            from: date(2026, 8, 7),
            to: date(2026, 8, 1),
            slots: Vec::new(),
        };

        let err = service.replace_work_slots(command).await.unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn replace_work_slots_rejects_a_slot_dated_outside_the_window() {
        let employee_id = EmployeeId(Uuid::new_v4());
        let mut service = service(MockRhythmRepository::new(), MockWorkSlotRepository::new());

        let command = ReplaceWorkSlotsCommand {
            organization_id: OrganizationId(Uuid::new_v4()),
            employee_id,
            from: date(2026, 8, 1),
            to: date(2026, 8, 7),
            slots: vec![WorkSlotInput {
                work_date: date(2026, 8, 8),
                starts_minute: 480,
                ends_minute: 720,
            }],
        };

        let err = service.replace_work_slots(command).await.unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[tokio::test]
    async fn get_work_time_delegates_to_both_repositories() {
        let employee_id = EmployeeId(Uuid::new_v4());
        let from = date(2026, 8, 1);
        let to = date(2026, 8, 7);

        let mut rhythm_repository = MockRhythmRepository::new();
        rhythm_repository
            .expect_find_overlapping()
            .with(eq(employee_id), eq(from), eq(to))
            .returning(|_, _, _| Box::pin(async { Ok(Vec::new()) }));

        let mut work_slot_repository = MockWorkSlotRepository::new();
        work_slot_repository
            .expect_list_in_window()
            .with(eq(employee_id), eq(from), eq(to))
            .returning(|_, _, _| Box::pin(async { Ok(Vec::new()) }));

        let mut service = service(rhythm_repository, work_slot_repository);

        let (rhythms, work_slots) = service
            .get_work_time(employee_id, DateRange::new(from, to).unwrap())
            .await
            .unwrap();

        assert!(rhythms.is_empty());
        assert!(work_slots.is_empty());
    }
}
