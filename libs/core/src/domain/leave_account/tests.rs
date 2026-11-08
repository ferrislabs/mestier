use chrono::Utc;
use common::generate_uuid_v7;

use super::{
    AcquisitionDraft, LeaveAccount, LeaveAccountError, LeaveAccountId, LeaveAccountInput,
    LeaveDays, default_leave_accounts,
};
use crate::OrganizationId;

fn build(
    name: &str,
    acquisition: AcquisitionDraft,
    month: u8,
    carry: f64,
) -> Result<LeaveAccount, LeaveAccountError> {
    LeaveAccount::new(
        LeaveAccountId(generate_uuid_v7()),
        OrganizationId(generate_uuid_v7()),
        LeaveAccountInput {
            name,
            acquisition,
            period_start_month: month,
            carry_over_max: carry,
        },
        Utc::now(),
    )
}

const ANNUAL: AcquisitionDraft = AcquisitionDraft::Annual { days: 10.0 };

#[test]
fn a_blank_name_is_refused() {
    assert_eq!(
        build("  ", ANNUAL, 1, 0.0),
        Err(LeaveAccountError::NameEmpty)
    );
}

#[test]
fn the_period_start_month_stays_between_1_and_12() {
    assert_eq!(
        build("RTT", ANNUAL, 0, 0.0),
        Err(LeaveAccountError::PeriodStartMonthOutOfRange)
    );
    assert_eq!(
        build("RTT", ANNUAL, 13, 0.0),
        Err(LeaveAccountError::PeriodStartMonthOutOfRange)
    );
    assert!(build("RTT", ANNUAL, 12, 0.0).is_ok());
}

#[test]
fn the_acquisition_is_a_positive_number_of_days() {
    for draft in [
        AcquisitionDraft::Monthly { per_month: 0.0 },
        AcquisitionDraft::Monthly { per_month: -1.0 },
        AcquisitionDraft::Annual { days: 0.0 },
        AcquisitionDraft::Annual { days: 2.005 },
        AcquisitionDraft::Annual { days: f64::NAN },
    ] {
        assert_eq!(
            build("RTT", draft, 1, 0.0),
            Err(LeaveAccountError::AcquisitionInvalid)
        );
    }
}

#[test]
fn the_carry_over_maximum_is_never_negative() {
    assert_eq!(
        build("RTT", ANNUAL, 1, -0.5),
        Err(LeaveAccountError::CarryOverMaxInvalid)
    );
    assert!(build("RTT", ANNUAL, 1, 0.0).is_ok());
}

#[test]
fn decimals_are_stored_as_exact_hundredths() {
    assert_eq!(LeaveDays::from_decimal(2.08).unwrap().hundredths(), 208);
    assert_eq!(LeaveDays::from_decimal(0.1 + 0.2).unwrap().hundredths(), 30);
    assert_eq!(LeaveDays::from_decimal(2.5).unwrap().as_decimal(), 2.5);
    assert_eq!(LeaveDays::from_decimal(0.001), None);
}

#[test]
fn the_defaults_are_valid_accounts() {
    let names: Vec<&str> = default_leave_accounts().iter().map(|d| d.name).collect();
    assert_eq!(names, ["Congés payés", "RTT"]);
    for default in default_leave_accounts() {
        assert!(
            LeaveAccount::new(
                LeaveAccountId(generate_uuid_v7()),
                OrganizationId(generate_uuid_v7()),
                LeaveAccountInput::from(default),
                Utc::now(),
            )
            .is_ok()
        );
    }
}

#[test]
fn archiving_keeps_the_first_date_and_restoring_clears_it() {
    let mut account = build("RTT", ANNUAL, 1, 0.0).unwrap();
    let first = Utc::now();
    account.archive(first);
    account.archive(first + chrono::Duration::days(1));
    assert_eq!(account.archived_at, Some(first));
    account.restore();
    assert!(!account.is_archived());
}
