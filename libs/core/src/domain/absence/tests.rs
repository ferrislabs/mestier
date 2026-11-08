use chrono::{DateTime, NaiveDate, TimeZone, Utc};
use chrono_tz::Europe::Paris;
use common::generate_uuid_v7;
use std::str::FromStr;
use uuid::Uuid;

use super::{Absence, AbsenceError, AbsenceId, AbsenceKind, AbsenceSpan, AbsenceStatus, DayPart};
use crate::{AbsenceType, AbsenceTypeId, AbsenceTypeInput, CountMode, MemberId, OrganizationId};

fn date(year: i32, month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(year, month, day).unwrap()
}

fn utc(year: i32, month: u32, day: u32, hour: u32, minute: u32) -> DateTime<Utc> {
    Utc.with_ymd_and_hms(year, month, day, hour, minute, 0)
        .unwrap()
}

fn absence_type(needs_approval: bool, needs_proof: bool) -> AbsenceType {
    AbsenceType::new(
        AbsenceTypeId(generate_uuid_v7()),
        OrganizationId(generate_uuid_v7()),
        AbsenceTypeInput {
            name: "Congés payés",
            hue: 155,
            count_mode: CountMode::Working,
            account_id: None,
            paid: true,
            needs_approval,
            needs_proof,
            kind: AbsenceKind::Leave,
        },
        Utc::now(),
    )
    .unwrap()
}

fn requested(ty: &AbsenceType, proof: Option<&str>) -> Absence {
    let span = AbsenceSpan::new(
        date(2026, 7, 6),
        DayPart::Morning,
        date(2026, 7, 8),
        DayPart::Afternoon,
    )
    .unwrap();
    let (starts_at, ends_at) = span.window(Paris).unwrap();
    Absence::request(
        AbsenceId(generate_uuid_v7()),
        ty.organization_id,
        MemberId(generate_uuid_v7()),
        ty,
        span,
        (starts_at, ends_at),
        ty.needs_approval,
        None,
        proof,
        Some(MemberId(generate_uuid_v7())),
        Utc::now(),
    )
    .unwrap()
}

#[test]
fn spec_rh_3_8_a_full_day_pair_is_the_all_day_window_of_the_organization_timezone() {
    let span = AbsenceSpan::new(
        date(2026, 7, 6),
        DayPart::Morning,
        date(2026, 7, 6),
        DayPart::Afternoon,
    )
    .unwrap();

    let (starts_at, ends_at) = span.window(Paris).unwrap();

    assert_eq!(starts_at, utc(2026, 7, 5, 22, 0));
    assert_eq!(ends_at, utc(2026, 7, 6, 22, 0));
}

#[test]
fn spec_rh_3_8_half_days_split_the_day_at_noon_local_time() {
    let morning = AbsenceSpan::new(
        date(2026, 1, 12),
        DayPart::Morning,
        date(2026, 1, 12),
        DayPart::Morning,
    )
    .unwrap();
    let afternoon = AbsenceSpan::new(
        date(2026, 1, 12),
        DayPart::Afternoon,
        date(2026, 1, 12),
        DayPart::Afternoon,
    )
    .unwrap();

    assert_eq!(
        morning.window(Paris).unwrap(),
        (utc(2026, 1, 11, 23, 0), utc(2026, 1, 12, 11, 0))
    );
    assert_eq!(
        afternoon.window(Paris).unwrap(),
        (utc(2026, 1, 12, 11, 0), utc(2026, 1, 12, 23, 0))
    );
}

#[test]
fn spec_rh_3_8_a_multi_day_span_starts_after_noon_and_ends_at_noon() {
    let span = AbsenceSpan::new(
        date(2026, 1, 12),
        DayPart::Afternoon,
        date(2026, 1, 14),
        DayPart::Morning,
    )
    .unwrap();

    assert_eq!(
        span.window(Paris).unwrap(),
        (utc(2026, 1, 12, 11, 0), utc(2026, 1, 14, 11, 0))
    );
}

