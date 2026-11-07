use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::{
    CreateExpenseCategoryCommand, ExpenseCategoryId, OrganizationId, UpdateExpenseCategoryCommand,
};
use serde::Deserialize;
use utoipa::ToSchema;

use super::{
    error::ExpenseFailure,
    paths::{ExpenseCategoryPath, OrganizationExpenseCategoriesPath},
    response::ExpenseCategoryResponse,
};
use crate::{require_category_membership, require_org_membership, require_view_invoices};

#[derive(Debug, Deserialize, ToSchema)]
pub struct ExpenseCategoryRequest {
    pub name: String,
    #[serde(default)]
    pub hue: i64,
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/expense-categories",
    operation_id = "listExpenseCategories",
    tag = crate::TREASURY_TAG,
    params(
        ("organization_id" = OrganizationId, Path, description = "Organization identifier"),
    ),
    responses(
        (status = 200, description = "The categories, the default ones are created on first read", body = inline(DataEnvelope<Vec<ExpenseCategoryResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn list_handler(
    OrganizationExpenseCategoriesPath { organization_id }: OrganizationExpenseCategoriesPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<ExpenseCategoryResponse>>, ExpenseFailure> {
    require_view_invoices(&state, &identity, organization_id).await?;

    let categories = state
        .usecase
        .list_expense_categories(organization_id)
        .await?;

    Ok(Response::OK(
        categories.into_iter().map(Into::into).collect(),
    ))
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/expense-categories",
    operation_id = "createExpenseCategory",
    tag = crate::TREASURY_TAG,
    params(
        ("organization_id" = OrganizationId, Path, description = "Organization identifier"),
    ),
    request_body = ExpenseCategoryRequest,
    responses(
        (status = 201, description = "Category created", body = inline(DataEnvelope<ExpenseCategoryResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 409, description = "A category of this name exists"),
        (status = 422, description = "A rule is refused, the field is named"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn create_handler(
    OrganizationExpenseCategoriesPath { organization_id }: OrganizationExpenseCategoriesPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<ExpenseCategoryRequest>,
) -> Result<Response<ExpenseCategoryResponse>, ExpenseFailure> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let category = state
        .usecase
        .acting_as(user_id)
        .create_expense_category(CreateExpenseCategoryCommand {
            actor,
            organization_id,
            name: payload.name,
            hue: payload.hue,
        })
        .await??;

    Ok(Response::Created(category.into()))
}

#[utoipa::path(
    patch,
    path = "/api/v1/expense-categories/{category_id}",
    operation_id = "updateExpenseCategory",
    tag = crate::TREASURY_TAG,
    params(
        ("category_id" = ExpenseCategoryId, Path, description = "Category identifier"),
    ),
    request_body = ExpenseCategoryRequest,
    responses(
        (status = 200, description = "Category updated", body = inline(DataEnvelope<ExpenseCategoryResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Category not found"),
        (status = 409, description = "A category of this name exists"),
        (status = 422, description = "A rule is refused, the field is named"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn update_handler(
    ExpenseCategoryPath { category_id }: ExpenseCategoryPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<ExpenseCategoryRequest>,
) -> Result<Response<ExpenseCategoryResponse>, ExpenseFailure> {
    require_category_membership(&state, &identity, category_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let category = state
        .usecase
        .acting_as(user_id)
        .update_expense_category(UpdateExpenseCategoryCommand {
            actor,
            id: category_id,
            name: payload.name,
            hue: payload.hue,
        })
        .await??;

    Ok(Response::OK(category.into()))
}

#[utoipa::path(
    delete,
    path = "/api/v1/expense-categories/{category_id}",
    operation_id = "deleteExpenseCategory",
    tag = crate::TREASURY_TAG,
    params(
        ("category_id" = ExpenseCategoryId, Path, description = "Category identifier"),
    ),
    responses(
        (status = 204, description = "Category deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Category not found"),
        (status = 409, description = "An expense or a supplier uses the category"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn delete_handler(
    ExpenseCategoryPath { category_id }: ExpenseCategoryPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<crate::EmptyResponse>, ExpenseFailure> {
    require_category_membership(&state, &identity, category_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    state
        .usecase
        .acting_as(user_id)
        .delete_expense_category(category_id, actor)
        .await??;

    Ok(Response::NoContent)
}
