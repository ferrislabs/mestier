use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::{
    paths::OrganizationInvoicesOutstandingPath, require_view_invoices,
    response::CustomerOutstandingBalanceResponse,
};

/// The dunning read #320 built (`MestierUseCase::outstanding_balance_by_
/// customer`) and #319 left unwired — no route reached it until now, even
/// though `CustomerOutstandingBalanceResponse` already existed for it. One
/// number per customer, not per invoice, so the invoice list can show an
/// outstanding total without an N+1 over every row.
#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/invoices/outstanding",
    operation_id = "listOutstandingBalanceByCustomer",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    responses(
        (status = 200, description = "What each customer still owes, across every issued invoice", body = inline(DataEnvelope<Vec<CustomerOutstandingBalanceResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    OrganizationInvoicesOutstandingPath { organization_id }: OrganizationInvoicesOutstandingPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<CustomerOutstandingBalanceResponse>>, ApiError> {
    require_view_invoices(&state, &identity, organization_id).await?;

    let balances = state
        .usecase
        .outstanding_balance_by_customer(organization_id)
        .await?;
    let items: Vec<CustomerOutstandingBalanceResponse> = balances
        .into_iter()
        .map(CustomerOutstandingBalanceResponse::from)
        .collect();

    Ok(Response::OK(items))
}
