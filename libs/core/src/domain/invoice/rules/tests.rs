use chrono::{Datelike, TimeZone, Utc};
use rust_decimal::Decimal;
use uuid::Uuid;

use super::*;
use crate::{
    BillingFrequency, CustomerContextId, CustomerId, InvoiceDerived, OrganizationId, QuoteId,
    QuoteLine, QuoteLineId, QuoteStatus,
    domain::organization::billing_settings::{BillingSettings, BillingSettingsInput},
};

fn day(month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, month, day).unwrap()
}

fn dec(text: &str) -> Decimal {
    text.parse().unwrap()
}

fn org() -> OrganizationId {
    OrganizationId(Uuid::from_u128(1))
}

fn line(quantity: &str, price: i32, rate: Option<i32>, discount: &str) -> InvoiceLine {
    let now = Utc.with_ymd_and_hms(2026, 11, 7, 9, 0, 0).unwrap();
    InvoiceLine {
        id: InvoiceLineId(Uuid::new_v4()),
        organization_id: org(),
        invoice_id: InvoiceId(Uuid::from_u128(2)),
        label: "Pose".to_owned(),
        quantity: dec(quantity),
        unit_price_cents: price,
        vat_rate_basis_points: rate,
        discount_percent: discount_of(dec(discount)).unwrap(),
        product_id: None,
        service_rate_id: None,
        source_line_id: None,
        position: 0,
        deleted_at: None,
        created_at: now,
        updated_at: now,
    }
}

fn invoice(status: InvoiceStatus, kind: InvoiceKind, lines: Vec<InvoiceLine>) -> Invoice {
    let now = Utc.with_ymd_and_hms(2026, 11, 7, 9, 0, 0).unwrap();
    let pricing = totals_of(&lines, true).unwrap();
    Invoice {
        id: InvoiceId(Uuid::from_u128(2)),
        organization_id: org(),
        number: (status != InvoiceStatus::Draft).then(|| "FAC-2026-0001".to_owned()),
        kind,
        project_id: None,
        customer_id: CustomerId(Uuid::from_u128(3)),
        customer_context_id: CustomerContextId(Uuid::from_u128(4)),
        status,
        issued_at: None,
        due_at: None,
        notes: None,
        operation_nature: None,
        delivery_address: None,
        net_cents: pricing.net_cents,
        vat_breakdown: pricing.vat_breakdown,
        gross_cents: pricing.gross_cents,
        issuer_identity: None,
        generated_document: None,
        lines,
        source_invoice_id: None,
        title: "Chantier".to_owned(),
        payment_terms_days: 30,
        legal_notice: String::new(),
        internal_note: None,
        sent_at: None,
        quote_id: None,
        issued_on: (status != InvoiceStatus::Draft).then(|| day(11, 1)),
        reason: None,
        full_cancellation: false,
        derived: InvoiceDerived::default(),
        deleted_at: None,
        created_at: now,
        updated_at: now,
    }
}

fn draft(lines: Vec<InvoiceLine>) -> Invoice {
    invoice(InvoiceStatus::Draft, InvoiceKind::Standard, lines)
}

fn issued(lines: Vec<InvoiceLine>) -> Invoice {
    invoice(InvoiceStatus::Issued, InvoiceKind::Standard, lines)
}

fn facts(invoice: &Invoice) -> IssueFacts<'_> {
    IssueFacts {
        invoice,
        customer_kind: CustomerKind::Professional,
        registration_number: Some("12345678901234"),
        billing_address_is_postal: true,
        issue_date: day(11, 7),
        today: day(11, 7),
        last_issued_on: None,
    }
}

