use chrono::{DateTime, NaiveDate, Utc};
use common::CoreError;

use crate::{
    Expense, ExpenseCategory, ExpenseCategoryId, ExpenseId, OrganizationId, PaymentMethod,
};

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait ExpenseRepository: Send {
    fn insert(
        &mut self,
        expense: &Expense,
    ) -> impl Future<Output = Result<Expense, CoreError>> + Send;

    fn find_by_id(
        &mut self,
        id: ExpenseId,
    ) -> impl Future<Output = Result<Option<Expense>, CoreError>> + Send;

    fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
    ) -> impl Future<Output = Result<Vec<Expense>, CoreError>> + Send;

    fn update(
        &mut self,
        expense: &Expense,
    ) -> impl Future<Output = Result<Expense, CoreError>> + Send;

    fn mark_paid(
        &mut self,
        id: ExpenseId,
        paid_on: NaiveDate,
        method: PaymentMethod,
        updated_at: DateTime<Utc>,
    ) -> impl Future<Output = Result<Option<Expense>, CoreError>> + Send;

    fn reimburse(
        &mut self,
        id: ExpenseId,
        reimbursed_on: NaiveDate,
        updated_at: DateTime<Utc>,
    ) -> impl Future<Output = Result<Option<Expense>, CoreError>> + Send;

    fn delete(&mut self, id: ExpenseId) -> impl Future<Output = Result<(), CoreError>> + Send;
}

#[cfg_attr(any(test, feature = "mock"), mockall::automock)]
pub trait ExpenseCategoryRepository: Send {
    fn insert(
        &mut self,
        category: &ExpenseCategory,
    ) -> impl Future<Output = Result<ExpenseCategory, CoreError>> + Send;

    fn find_by_id(
        &mut self,
        id: ExpenseCategoryId,
    ) -> impl Future<Output = Result<Option<ExpenseCategory>, CoreError>> + Send;

    fn list_by_organization(
        &mut self,
        organization_id: OrganizationId,
    ) -> impl Future<Output = Result<Vec<ExpenseCategory>, CoreError>> + Send;

    fn update(
        &mut self,
        category: &ExpenseCategory,
    ) -> impl Future<Output = Result<ExpenseCategory, CoreError>> + Send;

    fn delete(
        &mut self,
        id: ExpenseCategoryId,
    ) -> impl Future<Output = Result<(), CoreError>> + Send;

    fn is_used(
        &mut self,
        id: ExpenseCategoryId,
    ) -> impl Future<Output = Result<bool, CoreError>> + Send;
}