#[test]
fn an_end_before_the_start_is_refused() {
    assert_eq!(
        AbsenceSpan::new(
            date(2026, 1, 13),
            DayPart::Morning,
            date(2026, 1, 12),
            DayPart::Afternoon
        ),
        Err(AbsenceError::EndBeforeStart)
    );
    assert_eq!(
        AbsenceSpan::new(
            date(2026, 1, 12),
            DayPart::Afternoon,
            date(2026, 1, 12),
            DayPart::Morning
        ),
        Err(AbsenceError::EndBeforeStart)
    );
}

#[test]
fn a_timed_window_maps_back_to_dates_and_parts() {
    let span =
        AbsenceSpan::from_window(utc(2026, 1, 12, 8, 0), utc(2026, 1, 12, 9, 0), Paris).unwrap();
    assert_eq!(span.start_part(), DayPart::Morning);
    assert_eq!(span.end_part(), DayPart::Morning);

    let all_day =
        AbsenceSpan::from_window(utc(2026, 1, 11, 23, 0), utc(2026, 1, 12, 23, 0), Paris).unwrap();
    assert_eq!(all_day.start_date(), date(2026, 1, 12));
    assert_eq!(all_day.end_date(), date(2026, 1, 12));
    assert_eq!(all_day.start_part(), DayPart::Morning);
    assert_eq!(all_day.end_part(), DayPart::Afternoon);
}

#[test]
fn spec_rh_3_2_a_type_needing_approval_creates_pending_otherwise_approved() {
    let needing = requested(&absence_type(true, false), None);
    let free = {
        let ty = absence_type(false, false);
        requested(&ty, None)
    };

    assert_eq!(needing.status, AbsenceStatus::Pending);
    assert_eq!(free.status, AbsenceStatus::Approved);
    assert!(free.decided_by_member_id.is_none());
}

#[test]
fn spec_rh_3_6_an_archived_type_cannot_be_chosen_for_a_new_absence() {
    let mut ty = absence_type(true, false);
    ty.archive(Utc::now());
    let span = AbsenceSpan::new(
        date(2026, 7, 6),
        DayPart::Morning,
        date(2026, 7, 6),
        DayPart::Afternoon,
    )
    .unwrap();

    let outcome = Absence::request(
        AbsenceId(generate_uuid_v7()),
        ty.organization_id,
        MemberId(generate_uuid_v7()),
        &ty,
        span,
        (utc(2026, 7, 5, 22, 0), utc(2026, 7, 6, 22, 0)),
        true,
        None,
        None,
        None,
        Utc::now(),
    );

    assert_eq!(outcome, Err(AbsenceError::TypeArchived));
}

#[test]
fn spec_rh_3_3_approving_records_decider_and_date_and_a_second_decision_is_refused() {
    let ty = absence_type(true, false);
    let mut absence = requested(&ty, None);
    let decider = MemberId(generate_uuid_v7());
    let at = Utc::now();

    absence.approve(&ty, Some(decider), at).unwrap();

    assert_eq!(absence.status, AbsenceStatus::Approved);
    assert_eq!(absence.decided_by_member_id, Some(decider));
    assert_eq!(absence.decided_at, Some(at));
    assert_eq!(
        absence.approve(&ty, Some(decider), at),
        Err(AbsenceError::AlreadyDecided)
    );
    assert_eq!(
        absence.reject("non", Some(decider), at),
        Err(AbsenceError::AlreadyDecided)
    );
}

#[test]
fn spec_rh_3_4_rejecting_needs_a_comment() {
    let ty = absence_type(true, false);
    let mut absence = requested(&ty, None);

    assert_eq!(
        absence.reject("   ", None, Utc::now()),
        Err(AbsenceError::CommentRequired)
    );
    assert_eq!(absence.status, AbsenceStatus::Pending);

    absence
        .reject(" période chargée ", None, Utc::now())
        .unwrap();
    assert_eq!(absence.status, AbsenceStatus::Rejected);
    assert_eq!(absence.decision_comment.as_deref(), Some("période chargée"));
}

