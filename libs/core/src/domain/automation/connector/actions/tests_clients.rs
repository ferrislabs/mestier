use std::str::FromStr;

use super::*;
use crate::{CustomerPipelineStage, CustomerStatus};

#[test]
fn the_hand_written_customer_connector_is_not_generated_again() {
    for spec in SPECS {
        assert_ne!(spec.kind, "mestier.customer.create");
    }
}

#[test]
fn reads_are_flagged_read_only_and_writes_are_not() {
    for spec in SPECS {
        let reads = spec.kind.ends_with(".get") || spec.kind.ends_with(".list");
        assert_eq!(spec.read_only, reads, "{}", spec.kind);
    }
}

#[test]
fn the_select_options_are_the_ones_the_domain_parses() {
    for option in STATUSES {
        assert!(CustomerStatus::from_str(option.value).is_ok());
    }
    for option in STAGES {
        assert!(CustomerPipelineStage::from_str(option.value).is_ok());
    }
    assert_eq!(STATUSES.len(), 3);
    assert_eq!(STAGES.len(), 6);
}
