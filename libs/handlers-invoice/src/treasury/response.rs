use chrono::{DateTime, NaiveDate, Utc};
use mestier_core::{
    CustomerId, Expense, ExpenseCategory, ExpenseCategoryId, ExpenseDisplayState, ExpenseId,
    ExpenseStatus, InvoiceId, Ledger, LedgerEntry, LedgerKind, LedgerStatus, MemberId, MonthTrend,
    OrganizationId, PaymentMethod, Receipt, ReceiptId, SupplierId,
};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct ExpenseCategoryResponse {
    pub id: ExpenseCategoryId,
    pub organization_id: OrganizationId,
    pub name: String,
    pub hue: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ExpenseCategory> for ExpenseCategoryResponse {
    fn from(value: ExpenseCategory) -> Self {
        Self {
            id: value.id,
            organization_id: value.organization_id,
            name: value.name,
            hue: value.hue,
            created_at: value.created_at,
            updated_at: value.updated_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct ExpenseResponse {
    pub id: ExpenseId,
    pub organization_id: OrganizationId,
    pub date: NaiveDate,
    pub label: String,
    pub supplier_id: Option<SupplierId>,
    pub supplier_name: Option<String>,
    pub category_id: ExpenseCategoryId,
    pub reference: Option<String>,
    pub amount_ht_cents: i32,
    pub vat_cents: i32,
    pub total_cents: i64,
    pub due_date: Option<NaiveDate>,
    pub status: ExpenseStatus,
    pub display_state: ExpenseDisplayState,
    pub days_late: i64,
    pub paid_on: Option<NaiveDate>,
    pub method: Option<PaymentMethod>,
    pub advanced_by_member_id: Option<MemberId>,
    pub reimbursed_on: Option<NaiveDate>,
    pub customer_id: Option<CustomerId>,
    pub customer_name: Option<String>,
    pub proof_file_key: Option<String>,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ExpenseResponse {
    pub fn new(expense: Expense, today: NaiveDate) -> Self {
        Self {
            display_state: expense.display_state(today),
            days_late: expense.days_late(today),
            total_cents: expense.total_cents(),
            id: expense.id,
            organization_id: expense.organization_id,
            date: expense.date,
            label: expense.label,
            supplier_id: expense.supplier_id,
            supplier_name: expense.supplier_name,
            category_id: expense.category_id,
            reference: expense.reference,
            amount_ht_cents: expense.amount_ht_cents,
            vat_cents: expense.vat_cents,
            due_date: expense.due_date,
            status: expense.status,
            paid_on: expense.paid_on,
            method: expense.method,
            advanced_by_member_id: expense.advanced_by_member_id,
            reimbursed_on: expense.reimbursed_on,
            customer_id: expense.customer_id,
            customer_name: expense.customer_name,
            proof_file_key: expense.proof_file_key,
            note: expense.note,
            created_at: expense.created_at,
            updated_at: expense.updated_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct ReceiptResponse {
    pub id: ReceiptId,
    pub organization_id: OrganizationId,
    pub customer_id: Option<CustomerId>,
    pub amount_cents: i32,
    pub paid_on: NaiveDate,
    pub method: PaymentMethod,
    pub reference: Option<String>,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
}

impl From<Receipt> for ReceiptResponse {
    fn from(value: Receipt) -> Self {
        Self {
            id: value.id,
            organization_id: value.organization_id,
            customer_id: value.customer_id,
            amount_cents: value.amount_cents,
            paid_on: value.paid_on,
            method: value.method,
            reference: value.reference,
            note: value.note,
            created_at: value.created_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct LedgerEntryResponse {
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

impl From<LedgerEntry> for LedgerEntryResponse {
    fn from(value: LedgerEntry) -> Self {
        Self {
            key: value.key,
            kind: value.kind,
            source_id: value.source_id,
            date: value.date,
            label: value.label,
            party: value.party,
            reference: value.reference,
            amount_cents: value.amount_cents,
            status: value.status,
            method: value.method,
            category_id: value.category_id,
            invoice_id: value.invoice_id,
            invoice_number: value.invoice_number,
            customer_id: value.customer_id,
            supplier_id: value.supplier_id,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct LedgerSummaryResponse {
    pub receipts_cents: i64,
    pub expenses_cents: i64,
    pub net_cents: i64,
    pub pending_cents: i64,
    pub unallocated_cents: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct MonthTrendResponse {
    pub month: String,
    pub receipts_cents: i64,
    pub expenses_cents: i64,
}

impl From<MonthTrend> for MonthTrendResponse {
    fn from(value: MonthTrend) -> Self {
        Self {
            month: value.month,
            receipts_cents: value.receipts_cents,
            expenses_cents: value.expenses_cents,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct VatSummaryResponse {
    pub collected_cents: i64,
    pub deductible_cents: i64,
    pub due_cents: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct PayablesResponse {
    pub total_cents: i64,
    pub overdue_cents: i64,
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct LedgerResponse {
    pub entries: Vec<LedgerEntryResponse>,
    pub months: Vec<String>,
    pub summary: LedgerSummaryResponse,
    pub trend: Vec<MonthTrendResponse>,
    pub vat: VatSummaryResponse,
    pub payables: PayablesResponse,
}

impl From<Ledger> for LedgerResponse {
    fn from(value: Ledger) -> Self {
        Self {
            entries: value.entries.into_iter().map(Into::into).collect(),
            months: value.months,
            summary: LedgerSummaryResponse {
                receipts_cents: value.summary.receipts_cents,
                expenses_cents: value.summary.expenses_cents,
                net_cents: value.summary.net_cents,
                pending_cents: value.summary.pending_cents,
                unallocated_cents: value.summary.unallocated_cents,
            },
            trend: value.trend.into_iter().map(Into::into).collect(),
            vat: VatSummaryResponse {
                collected_cents: value.vat.collected_cents,
                deductible_cents: value.vat.deductible_cents,
                due_cents: value.vat.due_cents,
            },
            payables: PayablesResponse {
                total_cents: value.payables.total_cents,
                overdue_cents: value.payables.overdue_cents,
            },
        }
    }
}
