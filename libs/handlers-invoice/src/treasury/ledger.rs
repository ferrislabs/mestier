use std::str::FromStr;

use auth::Identity;
use axum::{
    Extension, Json,
    extract::{Query, State},
};
use chrono::NaiveDate;
use handlers::{AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::{
    AllocateReceiptCommand, CustomerId, InvoiceId, LedgerMonth, PaymentMethod, RecordReceiptCommand,
};
use serde::Deserialize;
use utoipa::{IntoParams, ToSchema};

use super::{
    error::LedgerFailure,
    paths::{OrganizationLedgerPath, OrganizationReceiptsPath, ReceiptAllocatePath},
    response::{LedgerResponse, ReceiptResponse},
};
use crate::{
    invoice_error::InvoiceFailure, require_org_membership, require_receipt_membership,
    require_view_invoices, response::InvoicePaymentResponse,
};

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct LedgerQuery {
    #[serde(default)]
    pub month: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct RecordReceiptRequest {
    #[serde(default)]
    pub customer_id: Option<CustomerId>,
    pub amount_cents: i32,
    pub paid_on: NaiveDate,
    pub method: PaymentMethod,
    #[serde(default)]
    pub reference: Option<String>,
    #[serde(default)]
    pub note: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct AllocateReceiptRequest {
    pub invoice_id: InvoiceId,
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/ledger",
    operation_id = "readLedger",
    tag = crate::TREASURY_TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        LedgerQuery,
    ),
    responses(
        (status = 200, description = "The journal, its summary, the six-month trend and the VAT", body = inline(DataEnvelope<LedgerResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 422, description = "The month is not formatted YYYY-MM"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn read_handler(
    OrganizationLedgerPath { organization_id }: OrganizationLedgerPath,
    Query(query): Query<LedgerQuery>,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<LedgerResponse>, LedgerFailure> {
    require_view_invoices(&state, &identity, organization_id).await?;
    let month = query
        .month
        .as_deref()
        .filter(|month| !month.is_empty())
        .map(LedgerMonth::from_str)
        .transpose()
        .map_err(|_| LedgerFailure::BadMonth)?;

    let ledger = state.usecase.read_ledger(organization_id, month).await?;

    Ok(Response::OK(ledger.into()))
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/receipts",
    operation_id = "recordReceipt",
    tag = crate::TREASURY_TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    request_body = RecordReceiptRequest,
    responses(
        (status = 201, description = "Receipt recorded, not yet allocated to an invoice", body = inline(DataEnvelope<ReceiptResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Customer not found"),
        (status = 422, description = "A rule is refused, the field is named"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn record_handler(
    OrganizationReceiptsPath { organization_id }: OrganizationReceiptsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<RecordReceiptRequest>,
) -> Result<Response<ReceiptResponse>, LedgerFailure> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let receipt = state
        .usecase
        .acting_as(user_id)
        .record_receipt(RecordReceiptCommand {
            actor,
            organization_id,
            customer_id: payload.customer_id,
            amount_cents: payload.amount_cents,
            paid_on: payload.paid_on,
            method: payload.method,
            reference: payload.reference,
            note: payload.note,
            recorded_by: user_id,
        })
        .await??;

    Ok(Response::Created(receipt.into()))
}

#[utoipa::path(
    post,
    path = "/api/v1/receipts/{receipt_id}/allocate",
    operation_id = "allocateReceipt",
    tag = crate::TREASURY_TAG,
    params(
        ("receipt_id" = mestier_core::ReceiptId, Path, description = "Receipt identifier"),
    ),
    request_body = AllocateReceiptRequest,
    responses(
        (status = 200, description = "The receipt became a payment of the invoice", body = inline(DataEnvelope<InvoicePaymentResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Receipt or invoice not found"),
        (status = 409, description = "The invoice is not issued"),
        (status = 422, description = "The receipt exceeds the balance of the invoice"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn allocate_handler(
    ReceiptAllocatePath { receipt_id }: ReceiptAllocatePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<AllocateReceiptRequest>,
) -> Result<Response<InvoicePaymentResponse>, InvoiceFailure> {
    require_receipt_membership(&state, &identity, receipt_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let payment = state
        .usecase
        .acting_as(user_id)
        .allocate_receipt(AllocateReceiptCommand {
            actor,
            id: receipt_id,
            invoice_id: payload.invoice_id,
            recorded_by: user_id,
        })
        .await??;

    Ok(Response::OK(payment.into()))
}
