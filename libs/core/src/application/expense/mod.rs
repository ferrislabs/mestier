use authz::Subject;
use chrono::Utc;
use common::CoreError;
use mestier_macros::transactional;

use crate::{
    Expense, ExpenseCategory, ExpenseCategoryId, ExpenseId, ExpenseOutcome, OrganizationId,
    application::MestierUseCase,
    domain::expense::{
        ExpenseCategoryService, ExpenseService,
        commands::{
            CreateExpenseCategoryCommand, CreateExpenseCommand, MarkExpensePaidCommand,
            UpdateExpenseCategoryCommand, UpdateExpenseCommand,
        },
    },
};

impl MestierUseCase {
    #[transactional(expense, expense_category, supplier, customer, member, role, authz)]
    pub async fn create_expense(
        &self,
        command: CreateExpenseCommand,
    ) -> Result<ExpenseOutcome<Expense>, CoreError> {
        let mut service = ExpenseService::new(
            expense_repository,
            expense_category_repository,
            supplier_repository,
            customer_repository,
            member_repository,
            role_repository,
            authz,
        );
        service.create_expense(command, Utc::now()).await
    }

    #[transactional(expense, expense_category, supplier, customer, member, role, authz)]
    pub async fn get_expense(&self, id: ExpenseId) -> Result<Expense, CoreError> {
        let mut service = ExpenseService::new(
            expense_repository,
            expense_category_repository,
            supplier_repository,
            customer_repository,
            member_repository,
            role_repository,
            authz,
        );
        service.get_expense(id).await
    }

    #[transactional(expense, expense_category, supplier, customer, member, role, authz)]
    pub async fn list_expenses(
        &self,
        organization_id: OrganizationId,
    ) -> Result<Vec<Expense>, CoreError> {
        let mut service = ExpenseService::new(
            expense_repository,
            expense_category_repository,
            supplier_repository,
            customer_repository,
            member_repository,
            role_repository,
            authz,
        );
        service.list_expenses(organization_id).await
    }

    #[transactional(expense, expense_category, supplier, customer, member, role, authz)]
    pub async fn update_expense(
        &self,
        command: UpdateExpenseCommand,
    ) -> Result<ExpenseOutcome<Expense>, CoreError> {
        let mut service = ExpenseService::new(
            expense_repository,
            expense_category_repository,
            supplier_repository,
            customer_repository,
            member_repository,
            role_repository,
            authz,
        );
        service.update_expense(command, Utc::now()).await
    }

    #[transactional(expense, expense_category, supplier, customer, member, role, authz)]
    pub async fn mark_expense_paid(
        &self,
        command: MarkExpensePaidCommand,
    ) -> Result<ExpenseOutcome<Expense>, CoreError> {
        let mut service = ExpenseService::new(
            expense_repository,
            expense_category_repository,
            supplier_repository,
            customer_repository,
            member_repository,
            role_repository,
            authz,
        );
        service.mark_paid(command, Utc::now()).await
    }

    #[transactional(expense, expense_category, supplier, customer, member, role, authz)]
    pub async fn reimburse_expense(
        &self,
        id: ExpenseId,
        actor: Subject,
    ) -> Result<ExpenseOutcome<Expense>, CoreError> {
        let mut service = ExpenseService::new(
            expense_repository,
            expense_category_repository,
            supplier_repository,
            customer_repository,
            member_repository,
            role_repository,
            authz,
        );
        let now = Utc::now();
        service.reimburse(id, actor, now.date_naive(), now).await
    }

    #[transactional(expense, expense_category, supplier, customer, member, role, authz)]
    pub async fn delete_expense(&self, id: ExpenseId, actor: Subject) -> Result<(), CoreError> {
        let mut service = ExpenseService::new(
            expense_repository,
            expense_category_repository,
            supplier_repository,
            customer_repository,
            member_repository,
            role_repository,
            authz,
        );
        service.delete_expense(id, actor).await
    }

    #[transactional(expense_category, role, member, authz)]
    pub async fn list_expense_categories(
        &self,
        organization_id: OrganizationId,
    ) -> Result<Vec<ExpenseCategory>, CoreError> {
        let mut service = ExpenseCategoryService::new(
            expense_category_repository,
            member_repository,
            role_repository,
            authz,
        );
        service.list_categories(organization_id, Utc::now()).await
    }

    #[transactional(expense_category, role, member, authz)]
    pub async fn get_expense_category(
        &self,
        id: ExpenseCategoryId,
    ) -> Result<ExpenseCategory, CoreError> {
        let mut service = ExpenseCategoryService::new(
            expense_category_repository,
            member_repository,
            role_repository,
            authz,
        );
        service.get_category(id).await
    }

    #[transactional(expense_category, role, member, authz)]
    pub async fn create_expense_category(
        &self,
        command: CreateExpenseCategoryCommand,
    ) -> Result<ExpenseOutcome<ExpenseCategory>, CoreError> {
        let mut service = ExpenseCategoryService::new(
            expense_category_repository,
            member_repository,
            role_repository,
            authz,
        );
        service.create_category(command, Utc::now()).await
    }

    #[transactional(expense_category, role, member, authz)]
    pub async fn update_expense_category(
        &self,
        command: UpdateExpenseCategoryCommand,
    ) -> Result<ExpenseOutcome<ExpenseCategory>, CoreError> {
        let mut service = ExpenseCategoryService::new(
            expense_category_repository,
            member_repository,
            role_repository,
            authz,
        );
        service.update_category(command, Utc::now()).await
    }

    #[transactional(expense_category, role, member, authz)]
    pub async fn delete_expense_category(
        &self,
        id: ExpenseCategoryId,
        actor: Subject,
    ) -> Result<ExpenseOutcome<()>, CoreError> {
        let mut service = ExpenseCategoryService::new(
            expense_category_repository,
            member_repository,
            role_repository,
            authz,
        );
        service.delete_category(id, actor).await
    }
}