fn quote(lines: Vec<(&str, i32, Option<i32>, &str)>) -> Quote {
    let now = Utc.with_ymd_and_hms(2026, 11, 7, 9, 0, 0).unwrap();
    let id = QuoteId(Uuid::from_u128(9));
    let quote_lines: Vec<QuoteLine> = lines
        .into_iter()
        .map(|(quantity, price, rate, discount)| QuoteLine {
            id: QuoteLineId(Uuid::new_v4()),
            organization_id: org(),
            quote_id: id,
            service_rate_id: None,
            product_id: None,
            label: "Pose".to_owned(),
            quantity: dec(quantity),
            unit: "unit".to_owned(),
            unit_price_cents: price,
            vat_rate_bp: rate,
            discount_percent: DiscountPercent::try_from(dec(discount)).unwrap(),
            billing_frequency: BillingFrequency::OneShot,
            notes: None,
            photo_keys: vec![],
            deleted_at: None,
            created_at: now,
            updated_at: now,
        })
        .collect();
    let net: i64 = quote_lines
        .iter()
        .map(|line| line.net_cents().unwrap())
        .sum();

    Quote {
        reference: Some("DEV-2026-0007".to_owned()),
        status: QuoteStatus::Accepted,
        net_cents: net as i32,
        gross_cents: net as i32,
        lines: quote_lines,
        ..Quote::draft(
            id,
            org(),
            CustomerId(Uuid::from_u128(3)),
            CustomerContextId(Uuid::from_u128(4)),
            "Salle de bain".to_owned(),
            now,
        )
    }
}

fn target() -> DocumentLines {
    DocumentLines {
        organization_id: org(),
        invoice_id: InvoiceId(Uuid::from_u128(5)),
        now: Utc.with_ymd_and_hms(2026, 11, 7, 9, 0, 0).unwrap(),
    }
}

fn refusal(outcome: InvoiceOutcome<()>) -> InvoiceError {
    outcome.expect_err("the rule must refuse")
}

#[test]
fn spec_crm_5_1_issuing_a_complete_draft_passes() {
    let invoice = draft(vec![line("2", 10_000, Some(2000), "0")]);

    assert_eq!(validate_issue(&facts(&invoice)), Ok(()));
}

#[test]
fn spec_crm_5_1_issuing_needs_a_draft() {
    let invoice = issued(vec![line("1", 10_000, None, "0")]);

    assert_eq!(
        refusal(validate_issue(&facts(&invoice))),
        InvoiceError::Locked
    );
}

#[test]
fn spec_crm_5_1_issuing_needs_valid_lines() {
    let mut blank = line("1", 10_000, None, "0");
    blank.label = "  ".to_owned();
    let mut empty_quantity = line("1", 10_000, None, "0");
    empty_quantity.quantity = Decimal::ZERO;

    for lines in [vec![], vec![blank], vec![empty_quantity]] {
        let invoice = draft(lines);

        assert_eq!(
            refusal(validate_issue(&facts(&invoice))).field(),
            InvoiceField::Lines
        );
    }
}

#[test]
fn spec_crm_5_1_issuing_needs_a_positive_total() {
    let invoice = draft(vec![
        line("1", 10_000, None, "0"),
        line("1", -10_000, None, "0"),
    ]);

    assert_eq!(
        refusal(validate_issue(&facts(&invoice))).field(),
        InvoiceField::Total
    );
}

#[test]
fn spec_crm_5_1_issuing_needs_a_billing_street_and_city() {
    let invoice = draft(vec![line("1", 10_000, None, "0")]);
    let facts = IssueFacts {
        billing_address_is_postal: false,
        ..facts(&invoice)
    };

    assert_eq!(
        refusal(validate_issue(&facts)).field(),
        InvoiceField::BillingAddress
    );
}

#[test]
fn spec_crm_5_1_a_professional_needs_a_siret_and_an_individual_does_not() {
    let invoice = draft(vec![line("1", 10_000, None, "0")]);
    let professional = IssueFacts {
        registration_number: Some("  "),
        ..facts(&invoice)
    };
    let individual = IssueFacts {
        customer_kind: CustomerKind::Individual,
        registration_number: None,
        ..facts(&invoice)
    };

    assert_eq!(
        refusal(validate_issue(&professional)).field(),
        InvoiceField::RegistrationNumber
    );
    assert_eq!(validate_issue(&individual), Ok(()));
}

#[test]
fn spec_crm_5_2_an_issue_date_in_the_future_is_refused() {
    let invoice = draft(vec![line("1", 10_000, None, "0")]);
    let facts = IssueFacts {
        issue_date: day(11, 8),
        ..facts(&invoice)
    };

    assert_eq!(
        refusal(validate_issue(&facts)).code(),
        "E_INVOICE_DATE_RANGE"
    );
}

