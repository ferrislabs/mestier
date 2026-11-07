use chrono::NaiveDate;
use common::generate_uuid_v7;
use uuid::Uuid;

use crate::{
    CustomerId, DocumentVatRow, Expense, ExpenseCategoryId, ExpenseId, ExpenseStatus, InvoiceId,
    LedgerKind, LedgerMonth, LedgerStatus, MemberId, OrganizationId, PaymentMethod, ReceiptRow,
    SupplierId, build_ledger,
};

fn day(month: u32, day: u32) -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, month, day).unwrap()
}

fn receipt(date: NaiveDate, amount_cents: i64, invoice: bool) -> ReceiptRow {
    ReceiptRow {
        id: generate_uuid_v7(),
        date,
        amount_cents,
        method: PaymentMethod::Transfer,
        reference: Some("VIR".to_owned()),
        note: None,
        invoice_id: invoice.then(|| InvoiceId(generate_uuid_v7())),
        invoice_number: invoice.then(|| "F-2026-0001".to_owned()),
        customer_id: Some(CustomerId(generate_uuid_v7())),
        customer_name: Some("Lemoine".to_owned()),
    }
}

fn expense(date: NaiveDate, amount_ht: i32, vat: i32) -> Expense {
    let now = date.and_hms_opt(8, 0, 0).unwrap().and_utc();
    Expense {
        id: ExpenseId(generate_uuid_v7()),
        organization_id: OrganizationId(Uuid::nil()),
        date,
        label: "Carburant".to_owned(),
        supplier_id: Some(SupplierId(generate_uuid_v7())),
        category_id: ExpenseCategoryId(generate_uuid_v7()),
        reference: None,
        amount_ht_cents: amount_ht,
        vat_cents: vat,
        due_date: None,
        status: ExpenseStatus::Paid,
        paid_on: Some(date),
        method: Some(PaymentMethod::Card),
        advanced_by_member_id: None,
        reimbursed_on: None,
        customer_id: None,
        proof_file_key: None,
        note: None,
        created_at: now,
        updated_at: now,
        supplier_name: Some("Avia".to_owned()),
        customer_name: None,
    }
}

#[test]
fn the_journal_signs_receipts_and_expenses_and_sorts_by_date_descending() {
    let today = day(11, 10);
    let refund = receipt(day(11, 5), -2_000, true);
    let payment = receipt(day(11, 8), 10_000, true);
    let fuel = expense(day(11, 6), 8_000, 1_600);

    let ledger = build_ledger(vec![refund, payment], &[fuel], &[], None, today);

    let dates: Vec<NaiveDate> = ledger.entries.iter().map(|entry| entry.date).collect();
    assert_eq!(dates, vec![day(11, 8), day(11, 6), day(11, 5)]);
    let amounts: Vec<i64> = ledger
        .entries
        .iter()
        .map(|entry| entry.amount_cents)
        .collect();
    assert_eq!(amounts, vec![10_000, -9_600, -2_000]);
    assert_eq!(ledger.entries[1].kind, LedgerKind::Expense);
    assert_eq!(ledger.entries[1].party, "Avia");
    assert_eq!(
        ledger.entries[0].invoice_number.as_deref(),
        Some("F-2026-0001")
    );
}

#[test]
fn entries_of_the_same_day_are_ordered_by_key() {
    let a = receipt(day(11, 8), 100, true);
    let b = receipt(day(11, 8), 200, true);
    let (first, second) = if format!("receipt:{}", a.id) < format!("receipt:{}", b.id) {
        (a.id, b.id)
    } else {
        (b.id, a.id)
    };

    let ledger = build_ledger(vec![a, b], &[], &[], None, day(11, 10));

    assert_eq!(ledger.entries[0].source_id, first);
    assert_eq!(ledger.entries[1].source_id, second);
}

#[test]
fn the_summary_gives_receipts_expenses_net_pending_and_unallocated() {
    let today = day(11, 10);
    let allocated = receipt(day(11, 8), 10_000, true);
    let unallocated = receipt(day(11, 7), 4_000, false);
    let refund = receipt(day(11, 5), -1_000, true);
    let paid = expense(day(11, 6), 8_000, 1_600);
    let mut pending = expense(day(11, 4), 5_000, 1_000);
    pending.status = ExpenseStatus::ToPay;
    pending.paid_on = None;
    pending.method = None;
    pending.due_date = Some(day(11, 30));
    let mut late = expense(day(11, 3), 2_000, 400);
    late.status = ExpenseStatus::ToPay;
    late.paid_on = None;
    late.method = None;
    late.due_date = Some(day(11, 9));
    let mut advanced = expense(day(11, 2), 1_000, 200);
    advanced.advanced_by_member_id = Some(MemberId(generate_uuid_v7()));

    let ledger = build_ledger(
        vec![allocated, unallocated, refund],
        &[paid, pending, late, advanced],
        &[],
        None,
        today,
    );

    assert_eq!(ledger.summary.receipts_cents, 13_000);
    assert_eq!(ledger.summary.expenses_cents, 9_600 + 6_000 + 2_400 + 1_200);
    assert_eq!(ledger.summary.net_cents, 13_000 - 19_200);
    assert_eq!(ledger.summary.pending_cents, 6_000 + 2_400);
    assert_eq!(ledger.summary.unallocated_cents, 4_000);
    let statuses: Vec<LedgerStatus> = ledger
        .entries
        .iter()
        .filter(|entry| entry.kind == LedgerKind::Expense)
        .map(|entry| entry.status)
        .collect();
    assert!(statuses.contains(&LedgerStatus::Late));
    assert!(statuses.contains(&LedgerStatus::Pending));
    assert!(statuses.contains(&LedgerStatus::ToReimburse));
    assert!(statuses.contains(&LedgerStatus::Done));
    assert_eq!(ledger.payables.total_cents, 6_000 + 2_400);
    assert_eq!(ledger.payables.overdue_cents, 2_400);
}

