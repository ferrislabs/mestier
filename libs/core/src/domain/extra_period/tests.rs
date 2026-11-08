use chrono::{Duration, NaiveDate, Utc};
use common::generate_uuid_v7;

use super::*;
use crate::{DayRefusal, LunchInput, SurchargeInput, SurchargeKind};

fn member() -> MemberId {
    MemberId(generate_uuid_v7())
}

fn organization() -> OrganizationId {
    OrganizationId(generate_uuid_v7())
}

fn draft(member_id: MemberId) -> ExtraPeriodDraft {
    ExtraPeriodDraft {
        member_id,
        date: NaiveDate::from_ymd_opt(2026, 11, 14).unwrap(),
        start: "08:00".to_owned(),
        end: "12:00".to_owned(),
        lunch: None,
        reason: "Rattrapage du chantier".to_owned(),
        surcharge_id: None,
        rate_bp: None,
        approve: false,
    }
}

fn surcharge(rate_bp: i32) -> Surcharge {
    Surcharge::new(
        SurchargeId(generate_uuid_v7()),
        organization(),
        SurchargeInput {
            name: "Dimanche",
            rate_bp,
            kind: SurchargeKind::Sunday,
        },
        Utc::now(),
    )
    .unwrap()
}

fn declare(
    draft: &ExtraPeriodDraft,
    surcharge: Option<&Surcharge>,
) -> ExtraPeriodOutcome<ExtraPeriod> {
    ExtraPeriod::declare(
        ExtraPeriodId(generate_uuid_v7()),
        organization(),
        draft,
        surcharge,
        Some(member()),
        Utc::now(),
    )
}

#[test]
fn spec_rh_6_4_an_end_not_after_the_start_is_refused() {
    let mut equal = draft(member());
    equal.end = "08:00".to_owned();
    let mut before = draft(member());
    before.end = "07:00".to_owned();

    assert_eq!(
        declare(&equal, None),
        Err(ExtraPeriodError::Day(DayRefusal::EndBeforeStart))
    );
    assert_eq!(
        declare(&before, None),
        Err(ExtraPeriodError::Day(DayRefusal::EndBeforeStart))
    );
    assert_eq!(
        ExtraPeriodError::Day(DayRefusal::EndBeforeStart).field(),
        "end"
    );
}

#[test]
fn a_lunch_must_sit_inside_the_span() {
    let mut outside = draft(member());
    outside.lunch = Some(LunchInput {
        start: "07:00".to_owned(),
        end: "09:00".to_owned(),
    });

    assert_eq!(
        declare(&outside, None),
        Err(ExtraPeriodError::Day(DayRefusal::LunchOutsideDay))
    );
}

#[test]
fn a_blank_reason_is_refused() {
    let mut blank = draft(member());
    blank.reason = "   ".to_owned();

    assert_eq!(declare(&blank, None), Err(ExtraPeriodError::ReasonBlank));
}

#[test]
fn spec_rh_6_2_the_rate_is_copied_from_the_surcharge() {
    let mut surcharge = surcharge(10_000);
    let mut with_rate = draft(member());
    with_rate.surcharge_id = Some(surcharge.id);
    with_rate.rate_bp = Some(1);

    let period = declare(&with_rate, Some(&surcharge)).unwrap();
    surcharge
        .revise(SurchargeInput {
            name: "Dimanche",
            rate_bp: 5000,
            kind: SurchargeKind::Sunday,
        })
        .unwrap();

    assert_eq!(period.rate.get(), 10_000);
    assert_eq!(period.surcharge_id, Some(surcharge.id));
}

#[test]
fn a_free_rate_is_kept_and_defaults_to_zero() {
    let mut free = draft(member());
    free.rate_bp = Some(3000);
    let none = draft(member());
    let mut too_high = draft(member());
    too_high.rate_bp = Some(20_001);

    assert_eq!(declare(&free, None).unwrap().rate.get(), 3000);
    assert_eq!(declare(&none, None).unwrap().rate.get(), 0);
    assert_eq!(declare(&too_high, None), Err(ExtraPeriodError::RateInvalid));
}

#[test]
fn an_archived_surcharge_cannot_be_chosen() {
    let mut archived = surcharge(2500);
    archived.archive(Utc::now());
    let mut chosen = draft(member());
    chosen.surcharge_id = Some(archived.id);

    assert_eq!(
        declare(&chosen, Some(&archived)),
        Err(ExtraPeriodError::SurchargeArchived)
    );
}

#[test]
fn a_period_is_pending_unless_approved_at_creation() {
    let mut approved = draft(member());
    approved.approve = true;

    let pending = declare(&draft(member()), None).unwrap();
    let decided = declare(&approved, None).unwrap();

    assert_eq!(pending.status, ExtraStatus::Pending);
    assert!(pending.decided_at.is_none());
    assert_eq!(decided.status, ExtraStatus::Approved);
    assert_eq!(decided.decided_by_member_id, decided.created_by_member_id);
    assert!(decided.decided_at.is_some());
}

#[test]
fn spec_rh_6_3_a_period_is_decided_once_and_a_rejection_needs_a_comment() {
    let now = Utc::now();
    let decider = Some(member());

    let mut approved = declare(&draft(member()), None).unwrap();
    approved.approve(decider, now).unwrap();
    assert_eq!(
        approved.approve(decider, now),
        Err(ExtraPeriodError::AlreadyDecided)
    );
    assert_eq!(
        approved.reject("Non", decider, now),
        Err(ExtraPeriodError::AlreadyDecided)
    );

    let mut pending = declare(&draft(member()), None).unwrap();
    assert_eq!(
        pending.reject("  ", decider, now),
        Err(ExtraPeriodError::CommentRequired)
    );
    assert_eq!(pending.status, ExtraStatus::Pending);
    pending.reject(" Pas validé ", decider, now).unwrap();
    assert_eq!(pending.status, ExtraStatus::Rejected);
    assert_eq!(pending.decision_comment.as_deref(), Some("Pas validé"));
    assert_eq!(
        pending.cancel(decider, now),
        Err(ExtraPeriodError::AlreadyDecided)
    );
    assert_eq!(
        pending.approve(decider, now + Duration::seconds(1)),
        Err(ExtraPeriodError::AlreadyDecided)
    );
}

#[test]
fn an_approved_period_can_still_be_cancelled_once() {
    let now = Utc::now();
    let mut period = declare(&draft(member()), None).unwrap();
    period.approve(None, now).unwrap();

    period.cancel(None, now).unwrap();

    assert_eq!(period.status, ExtraStatus::Cancelled);
    assert_eq!(
        period.cancel(None, now),
        Err(ExtraPeriodError::AlreadyDecided)
    );
}
