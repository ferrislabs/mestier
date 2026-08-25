use auth::Identity;
use axum::{Extension, Json, extract::State};
use chrono::NaiveDate;
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_user_id};
use mestier_core::{InvoiceId, RecordInvoicePaymentCommand};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    paths::InvoicePaymentsPath, require_invoice_membership, response::InvoicePaymentResponse,
};

/// `recorded_by` is never a request field — it comes from `resolve_user_id`,
/// same as `acting_as(actor)` everywhere else in this codebase.
#[derive(Debug, Deserialize, ToSchema)]
pub struct RecordInvoicePaymentRequest {
    pub amount_cents: i32,
    pub paid_on: NaiveDate,
    pub method: String,
    pub reference: Option<String>,
    pub note: Option<String>,
    #[serde(default)]
    pub allow_exceeding_total: bool,
}

#[utoipa::path(
    post,
    path = "/api/v1/invoices/{invoice_id}/payments",
    operation_id = "recordInvoicePayment",
    tag = super::super::TAG,
    params(
        ("invoice_id" = InvoiceId, Path, description = "Invoice identifier"),
    ),
    request_body = RecordInvoicePaymentRequest,
    responses(
        (status = 201, description = "Payment recorded", body = inline(DataEnvelope<InvoicePaymentResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Invoice not found"),
        (status = 409, description = "The invoice is not issued, or this payment would exceed its gross total net of credit notes"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    InvoicePaymentsPath { invoice_id }: InvoicePaymentsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<RecordInvoicePaymentRequest>,
) -> Result<Response<InvoicePaymentResponse>, ApiError> {
    require_invoice_membership(&state, &identity, invoice_id).await?;
    let actor = resolve_user_id(&state, &identity).await?;

    let payment = state
        .usecase
        .acting_as(actor)
        .record_invoice_payment(RecordInvoicePaymentCommand {
            invoice_id,
            amount_cents: payload.amount_cents,
            paid_on: payload.paid_on,
            method: payload.method,
            reference: payload.reference,
            note: payload.note,
            recorded_by: actor,
            allow_exceeding_total: payload.allow_exceeding_total,
        })
        .await?;

    Ok(Response::Created(payment.into()))
}
