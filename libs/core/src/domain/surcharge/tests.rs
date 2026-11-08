use chrono::Utc;
use common::generate_uuid_v7;

use super::*;

fn input(name: &str, rate_bp: i32) -> SurchargeInput<'_> {
    SurchargeInput {
        name,
        rate_bp,
        kind: SurchargeKind::Overtime,
    }
}

fn surcharge() -> Surcharge {
    Surcharge::new(
        SurchargeId(generate_uuid_v7()),
        OrganizationId(generate_uuid_v7()),
        input("Nuit", 2500),
        Utc::now(),
    )
    .unwrap()
}

#[test]
fn a_blank_name_is_refused_and_a_name_is_trimmed() {
    assert_eq!(
        SurchargeName::try_from("  "),
        Err(SurchargeError::NameEmpty)
    );
    assert_eq!(SurchargeName::try_from(" Nuit ").unwrap().as_str(), "Nuit");
}

#[test]
fn a_rate_stays_between_zero_and_the_cap() {
    assert_eq!(RateBp::try_from(-1), Err(SurchargeError::RateInvalid));
    assert_eq!(RateBp::try_from(20_001), Err(SurchargeError::RateInvalid));
    assert_eq!(RateBp::try_from(0).unwrap(), RateBp::ZERO);
    assert_eq!(RateBp::try_from(20_000).unwrap().get(), 20_000);
}

#[test]
fn the_defaults_follow_the_prototype_list() {
    let defaults: Vec<(&str, i32, SurchargeKind)> = default_surcharges()
        .iter()
        .map(|default| (default.name, default.rate_bp, default.kind))
        .collect();

    assert_eq!(
        defaults,
        vec![
            ("Heures supplémentaires", 2500, SurchargeKind::Overtime),
            (
                "Heures supplémentaires au-delà de 8 h",
                5000,
                SurchargeKind::Overtime
            ),
            ("Travail du dimanche", 10000, SurchargeKind::Sunday),
            ("Jour férié travaillé", 10000, SurchargeKind::Holiday),
            ("Travail de nuit", 2500, SurchargeKind::Night),
        ]
    );
}

#[test]
fn revising_with_a_bad_value_changes_nothing() {
    let mut surcharge = surcharge();
    let before = surcharge.clone();

    assert_eq!(
        surcharge.revise(input("", 100)),
        Err(SurchargeError::NameEmpty)
    );
    assert_eq!(
        surcharge.revise(input("Autre", -5)),
        Err(SurchargeError::RateInvalid)
    );

    assert_eq!(surcharge, before);
}

#[test]
fn archiving_keeps_the_first_date_and_restoring_clears_it() {
    let mut surcharge = surcharge();
    let first = Utc::now();
    surcharge.archive(first);
    surcharge.archive(first + chrono::Duration::days(1));
    assert_eq!(surcharge.archived_at, Some(first));
    assert!(surcharge.is_archived());

    surcharge.restore();

    assert!(!surcharge.is_archived());
}