#[test]
fn spec_crm_5_2_an_issue_date_before_the_last_issued_invoice_is_refused() {
    let invoice = draft(vec![line("1", 10_000, None, "0")]);
    let before = IssueFacts {
        issue_date: day(11, 2),
        last_issued_on: Some(day(11, 3)),
        ..facts(&invoice)
    };
    let same_day = IssueFacts {
        issue_date: day(11, 3),
        last_issued_on: Some(day(11, 3)),
        ..facts(&invoice)
    };

    assert_eq!(
        refusal(validate_issue(&before)).code(),
        "E_INVOICE_DATE_RANGE"
    );
    assert_eq!(validate_issue(&same_day), Ok(()));
}

#[test]
fn spec_crm_5_4_payment_terms_outside_zero_to_sixty_are_refused() {
    let mut invoice = draft(vec![line("1", 10_000, None, "0")]);
    invoice.payment_terms_days = 61;

    assert_eq!(
        refusal(validate_issue(&facts(&invoice))).field(),
        InvoiceField::PaymentTermsDays
    );

    invoice.payment_terms_days = 60;
    assert_eq!(validate_issue(&facts(&invoice)), Ok(()));
    invoice.payment_terms_days = 0;
    assert_eq!(validate_issue(&facts(&invoice)), Ok(()));
}

#[test]
fn spec_crm_5_4_the_due_date_is_the_issue_date_plus_the_terms() {
    assert_eq!(due_at_for(day(11, 7), 30).date_naive(), day(12, 7));
    assert_eq!(due_at_for(day(11, 7), 0).date_naive(), day(11, 7));
    assert_eq!(
        due_at_for(day(11, 7), 60).date_naive(),
        day(1, 6).with_year(2027).unwrap()
    );
}

fn settings(penalty: &str, fee_cents: i64) -> BillingSettings {
    BillingSettings::try_from(BillingSettingsInput {
        payment_terms_days: 30,
        quote_validity_days: 30,
        default_vat_rate_bp: 2000,
        late_penalty: penalty.to_owned(),
        recovery_fee_cents: fee_cents,
        invoice_footer: String::new(),
        number_padding: 4,
        iban: None,
        bic: None,
    })
    .unwrap()
}

#[test]
fn spec_crm_5_5_an_individual_gets_the_late_penalty_only() {
    let notice = legal_notice_for(
        CustomerKind::Individual,
        &settings("trois fois le taux légal", 4000),
    );

    assert_eq!(notice, "Pénalités de retard : trois fois le taux légal.");
}

#[test]
fn spec_crm_5_5_a_professional_gets_the_penalty_and_the_recovery_fee() {
    let notice = legal_notice_for(
        CustomerKind::Professional,
        &settings("trois fois le taux légal", 4000),
    );

    assert_eq!(
        notice,
        "Pénalités de retard : trois fois le taux légal. Indemnité forfaitaire pour frais de recouvrement : 40,00\u{a0}€."
    );
}

#[test]
fn money_two_lines_of_100_33_at_20_percent_give_20066_and_4013() {
    let lines = vec![
        line("1", 10_033, Some(2000), "0"),
        line("1", 10_033, Some(2000), "0"),
    ];

    let pricing = totals_of(&lines, true).unwrap();

    assert_eq!(pricing.net_cents, 20_066);
    assert_eq!(
        pricing.vat_breakdown,
        vec![InvoiceVatBreakdownLine {
            rate_bp: 2000,
            vat_cents: 4013
        }]
    );
    assert_eq!(pricing.gross_cents, 24_079);
}

#[test]
fn money_a_line_discount_applies_before_vat_and_rounds_half_away_from_zero() {
    let lines = [line("1", 1_005, None, "10")];

    assert_eq!(line_net_cents(&lines[0]), Ok(905));
    let half = [line("1", 5, None, "10")];
    assert_eq!(line_net_cents(&half[0]), Ok(5));
    let negative = [line("1", -5, None, "50")];
    assert_eq!(line_net_cents(&negative[0]), Ok(-3));
}

#[test]
fn money_an_organization_not_subject_to_vat_carries_no_breakdown() {
    let pricing = totals_of(&[line("1", 10_000, Some(2000), "0")], false).unwrap();

    assert_eq!(pricing.vat_breakdown, vec![]);
    assert_eq!(pricing.gross_cents, 10_000);
}

