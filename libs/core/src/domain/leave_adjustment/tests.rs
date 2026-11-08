use chrono::{NaiveDate, Utc};
use common::generate_uuid_v7;

use super::{
    AdjustmentKind, LeaveAdjustment, LeaveAdjustmentDraft, LeaveAdjustmentError, LeaveAdjustmentId,
};
use crate::{
    AcquisitionDraft, LeaveAccount, LeaveAccountId, LeaveAccountInput, MemberId, OrganizationId,
};

fn account() -> LeaveAccount {
    LeaveAccount::new(
        LeaveAccountId(generate_uuid_v7()),
        OrganizationId(generate_uuid_v7()),
        LeaveAccountInput {
            name: "Congés payés",
            acquisition: AcquisitionDraft::Monthly { per_month: 2.08 },
            period_start_month: 6,
            carry_over_max: 5.0,
        },
        Utc::now(),
    )
    .unwrap()
}

fn draft(
    account: &LeaveAccount,
    days: f64,
    kind: AdjustmentKind,
    reason: &str,
) -> LeaveAdjustmentDraft {
    LeaveAdjustmentDraft {
        member_id: MemberId(generate_uuid_v7()),
        account_id: account.id,
        date: NaiveDate::from_ymd_opt(2026, 7, 1).unwrap(),
        days,
        kind,
        reason: reason.to_owned(),
    }
}

fn record(
    account: &LeaveAccount,
    draft: &LeaveAdjustmentDraft,
) -> Result<LeaveAdjustment, LeaveAdjustmentError> {
    LeaveAdjustment::record(
        LeaveAdjustmentId(generate_uuid_v7()),
        account.organization_id,
        draft,
        account,
        None,
        Utc::now(),
    )
}

#[test]
fn spec_rh_4_2_an_adjustment_of_zero_days_is_refused() {
    let account = account();
    let zero = draft(&account, 0.0, AdjustmentKind::Adjustment, "Erreur");
    assert_eq!(
        record(&account, &zero),
        Err(LeaveAdjustmentError::DaysInvalid)
    );
    let nan = draft(&account, f64::NAN, AdjustmentKind::Adjustment, "Erreur");
    assert_eq!(
        record(&account, &nan),
        Err(LeaveAdjustmentError::DaysInvalid)
    );
    let fine = draft(&account, 1.234, AdjustmentKind::Adjustment, "Erreur");
    assert_eq!(
        record(&account, &fine),
        Err(LeaveAdjustmentError::DaysInvalid)
    );
}

#[test]
fn spec_rh_4_2_an_adjustment_without_a_reason_is_refused() {
    let account = account();
    let blank = draft(&account, -1.5, AdjustmentKind::Adjustment, "   ");
    assert_eq!(
        record(&account, &blank),
        Err(LeaveAdjustmentError::ReasonEmpty)
    );
}

#[test]
fn a_negative_adjustment_is_kept_signed_and_the_reason_trimmed() {
    let account = account();
    let adjustment = record(
        &account,
        &draft(
            &account,
            -1.5,
            AdjustmentKind::Adjustment,
            "  Erreur de saisie ",
        ),
    )
    .unwrap();
    assert_eq!(adjustment.days.days().hundredths(), -150);
    assert_eq!(adjustment.reason.as_str(), "Erreur de saisie");
}

#[test]
fn a_carry_over_is_positive_and_within_the_account_maximum() {
    let account = account();
    assert_eq!(
        record(
            &account,
            &draft(&account, -1.0, AdjustmentKind::CarryOver, "Report")
        ),
        Err(LeaveAdjustmentError::CarryOverNegative)
    );
    assert_eq!(
        record(
            &account,
            &draft(&account, 5.5, AdjustmentKind::CarryOver, "Report")
        ),
        Err(LeaveAdjustmentError::CarryOverTooLarge)
    );
    assert!(
        record(
            &account,
            &draft(&account, 5.0, AdjustmentKind::CarryOver, "Report")
        )
        .is_ok()
    );
}

#[test]
fn an_archived_account_refuses_new_adjustments() {
    let mut account = account();
    account.archive(Utc::now());
    assert_eq!(
        record(
            &account,
            &draft(&account, 1.0, AdjustmentKind::Adjustment, "Geste")
        ),
        Err(LeaveAdjustmentError::AccountArchived)
    );
}
