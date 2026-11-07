use chrono::{DateTime, Duration, NaiveDate, Utc};

use crate::{
    Customer, CustomerId, InvoiceKind, InvoiceStatus,
    domain::invoice::{
        rules::{display_state, settlement_of},
        values::InvoiceDisplayState,
    },
};

pub const REVENUE_WINDOW_DAYS: i64 = 365;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrimaryContact {
    pub first_name: String,
    pub last_name: String,
    pub role: Option<String>,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub mobile: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomerDirectoryEntry {
    pub customer_id: CustomerId,
    pub city: Option<String>,
    pub primary_contact: Option<PrimaryContact>,
    pub last_quote_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CustomerInvoiceFact {
    pub customer_id: CustomerId,
    pub kind: InvoiceKind,
    pub status: InvoiceStatus,
    pub gross_cents: i32,
    pub net_cents: i32,
    pub issued_at: DateTime<Utc>,
    pub issued_on: NaiveDate,
    pub due_on: Option<NaiveDate>,
    pub paid_cents: i64,
    pub credited_cents: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct CustomerIndicators {
    pub revenue_12m_cents: i64,
    pub overdue_cents: i64,
    pub last_invoice_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CustomerListRow {
    pub customer: Customer,
    pub city: Option<String>,
    pub primary_contact: Option<PrimaryContact>,
    pub revenue_12m_cents: i64,
    pub overdue_cents: i64,
    pub last_activity_at: Option<DateTime<Utc>>,
}

fn revenue_window_start(today: NaiveDate) -> NaiveDate {
    today
        .checked_sub_signed(Duration::days(REVENUE_WINDOW_DAYS))
        .unwrap_or(today)
}

pub fn indicators_of(facts: &[CustomerInvoiceFact], today: NaiveDate) -> CustomerIndicators {
    let since = revenue_window_start(today);
    let mut revenue = 0_i64;
    let mut overdue = 0_i64;
    let mut last_invoice_at: Option<DateTime<Utc>> = None;

    for fact in facts {
        if matches!(fact.status, InvoiceStatus::Draft | InvoiceStatus::Cancelled) {
            continue;
        }
        let is_credit_note = fact.kind == InvoiceKind::CreditNote;
        if fact.issued_on >= since {
            let net = i64::from(fact.net_cents);
            revenue += if is_credit_note { -net } else { net };
        }
        if is_credit_note {
            continue;
        }
        last_invoice_at = last_invoice_at.max(Some(fact.issued_at));
        let settlement = settlement_of(
            fact.status,
            fact.kind,
            fact.gross_cents,
            fact.credited_cents,
            fact.paid_cents,
        );
        let state = display_state(
            fact.status,
            fact.kind,
            fact.gross_cents,
            fact.due_on,
            settlement,
            today,
        );
        if state == InvoiceDisplayState::Overdue {
            overdue += settlement.balance_cents;
        }
    }

    CustomerIndicators {
        revenue_12m_cents: revenue.max(0),
        overdue_cents: overdue,
        last_invoice_at,
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use uuid::Uuid;

    use super::*;

    fn day(year: i32, month: u32, date: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(year, month, date).unwrap()
    }

    fn at(date: NaiveDate) -> DateTime<Utc> {
        Utc.from_utc_datetime(&date.and_hms_opt(9, 0, 0).unwrap())
    }

    fn fact(issued_on: NaiveDate, due_on: Option<NaiveDate>, net: i32) -> CustomerInvoiceFact {
        CustomerInvoiceFact {
            customer_id: CustomerId(Uuid::nil()),
            kind: InvoiceKind::Standard,
            status: InvoiceStatus::Issued,
            gross_cents: net + net / 5,
            net_cents: net,
            issued_at: at(issued_on),
            issued_on,
            due_on,
            paid_cents: 0,
            credited_cents: 0,
        }
    }

    fn credit_note(issued_on: NaiveDate, net: i32) -> CustomerInvoiceFact {
        CustomerInvoiceFact {
            kind: InvoiceKind::CreditNote,
            ..fact(issued_on, None, net)
        }
    }

    fn today() -> NaiveDate {
        day(2026, 11, 7)
    }

    #[test]
    fn revenue_is_issued_minus_credit_notes_over_365_days() {
        let facts = [
            fact(day(2026, 10, 1), Some(day(2026, 10, 31)), 100_000),
            fact(day(2025, 11, 7), Some(day(2025, 12, 7)), 50_000),
            fact(day(2025, 11, 6), Some(day(2025, 12, 6)), 70_000),
            credit_note(day(2026, 10, 15), 20_000),
            credit_note(day(2025, 1, 1), 99_000),
        ];

        assert_eq!(indicators_of(&facts, today()).revenue_12m_cents, 130_000);
    }

    #[test]
    fn revenue_is_floored_at_zero() {
        let facts = [
            fact(day(2026, 10, 1), None, 10_000),
            credit_note(day(2026, 10, 15), 40_000),
        ];

        assert_eq!(indicators_of(&facts, today()).revenue_12m_cents, 0);
    }

    #[test]
    fn drafts_and_cancelled_invoices_are_not_revenue() {
        let mut draft = fact(day(2026, 10, 1), None, 10_000);
        draft.status = InvoiceStatus::Draft;
        let mut cancelled = fact(day(2026, 10, 1), None, 20_000);
        cancelled.status = InvoiceStatus::Cancelled;

        assert_eq!(
            indicators_of(&[draft, cancelled], today()),
            CustomerIndicators::default()
        );
    }

    #[test]
    fn overdue_sums_the_balance_of_invoices_due_strictly_before_today() {
        let mut partial = fact(day(2026, 9, 1), Some(day(2026, 10, 1)), 100_000);
        partial.paid_cents = 30_000;
        let facts = [
            partial,
            fact(day(2026, 10, 7), Some(day(2026, 11, 6)), 10_000),
            fact(day(2026, 10, 8), Some(day(2026, 11, 7)), 77_700),
            fact(day(2026, 10, 9), Some(day(2026, 12, 1)), 55_500),
        ];

        assert_eq!(
            indicators_of(&facts, today()).overdue_cents,
            90_000 + 12_000
        );
    }

    #[test]
    fn overdue_is_not_capped_and_ignores_paid_credited_and_refunded_invoices() {
        let mut paid = fact(day(2026, 9, 1), Some(day(2026, 10, 1)), 10_000);
        paid.paid_cents = paid.gross_cents.into();
        let mut credited = fact(day(2026, 9, 1), Some(day(2026, 10, 1)), 10_000);
        credited.credited_cents = credited.gross_cents.into();
        let mut refund_due = fact(day(2026, 9, 1), Some(day(2026, 10, 1)), 10_000);
        refund_due.paid_cents = 15_000;
        let late = fact(day(2026, 9, 1), Some(day(2026, 10, 1)), 1_000_000);

        let indicators = indicators_of(&[paid, credited, refund_due, late.clone()], today());

        assert_eq!(indicators.overdue_cents, i64::from(late.gross_cents));
    }

    #[test]
    fn credit_notes_never_count_as_overdue() {
        let mut note = credit_note(day(2026, 9, 1), 10_000);
        note.due_on = Some(day(2026, 9, 2));

        assert_eq!(indicators_of(&[note], today()).overdue_cents, 0);
    }

    #[test]
    fn the_last_invoice_date_is_the_latest_issue_instant() {
        let facts = [
            fact(day(2026, 3, 1), None, 100),
            fact(day(2026, 8, 1), None, 100),
            credit_note(day(2026, 10, 1), 100),
        ];

        assert_eq!(
            indicators_of(&facts, today()).last_invoice_at,
            Some(at(day(2026, 8, 1)))
        );
        assert_eq!(indicators_of(&[], today()).last_invoice_at, None);
    }
}
