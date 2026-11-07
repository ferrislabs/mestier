use auth::Identity;
use axum::{Extension, Json, extract::State};
use chrono::{NaiveDate, Utc};
use handlers::{AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::{
    CreateExpenseCommand, CustomerId, ExpenseCategoryId, ExpenseId, MarkExpensePaidCommand,
    MemberId, PaymentMethod, SupplierId, UpdateExpenseCommand,
};
use serde::Deserialize;
use utoipa::ToSchema;

use super::{
    error::ExpenseFailure,
    paths::{ExpensePath, ExpensePayPath, ExpenseReimbursePath, OrganizationExpensesPath},
    response::ExpenseResponse,
};
use crate::{require_expense_membership, require_org_membership, require_view_invoices};

fn today() -> NaiveDate {
    Utc::now().date_naive()
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateExpenseRequest {
    pub date: NaiveDate,
    pub label: String,
    #[serde(default)]
    pub supplier_id: Option<SupplierId>,
    pub category_id: ExpenseCategoryId,
    #[serde(default)]
    pub reference: Option<String>,
    pub amount_ht_cents: i32,
    #[serde(default)]
    pub vat_cents: i32,
    #[serde(default)]
    pub due_date: Option<NaiveDate>,
    #[serde(default)]
    pub advanced_by_member_id: Option<MemberId>,
    #[serde(default)]
    pub customer_id: Option<CustomerId>,
    #[serde(default)]
    pub proof_file_key: Option<String>,
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateExpenseRequest {
    pub date: NaiveDate,
    pub label: String,
    #[serde(default)]
    pub supplier_id: Option<SupplierId>,
    pub category_id: ExpenseCategoryId,
    #[serde(default)]
    pub reference: Option<String>,
    pub amount_ht_cents: i32,
    #[serde(default)]
    pub vat_cents: i32,
    #[serde(default)]
    pub due_date: Option<NaiveDate>,
    #[serde(default)]
    pub customer_id: Option<CustomerId>,
    #[serde(default)]
    pub proof_file_key: Option<String>,
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PayExpenseRequest {
    pub paid_on: NaiveDate,
    pub method: PaymentMethod,
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/expenses",
    operation_id = "listExpenses",
    tag = crate::TREASURY_TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    responses(
        (status = 200, description = "Every expense of the organization, newest first", body = inline(DataEnvelope<Vec<ExpenseResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_handler(
    OrganizationExpensesPath { organization_id }: OrganizationExpensesPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<ExpenseResponse>>, ExpenseFailure> {
    require_view_invoices(&state, &identity, organization_id).await?;

    let day = today();
    let expenses = state.usecase.list_expenses(organization_id).await?;

    Ok(Response::OK(
        expenses
            .into_iter()
            .map(|expense| ExpenseResponse::new(expense, day))
            .collect(),
    ))
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/expenses",
    operation_id = "createExpense",
    tag = crate::TREASURY_TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    request_body = CreateExpenseRequest,
    responses(
        (status = 201, description = "Expense created", body = inline(DataEnvelope<ExpenseResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Category, supplier, customer or member not found"),
        (status = 422, description = "A rule is refused, the field is named"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_handler(
    OrganizationExpensesPath { organization_id }: OrganizationExpensesPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateExpenseRequest>,
) -> Result<Response<ExpenseResponse>, ExpenseFailure> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let expense = state
        .usecase
        .acting_as(user_id)
        .create_expense(CreateExpenseCommand {
            actor,
            organization_id,
            date: payload.date,
            label: payload.label,
            supplier_id: payload.supplier_id,
            category_id: payload.category_id,
            reference: payload.reference,
            amount_ht_cents: payload.amount_ht_cents,
            vat_cents: payload.vat_cents,
            due_date: payload.due_date,
            advanced_by_member_id: payload.advanced_by_member_id,
            customer_id: payload.customer_id,
            proof_file_key: payload.proof_file_key,
            note: payload.note,
        })
        .await??;

    Ok(Response::Created(ExpenseResponse::new(expense, today())))
}

#[utoipa::path(
    get,
    path = "/api/v1/expenses/{expense_id}",
    operation_id = "getExpense",
    tag = crate::TREASURY_TAG,
    params(
        ("expense_id" = ExpenseId, Path, description = "Expense identifier"),
    ),
    responses(
        (status = 200, description = "The expense", body = inline(DataEnvelope<ExpenseResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Expense not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn get_handler(
    ExpensePath { expense_id }: ExpensePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<ExpenseResponse>, ExpenseFailure> {
    let expense = require_expense_membership(&state, &identity, expense_id).await?;
    require_view_invoices(&state, &identity, expense.organization_id).await?;

    Ok(Response::OK(ExpenseResponse::new(expense, today())))
}

#[utoipa::path(
    patch,
    path = "/api/v1/expenses/{expense_id}",
    operation_id = "updateExpense",
    tag = crate::TREASURY_TAG,
    params(
        ("expense_id" = ExpenseId, Path, description = "Expense identifier"),
    ),
    request_body = UpdateExpenseRequest,
    responses(
        (status = 200, description = "Expense updated, every field is replaced", body = inline(DataEnvelope<ExpenseResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Expense, category, supplier or customer not found"),
        (status = 422, description = "A rule is refused, the field is named"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_handler(
    ExpensePath { expense_id }: ExpensePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<UpdateExpenseRequest>,
) -> Result<Response<ExpenseResponse>, ExpenseFailure> {
    require_expense_membership(&state, &identity, expense_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let expense = state
        .usecase
        .acting_as(user_id)
        .update_expense(UpdateExpenseCommand {
            actor,
            id: expense_id,
            date: payload.date,
            label: payload.label,
            supplier_id: payload.supplier_id,
            category_id: payload.category_id,
            reference: payload.reference,
            amount_ht_cents: payload.amount_ht_cents,
            vat_cents: payload.vat_cents,
            due_date: payload.due_date,
            customer_id: payload.customer_id,
            proof_file_key: payload.proof_file_key,
            note: payload.note,
        })
        .await??;

    Ok(Response::OK(ExpenseResponse::new(expense, today())))
}

#[utoipa::path(
    delete,
    path = "/api/v1/expenses/{expense_id}",
    operation_id = "deleteExpense",
    tag = crate::TREASURY_TAG,
    params(
        ("expense_id" = ExpenseId, Path, description = "Expense identifier"),
    ),
    responses(
        (status = 204, description = "Expense deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Expense not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_handler(
    ExpensePath { expense_id }: ExpensePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<crate::EmptyResponse>, ExpenseFailure> {
    require_expense_membership(&state, &identity, expense_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    state
        .usecase
        .acting_as(user_id)
        .delete_expense(expense_id, actor)
        .await?;

    Ok(Response::NoContent)
}

#[utoipa::path(
    post,
    path = "/api/v1/expenses/{expense_id}/pay",
    operation_id = "payExpense",
    tag = crate::TREASURY_TAG,
    params(
        ("expense_id" = ExpenseId, Path, description = "Expense identifier"),
    ),
    request_body = PayExpenseRequest,
    responses(
        (status = 200, description = "Expense marked paid", body = inline(DataEnvelope<ExpenseResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Expense not found"),
        (status = 409, description = "The expense is already paid or was advanced by a member"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn pay_handler(
    ExpensePayPath { expense_id }: ExpensePayPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<PayExpenseRequest>,
) -> Result<Response<ExpenseResponse>, ExpenseFailure> {
    require_expense_membership(&state, &identity, expense_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let expense = state
        .usecase
        .acting_as(user_id)
        .mark_expense_paid(MarkExpensePaidCommand {
            actor,
            id: expense_id,
            paid_on: payload.paid_on,
            method: payload.method,
        })
        .await??;

    Ok(Response::OK(ExpenseResponse::new(expense, today())))
}

#[utoipa::path(
    post,
    path = "/api/v1/expenses/{expense_id}/reimburse",
    operation_id = "reimburseExpense",
    tag = crate::TREASURY_TAG,
    params(
        ("expense_id" = ExpenseId, Path, description = "Expense identifier"),
    ),
    responses(
        (status = 200, description = "Advance reimbursed today", body = inline(DataEnvelope<ExpenseResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Expense not found"),
        (status = 409, description = "The expense was not advanced or is already reimbursed"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn reimburse_handler(
    ExpenseReimbursePath { expense_id }: ExpenseReimbursePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<ExpenseResponse>, ExpenseFailure> {
    require_expense_membership(&state, &identity, expense_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let expense = state
        .usecase
        .acting_as(user_id)
        .reimburse_expense(expense_id, actor)
        .await??;

    Ok(Response::OK(ExpenseResponse::new(expense, today())))
}
