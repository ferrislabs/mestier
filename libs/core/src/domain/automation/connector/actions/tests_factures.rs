use std::str::FromStr;

use super::*;
use crate::{InvoiceKind, OperationNature, PaymentMethod};

#[test]
fn the_payment_writes_are_offered_as_writes() {
    for kind in [
        "mestier.invoice.record_payment",
        "mestier.invoice.delete_payment",
    ] {
        let spec = SPECS.iter().find(|spec| spec.kind == kind).unwrap();

        assert!(!spec.read_only, "{kind}");
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
        [
            "mestier.invoice.get",
            "mestier.invoice.list",
            "mestier.invoice.list_by_project",
            "mestier.invoice.list_credit_notes",
            "mestier.invoice.balance",
            "mestier.invoice.list_payments",
            "mestier.invoice.outstanding_by_customer",
            "mestier.invoice.project_billing_summary",
        ]
    );
}

#[test]
fn the_select_options_are_the_ones_the_domain_parses() {
    for option in KINDS {
        assert!(InvoiceKind::from_str(option.value).is_ok());
    }
    for option in NATURES {
        assert!(OperationNature::from_str(option.value).is_ok());
    }
    for option in METHODS {
        assert!(PaymentMethod::from_str(option.value).is_ok());
    }
    assert_eq!(METHODS.len(), PaymentMethod::ALL.len());
    assert_eq!(KINDS.len(), InvoiceKind::ALL.len());
    assert_eq!(NATURES.len(), OperationNature::ALL.len());
}
