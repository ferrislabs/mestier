use chrono::Utc;
use common::generate_uuid_v7;

use super::{
    AbsenceType, AbsenceTypeError, AbsenceTypeId, AbsenceTypeInput, CountMode,
    default_absence_types,
};
use crate::{AbsenceKind, OrganizationId};

fn input(name: &str, hue: u16) -> AbsenceTypeInput<'_> {
    AbsenceTypeInput {
        name,
        hue,
        count_mode: CountMode::Working,
        account_id: None,
        paid: true,
        needs_approval: true,
        needs_proof: false,
        kind: AbsenceKind::Leave,
    }
}

fn build(name: &str, hue: u16) -> Result<AbsenceType, AbsenceTypeError> {
    AbsenceType::new(
        AbsenceTypeId(generate_uuid_v7()),
        OrganizationId(generate_uuid_v7()),
        input(name, hue),
        Utc::now(),
    )
}

#[test]
fn a_blank_name_is_refused() {
    assert_eq!(build("   ", 10), Err(AbsenceTypeError::NameEmpty));
}

#[test]
fn a_hue_beyond_360_is_refused() {
    assert_eq!(build("RTT", 361), Err(AbsenceTypeError::HueOutOfRange));
    assert!(build("RTT", 360).is_ok());
    assert!(build("RTT", 0).is_ok());
}

#[test]
fn the_name_is_trimmed() {
    assert_eq!(build("  RTT ", 10).unwrap().name.as_str(), "RTT");
}

#[test]
fn archiving_and_restoring_toggle_the_archive_date() {
    let mut ty = build("RTT", 10).unwrap();
    let at = Utc::now();

    ty.archive(at);
    assert_eq!(ty.archived_at, Some(at));
    ty.restore();
    assert_eq!(ty.archived_at, None);
}

#[test]
fn spec_rh_3_1_the_defaults_are_the_prototype_list() {
    let names: Vec<&str> = default_absence_types().iter().map(|d| d.name).collect();
    assert_eq!(
        names,
        [
            "Congés payés",
            "RTT",
            "Arrêt maladie",
            "Intempéries",
            "Événement familial",
            "Formation",
            "Congé sans solde",
        ]
    );
    for default in default_absence_types() {
        assert!(AbsenceTypeInput::from(default).validate().is_ok());
    }
}
