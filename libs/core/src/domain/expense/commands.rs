use authz::Subject;
use chrono::NaiveDate;

use crate::{
    CustomerId, ExpenseCategoryId, ExpenseId, MemberId, OrganizationId, PaymentMethod, SupplierId,
};

#[derive(Debug, Clone)]
pub struct CreateExpenseCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub date: NaiveDate,
    pub label: String,
    pub supplier_id: Option<SupplierId>,
    pub category_id: ExpenseCategoryId,
    pub reference: Option<String>,
    pub amount_ht_cents: i32,
    pub vat_cents: i32,
    pub due_date: Option<NaiveDate>,
    pub advanced_by_member_id: Option<MemberId>,
    pub customer_id: Option<CustomerId>,
    pub proof_file_key: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Clone)]
pub struct UpdateExpenseCommand {
    pub actor: Subject,
    pub id: ExpenseId,
    pub date: NaiveDate,
    pub label: String,
    pub supplier_id: Option<SupplierId>,
    pub category_id: ExpenseCategoryId,
    pub reference: Option<String>,
    pub amount_ht_cents: i32,
    pub vat_cents: i32,
    pub due_date: Option<NaiveDate>,
    pub customer_id: Option<CustomerId>,
    pub proof_file_key: Option<String>,
    pub note: Option<String>,
}

#[derive(Debug, Clone)]
pub struct MarkExpensePaidCommand {
    pub actor: Subject,
    pub id: ExpenseId,
    pub paid_on: NaiveDate,
    pub method: PaymentMethod,
}

#[derive(Debug, Clone)]
pub struct CreateExpenseCategoryCommand {
    pub actor: Subject,
    pub organization_id: OrganizationId,
    pub name: String,
    pub hue: i64,
}

#[derive(Debug, Clone)]
pub struct UpdateExpenseCategoryCommand {
    pub actor: Subject,
    pub id: ExpenseCategoryId,
    pub name: String,
    pub hue: i64,
}
