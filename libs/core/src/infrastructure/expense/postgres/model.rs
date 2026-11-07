use std::str::FromStr;

use chrono::{DateTime, NaiveDate, Utc};
use common::CoreError;
use uuid::Uuid;

use crate::{
    CustomerId, Expense, ExpenseCategory, ExpenseCategoryId, ExpenseId, ExpenseStatus, MemberId,
    OrganizationId, PaymentMethod, SupplierId,
};

#[derive(Debug, Clone)]
pub struct ExpenseRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub date: NaiveDate,
    pub label: String,
    pub supplier_id: Option<Uuid>,
    pub category_id: Uuid,
    pub reference: Option<String>,
    pub amount_ht_cents: i32,
    pub vat_cents: i32,
    pub due_date: Option<NaiveDate>,
    pub status: String,
    pub paid_on: Option<NaiveDate>,
    pub method: Option<String>,
    pub advanced_by_member_id: Option<Uuid>,
    pub reimbursed_on: Option<NaiveDate>,
    pub customer_id: Option<Uuid>,
    pub proof_file_key: Option<String>,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub supplier_name: Option<String>,
    pub customer_name: Option<String>,
}

fn corrupt(what: &str, error: impl std::fmt::Display) -> CoreError {
    CoreError::Internal(format!("invalid expense {what} in database: {error}"))
}

impl TryFrom<ExpenseRow> for Expense {
    type Error = CoreError;

    fn try_from(row: ExpenseRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: ExpenseId(row.id),
            organization_id: OrganizationId(row.org_id),
            date: row.date,
            label: row.label,
            supplier_id: row.supplier_id.map(SupplierId),
            category_id: ExpenseCategoryId(row.category_id),
            reference: row.reference,
            amount_ht_cents: row.amount_ht_cents,
            vat_cents: row.vat_cents,
            due_date: row.due_date,
            status: ExpenseStatus::from_str(&row.status).map_err(|e| corrupt("status", e))?,
            paid_on: row.paid_on,
            method: row
                .method
                .as_deref()
                .map(PaymentMethod::from_str)
                .transpose()
                .map_err(|e| corrupt("method", e))?,
            advanced_by_member_id: row.advanced_by_member_id.map(MemberId),
            reimbursed_on: row.reimbursed_on,
            customer_id: row.customer_id.map(CustomerId),
            proof_file_key: row.proof_file_key,
            note: row.note,
            created_at: row.created_at,
            updated_at: row.updated_at,
            supplier_name: row.supplier_name,
            customer_name: row.customer_name,
        })
    }
}

#[derive(Debug, Clone)]
pub struct ExpenseCategoryRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub name: String,
    pub hue: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<ExpenseCategoryRow> for ExpenseCategory {
    fn from(row: ExpenseCategoryRow) -> Self {
        Self {
            id: ExpenseCategoryId(row.id),
            organization_id: OrganizationId(row.org_id),
            name: row.name,
            hue: row.hue,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