#[test]
fn spec_crm_5_8_a_deposit_has_one_lump_line_per_rate() {
    let quote = quote(vec![
        ("1", 100_000, Some(2000), "0"),
        ("1", 50_000, Some(1000), "0"),
    ]);

    let lines = deposit_lines(
        &quote,
        DepositPercent::try_from(dec("30")).unwrap(),
        &target(),
    )
    .unwrap();

    assert_eq!(lines.len(), 2);
    assert_eq!(
        lines
            .iter()
            .map(|line| (
                line.vat_rate_basis_points,
                line.unit_price_cents,
                line.quantity
            ))
            .collect::<Vec<_>>(),
        vec![
            (Some(1000), 15_000, Decimal::ONE),
            (Some(2000), 30_000, Decimal::ONE)
        ]
    );
    assert!(
        lines[0]
            .label
            .starts_with("Acompte de 30 % sur devis DEV-2026-0007 (TVA 10 %)")
    );
    assert!(lines[1].label.contains("(TVA 20 %)"));
    assert!(lines[1].label.ends_with("\nSalle de bain"));
}

#[test]
fn spec_crm_5_8_a_deposit_price_rounds_half_away_from_zero() {
    let quote = quote(vec![("1", 1_005, Some(2000), "0")]);

    let lines = deposit_lines(
        &quote,
        DepositPercent::try_from(dec("50")).unwrap(),
        &target(),
    )
    .unwrap();

    assert_eq!(lines[0].unit_price_cents, 503);
    assert!(!lines[0].label.contains("TVA"));
}

#[test]
fn spec_crm_5_8_the_percentage_invoiced_is_the_deposit_base_over_the_quote_base() {
    assert_eq!(percent_invoiced(30_000, 100_000), dec("30.0"));
    assert_eq!(percent_invoiced(33_333, 100_000), dec("33.3"));
    assert_eq!(percent_invoiced(12_550, 100_000), dec("12.6"));
    assert_eq!(percent_invoiced(10, 0), Decimal::ZERO);
}

fn deposit_of(percent: &str, status: InvoiceStatus, quote: &Quote) -> Invoice {
    let lines = deposit_lines(
        quote,
        DepositPercent::try_from(dec(percent)).unwrap(),
        &target(),
    )
    .unwrap();
    let mut deposit = invoice(status, InvoiceKind::Deposit, lines);
    deposit.quote_id = Some(quote.id);
    deposit.derived.display_state = if status == InvoiceStatus::Draft {
        InvoiceDisplayState::Draft
    } else {
        InvoiceDisplayState::Unpaid
    };
    deposit
}

#[test]
fn spec_crm_5_8_deposits_count_against_the_remaining_percentage() {
    let quote = quote(vec![("1", 100_000, Some(2000), "0")]);
    let active = vec![
        deposit_of("30", InvoiceStatus::Issued, &quote),
        deposit_of("20", InvoiceStatus::Draft, &quote),
    ];

    let info = quote_invoicing(&quote, &active);

    assert_eq!(info.invoiced_percent, dec("50.0"));
    assert_eq!(info.remaining_percent, dec("50.0"));
    assert!(info.can_full && info.can_deposit);
    assert_eq!(info.blocker, None);
}

#[test]
fn spec_crm_5_8_deposits_covering_the_quote_leave_only_the_full_invoice() {
    let quote = quote(vec![("1", 100_000, Some(2000), "0")]);
    let active = vec![deposit_of("100", InvoiceStatus::Issued, &quote)];

    let info = quote_invoicing(&quote, &active);

    assert!(info.can_full);
    assert!(!info.can_deposit);
    assert_eq!(
        info.blocker,
        Some(QuoteInvoicingBlocker::DepositsCoverQuote)
    );
}