#[test]
fn a_negative_receipt_without_invoice_is_not_unallocated() {
    let ledger = build_ledger(
        vec![receipt(day(11, 8), -500, false)],
        &[],
        &[],
        None,
        day(11, 10),
    );

    assert_eq!(ledger.entries[0].status, LedgerStatus::Done);
    assert_eq!(ledger.summary.unallocated_cents, 0);
}

#[test]
fn the_month_filter_narrows_the_summary_but_not_the_trend() {
    let today = day(11, 10);
    let october = receipt(day(10, 20), 7_000, true);
    let november = receipt(day(11, 2), 3_000, true);

    let ledger = build_ledger(
        vec![october, november],
        &[],
        &[],
        Some("2026-10".parse::<LedgerMonth>().unwrap()),
        today,
    );

    assert_eq!(ledger.summary.receipts_cents, 7_000);
    let november_trend = ledger.trend.last().unwrap();
    assert_eq!(november_trend.month, "2026-11");
    assert_eq!(november_trend.receipts_cents, 3_000);
    assert_eq!(
        ledger.months,
        vec!["2026-11".to_owned(), "2026-10".to_owned()]
    );
}

#[test]
fn the_trend_covers_the_last_six_months_with_the_empty_ones() {
    let today = day(2, 14);
    let january = receipt(day(1, 5), 1_000, true);
    let fuel = expense(day(1, 6), 500, 100);

    let ledger = build_ledger(vec![january], &[fuel], &[], None, today);

    let months: Vec<&str> = ledger.trend.iter().map(|m| m.month.as_str()).collect();
    assert_eq!(
        months,
        vec![
            "2025-09", "2025-10", "2025-11", "2025-12", "2026-01", "2026-02"
        ]
    );
    let january = &ledger.trend[4];
    assert_eq!(january.receipts_cents, 1_000);
    assert_eq!(january.expenses_cents, 600);
    assert_eq!(ledger.trend[0].receipts_cents, 0);
    assert_eq!(ledger.trend[5].expenses_cents, 0);
}

#[test]
fn an_entry_outside_the_window_is_not_in_the_trend() {
    let old = receipt(day(1, 5), 9_999, true);

    let ledger = build_ledger(vec![old], &[], &[], None, day(11, 10));

    assert!(ledger.trend.iter().all(|month| month.receipts_cents == 0));
    assert_eq!(ledger.trend.len(), 6);
}

#[test]
fn vat_is_collected_minus_credit_notes_deductible_from_expenses_and_due_the_difference() {
    let documents = [
        DocumentVatRow {
            date: day(11, 2),
            vat_cents: 20_000,
            credit_note: false,
        },
        DocumentVatRow {
            date: day(11, 5),
            vat_cents: 5_000,
            credit_note: true,
        },
        DocumentVatRow {
            date: day(10, 5),
            vat_cents: 1_000,
            credit_note: false,
        },
    ];
    let expenses = [
        expense(day(11, 3), 10_000, 2_000),
        expense(day(10, 3), 1_000, 200),
    ];

    let all = build_ledger(vec![], &expenses, &documents, None, day(11, 10));
    let november = build_ledger(
        vec![],
        &expenses,
        &documents,
        Some("2026-11".parse().unwrap()),
        day(11, 10),
    );

    assert_eq!(all.vat.collected_cents, 16_000);
    assert_eq!(all.vat.deductible_cents, 2_200);
    assert_eq!(all.vat.due_cents, 13_800);
    assert_eq!(november.vat.collected_cents, 15_000);
    assert_eq!(november.vat.deductible_cents, 2_000);
    assert_eq!(november.vat.due_cents, 13_000);
}

#[test]
fn a_month_parses_only_as_year_dash_month() {
    assert!("2026-11".parse::<LedgerMonth>().is_ok());
    for bad in ["2026-13", "2026-1", "26-11", "2026/11", "", "2026-00"] {
        assert!(bad.parse::<LedgerMonth>().is_err(), "{bad}");
    }
    assert_eq!(
        "2026-01".parse::<LedgerMonth>().unwrap().previous().key(),
        "2025-12"
    );
}