#[test]
fn spec_rh_3_5_approving_a_type_needing_proof_without_a_proof_file_is_refused() {
    let ty = absence_type(true, true);
    let mut without = requested(&ty, None);
    let mut with = requested(&ty, Some("absences/certificat.pdf"));

    assert_eq!(
        without.approve(&ty, None, Utc::now()),
        Err(AbsenceError::ProofRequired)
    );
    assert_eq!(without.status, AbsenceStatus::Pending);
    assert!(with.approve(&ty, None, Utc::now()).is_ok());
}

#[test]
fn cancelling_is_allowed_while_pending_or_approved_only() {
    let ty = absence_type(true, false);
    let mut pending = requested(&ty, None);
    let mut approved = requested(&ty, None);
    approved.approve(&ty, None, Utc::now()).unwrap();
    let mut rejected = requested(&ty, None);
    rejected.reject("non", None, Utc::now()).unwrap();

    assert!(pending.cancel(None, Utc::now()).is_ok());
    assert!(approved.cancel(None, Utc::now()).is_ok());
    assert_eq!(
        rejected.cancel(None, Utc::now()),
        Err(AbsenceError::AlreadyDecided)
    );
    assert_eq!(
        pending.cancel(None, Utc::now()),
        Err(AbsenceError::AlreadyDecided)
    );
}

#[test]
fn only_an_approved_absence_is_in_force() {
    assert!(AbsenceStatus::Approved.is_in_force());
    assert!(!AbsenceStatus::Pending.is_in_force());
    assert!(!AbsenceStatus::Rejected.is_in_force());
    assert!(!AbsenceStatus::Cancelled.is_in_force());
}

#[test]
fn an_unreadable_proof_key_is_refused() {
    let ty = absence_type(true, true);
    let span = AbsenceSpan::new(
        date(2026, 7, 6),
        DayPart::Morning,
        date(2026, 7, 6),
        DayPart::Afternoon,
    )
    .unwrap();

    let outcome = Absence::request(
        AbsenceId(generate_uuid_v7()),
        ty.organization_id,
        MemberId(generate_uuid_v7()),
        &ty,
        span,
        (utc(2026, 7, 5, 22, 0), utc(2026, 7, 6, 22, 0)),
        true,
        None,
        Some("../etc/passwd"),
        None,
        Utc::now(),
    );

    assert_eq!(outcome, Err(AbsenceError::ProofKeyInvalid));
}

#[test]
fn employee_absence_id_parses_uuid() {
    let uuid = Uuid::new_v4();
    let parsed = AbsenceId::from_str(&uuid.to_string()).unwrap();

    assert_eq!(parsed.0, uuid);
}

#[test]
fn employee_absence_id_rejects_invalid_uuid() {
    assert!(AbsenceId::from_str("not-a-uuid").is_err());
}

#[test]
fn absence_kind_parses_known_values() {
    assert_eq!("LEAVE".parse::<AbsenceKind>().unwrap(), AbsenceKind::Leave);
    assert_eq!("SICK".parse::<AbsenceKind>().unwrap(), AbsenceKind::Sick);
    assert_eq!(
        "UNAVAILABLE".parse::<AbsenceKind>().unwrap(),
        AbsenceKind::Unavailable
    );
}

#[test]
fn absence_kind_rejects_unknown_values() {
    assert!("HOLIDAY".parse::<AbsenceKind>().is_err());
}

#[test]
fn absence_kind_round_trips_through_as_str() {
    for kind in [
        AbsenceKind::Leave,
        AbsenceKind::Sick,
        AbsenceKind::Unavailable,
    ] {
        assert_eq!(kind.as_str().parse::<AbsenceKind>().unwrap(), kind);
    }
}