#[test]
fn spec_crm_5_7_a_full_invoice_copies_the_lines_and_deducts_each_issued_deposit_per_rate() {
    let quote = quote(vec![
        ("2", 50_000, Some(2000), "10"),
        ("1", 20_000, Some(1000), "0"),
    ]);
    let deposit = deposit_of("30", InvoiceStatus::Issued, &quote);

    let lines = full_invoice_lines(&quote, std::slice::from_ref(&deposit), &target()).unwrap();

    assert_eq!(lines.len(), 4);
    assert_eq!(lines[0].discount_percent.get(), dec("10"));
    assert_eq!(lines[0].quantity, dec("2"));
    let deductions: Vec<(Option<i32>, i32)> = lines[2..]
        .iter()
        .map(|line| (line.vat_rate_basis_points, line.unit_price_cents))
        .collect();
    assert_eq!(
        deductions,
        vec![(Some(1000), -6_000), (Some(2000), -27_000)]
    );
    assert!(lines[2].label.starts_with("Acompte déjà facturé ("));
    let total = totals_of(&lines, true).unwrap();
    assert_eq!(total.net_cents, 90_000 + 20_000 - 33_000);
}

#[test]
fn spec_crm_5_7_a_draft_deposit_is_not_deducted() {
    let quote = quote(vec![("1", 100_000, Some(2000), "0")]);

    let lines = full_invoice_lines(&quote, &[], &target()).unwrap();

    assert_eq!(lines.len(), 1);
}

#[test]
fn spec_crm_5_9_an_active_standard_invoice_blocks_the_quote_and_a_cancelled_one_does_not() {
    let quote = quote(vec![("1", 100_000, Some(2000), "0")]);
    let mut standard = issued(vec![line("1", 100_000, Some(2000), "0")]);
    standard.derived.display_state = InvoiceDisplayState::Unpaid;
    assert!(is_active_on_quote(&standard));

    let blocked = quote_invoicing(&quote, std::slice::from_ref(&standard));
    assert_eq!(
        blocked.blocker,
        Some(QuoteInvoicingBlocker::QuoteAlreadyInvoiced)
    );
    assert!(!blocked.can_full && !blocked.can_deposit);

    standard.derived.display_state = InvoiceDisplayState::Cancelled;
    assert!(!is_active_on_quote(&standard));
    let free = quote_invoicing(&quote, &[]);
    assert!(free.can_full && free.can_deposit);
}

#[test]
fn spec_crm_5_9_only_an_accepted_quote_can_be_invoiced() {
    let mut quote = quote(vec![("1", 100_000, None, "0")]);
    quote.status = QuoteStatus::Sent;

    let info = quote_invoicing(&quote, &[]);

    assert_eq!(info.blocker, Some(QuoteInvoicingBlocker::QuoteNotAccepted));
    assert!(!info.can_full && !info.can_deposit);
}

fn settlement(paid: i64, credited: i64, balance: i64) -> InvoiceSettlement {
    InvoiceSettlement {
        paid_cents: paid,
        credited_cents: credited,
        balance_cents: balance,
    }
}

#[test]
fn spec_crm_5_10_a_payment_above_the_balance_is_refused() {
    let outcome = check_payment(10_001, day(11, 7), day(11, 7), settlement(0, 0, 10_000));

    assert_eq!(refusal(outcome), InvoiceError::PaymentExceedsBalance);
    assert_eq!(
        check_payment(10_000, day(11, 7), day(11, 7), settlement(0, 0, 10_000)),
        Ok(())
    );
}

#[test]
fn spec_crm_5_10_a_refund_above_the_net_amount_collected_is_refused() {
    let outcome = check_payment(-6_001, day(11, 7), day(11, 7), settlement(6_000, 0, 4_000));

    assert_eq!(refusal(outcome), InvoiceError::RefundExceedsCollected);
    assert_eq!(
        check_payment(-6_000, day(11, 7), day(11, 7), settlement(6_000, 0, 4_000)),
        Ok(())
    );
}

#[test]
fn spec_crm_5_11_a_payment_needs_a_non_zero_amount_and_a_date_not_in_the_future() {
    let zero = check_payment(0, day(11, 7), day(11, 7), settlement(0, 0, 100));
    let future = check_payment(10, day(11, 8), day(11, 7), settlement(0, 0, 100));

    assert_eq!(refusal(zero).field(), InvoiceField::AmountCents);
    assert_eq!(refusal(future).field(), InvoiceField::PaidOn);
}

