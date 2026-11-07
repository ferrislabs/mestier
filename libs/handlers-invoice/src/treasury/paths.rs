use axum_extra::routing::TypedPath;
use mestier_core::{ExpenseCategoryId, ExpenseId, OrganizationId, ReceiptId};
use serde::Deserialize;

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/expenses")]
pub struct OrganizationExpensesPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/expenses/{expense_id}")]
pub struct ExpensePath {
    pub expense_id: ExpenseId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/expenses/{expense_id}/pay")]
pub struct ExpensePayPath {
    pub expense_id: ExpenseId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/expenses/{expense_id}/reimburse")]
pub struct ExpenseReimbursePath {
    pub expense_id: ExpenseId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/expense-categories")]
pub struct OrganizationExpenseCategoriesPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/expense-categories/{category_id}")]
pub struct ExpenseCategoryPath {
    pub category_id: ExpenseCategoryId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/ledger")]
pub struct OrganizationLedgerPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/receipts")]
pub struct OrganizationReceiptsPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/receipts/{receipt_id}/allocate")]
pub struct ReceiptAllocatePath {
    pub receipt_id: ReceiptId,
}
