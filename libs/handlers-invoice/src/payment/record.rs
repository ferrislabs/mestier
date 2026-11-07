use auth::Identity;
use axum::{Extension, Json, extract::State};
use chrono::NaiveDate;
use handlers::{AppState, DataEnvelope, Response};
use mestier_core::{InvoiceId, PaymentMethod, RecordInvoicePaymentCommand};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    invoice_error::InvoiceFailure, paths::InvoicePaymentsPath, require_invoice_membership,
    response::InvoicePaymentResponse,
};

/// `recorded_by` is never a request field — it comes from `handlers::
/// resolve_actor`, same as `acting_as(user_id)` everywhere else in this
/// codebase.
#[derive(Debug, Deserialize, ToSchema)]
pub struct RecordInvoicePaymentRequest {
    pub amount_cents: i32,
    pub paid_on: NaiveDate,
    pub method: PaymentMethod,
    pub reference: Option<String>,
    pub note: Option<String>,
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
        (status = 409, description = "The invoice is not issued (E_CONFLICT)"),
        (status = 422, description = "A rule refused the payment: E_PAYMENT_EXCEEDS_BALANCE, E_REFUND_EXCEEDS_COLLECTED, a zero amount or a future date"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    InvoicePaymentsPath { invoice_id }: InvoicePaymentsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<RecordInvoicePaymentRequest>,
) -> Result<Response<InvoicePaymentResponse>, InvoiceFailure> {
    require_invoice_membership(&state, &identity, invoice_id).await?;
    let (user_id, actor) = handlers::resolve_actor(&state, &identity).await?;

    let payment = state
        .usecase
        .acting_as(user_id)
        .record_invoice_payment(RecordInvoicePaymentCommand {
            invoice_id,
            actor,
            amount_cents: payload.amount_cents,
            paid_on: payload.paid_on,
            method: payload.method,
            reference: payload.reference,
            note: payload.note,
            recorded_by: user_id,
        })
        .await??;

    Ok(Response::Created(payment.into()))
}