fn state_of(
    status: InvoiceStatus,
    due: Option<NaiveDate>,
    paid: i64,
    credited: i64,
) -> InvoiceDisplayState {
    let settlement = settlement_of(status, InvoiceKind::Standard, 10_000, credited, paid);

    display_state(
        status,
        InvoiceKind::Standard,
        10_000,
        due,
        settlement,
        day(11, 7),
    )
}

#[test]
fn spec_crm_5_12_the_display_state_follows_the_documented_order() {
    let due = Some(day(11, 1));

    assert_eq!(
        state_of(InvoiceStatus::Draft, due, 0, 0),
        InvoiceDisplayState::Draft
    );
    assert_eq!(
        state_of(InvoiceStatus::Issued, due, 12_000, 0),
        InvoiceDisplayState::RefundDue
    );
    assert_eq!(
        state_of(InvoiceStatus::Issued, due, 0, 10_000),
        InvoiceDisplayState::Cancelled
    );
    assert_eq!(
        state_of(InvoiceStatus::Cancelled, due, 0, 0),
        InvoiceDisplayState::Cancelled
    );
    assert_eq!(
        state_of(InvoiceStatus::Issued, due, 10_000, 0),
        InvoiceDisplayState::Paid
    );
    assert_eq!(
        state_of(InvoiceStatus::Issued, due, 4_000, 0),
        InvoiceDisplayState::Overdue
    );
    assert_eq!(
        state_of(InvoiceStatus::Issued, Some(day(12, 1)), 4_000, 0),
        InvoiceDisplayState::Partial
    );
    assert_eq!(
        state_of(InvoiceStatus::Issued, Some(day(12, 1)), 0, 0),
        InvoiceDisplayState::Unpaid
    );
}

#[test]
fn spec_crm_5_12_a_paid_then_fully_credited_invoice_is_a_refund_due_not_cancelled() {
    assert_eq!(
        state_of(InvoiceStatus::Issued, Some(day(12, 1)), 10_000, 10_000),
        InvoiceDisplayState::RefundDue
    );
}

#[test]
fn spec_crm_5_13_overdue_means_the_due_date_is_strictly_before_today() {
    assert_eq!(
        state_of(InvoiceStatus::Issued, Some(day(11, 7)), 0, 0),
        InvoiceDisplayState::Unpaid
    );
    assert_eq!(
        state_of(InvoiceStatus::Issued, Some(day(11, 6)), 0, 0),
        InvoiceDisplayState::Overdue
    );
}

#[test]
fn spec_crm_5_18_the_balance_is_total_minus_credit_notes_minus_net_payments() {
    let settlement = settlement_of(
        InvoiceStatus::Issued,
        InvoiceKind::Standard,
        12_000,
        2_000,
        7_000,
    );

    assert_eq!(settlement.balance_cents, 3_000);
    assert_eq!(
        settlement_of(InvoiceStatus::Draft, InvoiceKind::Standard, 12_000, 0, 0).balance_cents,
        0
    );
    assert_eq!(
        settlement_of(InvoiceStatus::Issued, InvoiceKind::CreditNote, 12_000, 0, 0).balance_cents,
        0
    );
}

fn two_line_source() -> Invoice {
    issued(vec![
        line("4", 5_000, Some(2000), "0"),
        line("1", 30_000, Some(2000), "0"),
    ])
}

fn credit_note_of(
    source: &Invoice,
    request: &CreditRequest,
    earlier: &[Invoice],
) -> Vec<InvoiceLine> {
    credit_note_lines(
        source,
        earlier,
        request,
        InvoiceId(Uuid::new_v4()),
        Utc.with_ymd_and_hms(2026, 11, 7, 9, 0, 0).unwrap(),
    )
    .unwrap()
}

#[test]
fn spec_crm_5_14_a_credit_line_within_what_remains_is_accepted() {
    let source = two_line_source();
    let request = CreditRequest::Lines(vec![CreditLineRequest {
        source_line_id: source.lines[0].id,
        quantity: dec("1.5"),
    }]);

    let lines = credit_note_of(&source, &request, &[]);

    assert_eq!(lines.len(), 1);
    assert_eq!(lines[0].quantity, dec("1.5"));
    assert_eq!(lines[0].source_line_id, Some(source.lines[0].id));
    assert_eq!(lines[0].unit_price_cents, 5_000);
}

