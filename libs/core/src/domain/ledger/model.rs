use std::collections::BTreeSet;

use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use super::LedgerMonth;
use crate::{
    CustomerId, Expense, ExpenseCategoryId, ExpenseDisplayState, InvoiceId, PaymentMethod,
    SupplierId,
};

pub const TREND_MONTHS: usize = 6;

#[derive(Debug, Clone, PartialEq)]
pub struct ReceiptRow {
    pub id: Uuid,
    pub date: NaiveDate,
    pub amount_cents: i64,
    pub method: PaymentMethod,
    pub reference: Option<String>,
    pub note: Option<String>,
    pub invoice_id: Option<InvoiceId>,
    pub invoice_number: Option<String>,
    pub customer_id: Option<CustomerId>,
    pub customer_name: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DocumentVatRow {
    pub date: NaiveDate,
    pub vat_cents: i64,
    pub credit_note: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LedgerKind {
    Receipt,
    Expense,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum LedgerStatus {
    Done,
    Pending,
    Late,
    Unallocated,
    ToReimburse,
}

#[derive(Debug, Clone, PartialEq)]
pub struct LedgerEntry {
    pub key: String,
    pub kind: LedgerKind,
    pub source_id: Uuid,
    pub date: NaiveDate,
    pub label: String,
    pub party: String,
    pub reference: String,
    pub amount_cents: i64,
    pub status: LedgerStatus,
    pub method: Option<PaymentMethod>,
    pub category_id: Option<ExpenseCategoryId>,
    pub invoice_id: Option<InvoiceId>,
    pub invoice_number: Option<String>,
    pub customer_id: Option<CustomerId>,
    pub supplier_id: Option<SupplierId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct LedgerSummary {
    pub receipts_cents: i64,
    pub expenses_cents: i64,
    pub net_cents: i64,
    pub pending_cents: i64,
    pub unallocated_cents: i64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MonthTrend {
    pub month: String,
    pub receipts_cents: i64,
    pub expenses_cents: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct VatSummary {
    pub collected_cents: i64,
    pub deductible_cents: i64,
    pub due_cents: i64,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Payables {
    pub total_cents: i64,
    pub overdue_cents: i64,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Ledger {
    pub entries: Vec<LedgerEntry>,
    pub months: Vec<String>,
    pub summary: LedgerSummary,
    pub trend: Vec<MonthTrend>,
    pub vat: VatSummary,
    pub payables: Payables,
}

fn receipt_entry(row: ReceiptRow) -> LedgerEntry {
    let unallocated = row.invoice_id.is_none() && row.amount_cents > 0;
    LedgerEntry {
        key: format!("receipt:{}", row.id),
        kind: LedgerKind::Receipt,
        source_id: row.id,
        date: row.date,
        label: row.note.unwrap_or_default(),
        party: row.customer_name.unwrap_or_default(),
        reference: row.reference.unwrap_or_default(),
        amount_cents: row.amount_cents,
        status: if unallocated {
            LedgerStatus::Unallocated
        } else {
            LedgerStatus::Done
        },
        method: Some(row.method),
        category_id: None,
        invoice_id: row.invoice_id,
        invoice_number: row.invoice_number,
        customer_id: row.customer_id,
        supplier_id: None,
    }
}

fn expense_entry(expense: &Expense, today: NaiveDate) -> LedgerEntry {
    let status = match expense.display_state(today) {
        ExpenseDisplayState::Overdue => LedgerStatus::Late,
        ExpenseDisplayState::ToPay => LedgerStatus::Pending,
        ExpenseDisplayState::ToReimburse => LedgerStatus::ToReimburse,
        ExpenseDisplayState::Paid | ExpenseDisplayState::Reimbursed => LedgerStatus::Done,
    };
    LedgerEntry {
        key: format!("expense:{}", expense.id.0),
        kind: LedgerKind::Expense,
        source_id: expense.id.0,
        date: expense.date,
        label: expense.label.clone(),
        party: expense.supplier_name.clone().unwrap_or_default(),
        reference: expense.reference.clone().unwrap_or_default(),
        amount_cents: -expense.total_cents(),
        status,
        method: expense.method,
        category_id: Some(expense.category_id),
        invoice_id: None,
        invoice_number: None,
        customer_id: expense.customer_id,
        supplier_id: expense.supplier_id,
    }
}

pub fn summarize<'a>(entries: impl IntoIterator<Item = &'a LedgerEntry>) -> LedgerSummary {
    let mut summary = LedgerSummary::default();
    for entry in entries {
        match entry.kind {
            LedgerKind::Receipt => {
                summary.receipts_cents += entry.amount_cents;
                if entry.status == LedgerStatus::Unallocated {
                    summary.unallocated_cents += entry.amount_cents;
                }
            }
            LedgerKind::Expense => {
                summary.expenses_cents -= entry.amount_cents;
                if matches!(entry.status, LedgerStatus::Pending | LedgerStatus::Late) {
                    summary.pending_cents -= entry.amount_cents;
                }
            }
        }
    }
    summary.net_cents = summary.receipts_cents - summary.expenses_cents;
    summary
}

fn trend(entries: &[LedgerEntry], today: NaiveDate) -> Vec<MonthTrend> {
    let mut months = Vec::with_capacity(TREND_MONTHS);
    let mut cursor = LedgerMonth::of(today);
    for _ in 0..TREND_MONTHS {
        months.push(cursor);
        cursor = cursor.previous();
    }
    months.reverse();
    months
        .into_iter()
        .map(|month| {
            let summary = summarize(entries.iter().filter(|entry| month.contains(entry.date)));
            MonthTrend {
                month: month.key(),
                receipts_cents: summary.receipts_cents,
                expenses_cents: summary.expenses_cents,
            }
        })
        .collect()
}

fn vat(
    documents: &[DocumentVatRow],
    expenses: &[Expense],
    month: Option<LedgerMonth>,
) -> VatSummary {
    let in_month = |date: NaiveDate| month.is_none_or(|month| month.contains(date));
    let collected: i64 = documents
        .iter()
        .filter(|row| in_month(row.date))
        .map(|row| {
            if row.credit_note {
                -row.vat_cents
            } else {
                row.vat_cents
            }
        })
        .sum();
    let deductible: i64 = expenses
        .iter()
        .filter(|expense| in_month(expense.date))
        .map(|expense| i64::from(expense.vat_cents))
        .sum();
    VatSummary {
        collected_cents: collected,
        deductible_cents: deductible,
        due_cents: collected - deductible,
    }
}

fn payables(expenses: &[Expense], today: NaiveDate) -> Payables {
    let mut payables = Payables::default();
    for expense in expenses {
        match expense.display_state(today) {
            ExpenseDisplayState::ToPay => payables.total_cents += expense.total_cents(),
            ExpenseDisplayState::Overdue => {
                payables.total_cents += expense.total_cents();
                payables.overdue_cents += expense.total_cents();
            }
            ExpenseDisplayState::Paid
            | ExpenseDisplayState::ToReimburse
            | ExpenseDisplayState::Reimbursed => {}
        }
    }
    payables
}

pub fn build_ledger(
    receipts: Vec<ReceiptRow>,
    expenses: &[Expense],
    documents: &[DocumentVatRow],
    month: Option<LedgerMonth>,
    today: NaiveDate,
) -> Ledger {
    let mut entries: Vec<LedgerEntry> = receipts
        .into_iter()
        .map(receipt_entry)
        .chain(expenses.iter().map(|expense| expense_entry(expense, today)))
        .collect();
    entries.sort_by(|a, b| b.date.cmp(&a.date).then_with(|| a.key.cmp(&b.key)));

    let months: Vec<String> = entries
        .iter()
        .map(|entry| LedgerMonth::of(entry.date).key())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .rev()
        .collect();
    let summary = summarize(
        entries
            .iter()
            .filter(|entry| month.is_none_or(|month| month.contains(entry.date))),
    );

    Ledger {
        trend: trend(&entries, today),
        vat: vat(documents, expenses, month),
        payables: payables(expenses, today),
        entries,
        months,
        summary,
    }
}

#[cfg(test)]
mod tests;
