use chrono::{NaiveDate, Utc};
use common::generate_uuid_v7;

use super::*;
use crate::{DayRefusal, LunchInput};

fn day() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 11, 9).unwrap()
}

fn declare(input: &DayOverrideInput) -> DayOverrideOutcome<DayOverride> {
    DayOverride::declare(
        DayOverrideId(generate_uuid_v7()),
        OrganizationId(generate_uuid_v7()),
        MemberId(generate_uuid_v7()),
        day(),
        input,
        Utc::now(),
    )
}

fn work(start: &str, end: &str) -> DayOverrideInput {
    DayOverrideInput {
        kind: OverrideKind::Work,
        start: Some(start.to_owned()),
        end: Some(end.to_owned()),
        lunch: None,
        note: String::new(),
    }
}

#[test]
fn spec_rh_5_2_a_work_override_with_end_before_start_is_refused() {
    assert_eq!(
        declare(&work("17:00", "08:00")),
        Err(DayOverrideError::Day(DayRefusal::EndBeforeStart))
    );
    assert_eq!(
        declare(&work("08:00", "08:00")),
        Err(DayOverrideError::Day(DayRefusal::EndBeforeStart))
    );
    assert_eq!(
        DayOverrideError::Day(DayRefusal::EndBeforeStart).field(),
        "end"
    );
}

#[test]
fn spec_rh_5_2_a_work_override_needs_both_times() {
    let mut input = work("08:00", "17:00");
    input.end = None;
    assert_eq!(declare(&input), Err(DayOverrideError::EndMissing));
    input.end = Some("17:00".to_owned());
    input.start = None;
    assert_eq!(declare(&input), Err(DayOverrideError::StartMissing));
}

#[test]
fn spec_rh_5_2_the_lunch_must_sit_inside_the_day() {
    let mut input = work("08:00", "12:00");
    input.lunch = Some(LunchInput {
        start: "11:00".to_owned(),
        end: "13:00".to_owned(),
    });
    assert_eq!(
        declare(&input),
        Err(DayOverrideError::Day(DayRefusal::LunchOutsideDay))
    );
}

#[test]
fn an_off_override_carries_no_hours() {
    let off = DayOverrideInput {
        kind: OverrideKind::Off,
        start: None,
        end: None,
        lunch: None,
        note: " Pont ".to_owned(),
    };
    let declared = declare(&off).unwrap();
    assert_eq!(declared.shape, DayShape::Off);
    assert_eq!(declared.note, "Pont");

    let mut with_hours = off;
    with_hours.start = Some("08:00".to_owned());
    assert_eq!(declare(&with_hours), Err(DayOverrideError::OffHasHours));
}

#[test]
fn a_valid_work_override_keeps_its_hours_and_lunch() {
    let mut input = work("08:00", "17:00");
    input.lunch = Some(LunchInput {
        start: "12:00".to_owned(),
        end: "13:00".to_owned(),
    });
    let declared = declare(&input).unwrap();
    let hours = declared.shape.work().unwrap();
    assert_eq!(hours.worked_minutes(), 8 * 60);
    assert_eq!(declared.shape.kind(), OverrideKind::Work);
}

#[test]
fn a_period_is_ordered_and_bounded() {
    let from = day();
    let inside = from + chrono::Duration::days(61);
    let outside = from + chrono::Duration::days(62);
    assert!(PlanningPeriod::new(from, inside, 62).is_ok());
    assert_eq!(
        PlanningPeriod::new(from, outside, 62),
        Err(PlanningPeriodError::TooLong(62))
    );
    assert_eq!(
        PlanningPeriod::new(from, from - chrono::Duration::days(1), 62),
        Err(PlanningPeriodError::Inverted)
    );
    assert!(PlanningPeriod::new(from, from, 62).is_ok());
}
