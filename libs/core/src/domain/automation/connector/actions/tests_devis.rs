use std::str::FromStr;

use super::*;
use crate::QuoteStatus;

#[test]
fn the_hand_written_quote_search_is_not_generated_again() {
    for spec in SPECS {
        assert_ne!(spec.kind, "mestier.quote.search");
        assert_ne!(spec.kind, "mestier.quote.list");
    }
}

#[test]
fn reads_are_flagged_read_only_and_writes_are_not() {
    let read_only: Vec<&str> = SPECS
        .iter()
        .filter(|spec| spec.read_only)
        .map(|spec| spec.kind)
        .collect();

    assert_eq!(
        read_only,
        ["mestier.quote.get", "mestier.quote.plan_proposal"]
    );
}

#[test]
fn the_status_options_are_the_ones_the_domain_parses() {
    for option in STATUSES {
        assert!(QuoteStatus::from_str(option.value).is_ok());
    }
    let settable = QuoteStatus::ALL
        .into_iter()
        .filter(|status| *status != QuoteStatus::Superseded)
        .count();
    assert_eq!(STATUSES.len(), settable);
    assert!(
        STATUSES
            .iter()
            .all(|option| option.value != QuoteStatus::Superseded.as_str())
    );
}