#[test]
fn spec_crm_5_14_a_credit_line_above_what_remains_is_refused() {
    let source = two_line_source();
    let first = credit_note_of(
        &source,
        &CreditRequest::Lines(vec![CreditLineRequest {
            source_line_id: source.lines[0].id,
            quantity: dec("3"),
        }]),
        &[],
    );
    let mut earlier = issued(first);
    earlier.kind = InvoiceKind::CreditNote;

    let outcome = credit_note_lines(
        &source,
        std::slice::from_ref(&earlier),
        &CreditRequest::Lines(vec![CreditLineRequest {
            source_line_id: source.lines[0].id,
            quantity: dec("1.5"),
        }]),
        InvoiceId(Uuid::new_v4()),
        Utc::now(),
    );

    assert_eq!(outcome, Err(InvoiceError::CreditExceedsRemaining));
}

#[test]
fn spec_crm_5_14_two_requests_for_one_line_are_added_before_the_check() {
    let source = two_line_source();
    let request = CreditRequest::Lines(vec![
        CreditLineRequest {
            source_line_id: source.lines[0].id,
            quantity: dec("3"),
        },
        CreditLineRequest {
            source_line_id: source.lines[0].id,
            quantity: dec("2"),
        },
    ]);

    let outcome = credit_note_lines(
        &source,
        &[],
        &request,
        InvoiceId(Uuid::new_v4()),
        Utc::now(),
    );

    assert_eq!(outcome, Err(InvoiceError::CreditExceedsRemaining));
}

#[test]
fn spec_crm_5_14_an_empty_or_foreign_selection_is_refused() {
    let source = two_line_source();
    let empty = CreditRequest::Lines(vec![CreditLineRequest {
        source_line_id: source.lines[0].id,
        quantity: Decimal::ZERO,
    }]);
    let foreign = CreditRequest::Lines(vec![CreditLineRequest {
        source_line_id: InvoiceLineId(Uuid::new_v4()),
        quantity: Decimal::ONE,
    }]);

    for request in [empty, foreign] {
        let outcome = credit_note_lines(
            &source,
            &[],
            &request,
            InvoiceId(Uuid::new_v4()),
            Utc::now(),
        );

        assert_eq!(outcome.unwrap_err().field(), InvoiceField::Lines);
    }
}

#[test]
fn spec_crm_5_15_a_credit_note_is_refused_before_the_last_credit_note_or_the_invoice() {
    assert_eq!(
        refusal(check_credit_note_date(day(11, 7), Some(day(11, 8)), None)).code(),
        "E_INVOICE_DATE_RANGE"
    );
    assert_eq!(
        refusal(check_credit_note_date(day(11, 7), None, Some(day(11, 8)))).code(),
        "E_INVOICE_DATE_RANGE"
    );
    assert_eq!(
        check_credit_note_date(day(11, 7), Some(day(11, 7)), Some(day(11, 7))),
        Ok(())
    );
}

#[test]
fn spec_crm_5_16_crediting_every_remaining_line_is_a_full_cancellation() {
    let source = two_line_source();

    let all = credit_note_of(&source, &CreditRequest::Full, &[]);
    let partial = credit_note_of(
        &source,
        &CreditRequest::Lines(vec![CreditLineRequest {
            source_line_id: source.lines[0].id,
            quantity: dec("4"),
        }]),
        &[],
    );

    assert!(fully_credits(&source, &[], &all));
    assert!(!fully_credits(&source, &[], &partial));
    let mut earlier = issued(partial);
    earlier.kind = InvoiceKind::CreditNote;
    let rest = credit_note_of(
        &source,
        &CreditRequest::Full,
        std::slice::from_ref(&earlier),
    );
    assert_eq!(rest.len(), 1);
    assert!(fully_credits(
        &source,
        std::slice::from_ref(&earlier),
        &rest
    ));
}

#[test]
fn spec_crm_5_17_a_reminder_records_the_days_late() {
    assert_eq!(
        reminder_detail(Some(day(11, 1)), day(11, 7)),
        "Relance envoyée (6 jours de retard)"
    );
    assert_eq!(
        reminder_detail(Some(day(11, 7)), day(11, 7)),
        "Rappel envoyé"
    );
    assert_eq!(reminder_detail(None, day(11, 7)), "Rappel envoyé");
}
