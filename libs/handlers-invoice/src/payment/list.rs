use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::InvoiceId;

use crate::{
    paths::InvoicePaymentsPath, require_invoice_membership, response::InvoicePaymentResponse,
};

#[utoipa::path(
    get,
    path = "/api/v1/invoices/{invoice_id}/payments",
    operation_id = "listInvoicePayments",
    tag = super::super::TAG,
    params(
        ("invoice_id" = InvoiceId, Path, description = "Invoice identifier"),
    ),
    responses(
        (status = 200, description = "Every non-deleted payment against this invoice", body = inline(DataEnvelope<Vec<InvoicePaymentResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Invoice not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    InvoicePaymentsPath { invoice_id }: InvoicePaymentsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<InvoicePaymentResponse>>, ApiError> {
    require_invoice_membership(&state, &identity, invoice_id).await?;

    let payments = state.usecase.list_invoice_payments(invoice_id).await?;
    let items: Vec<InvoicePaymentResponse> = payments
        .into_iter()
        .map(InvoicePaymentResponse::from)
        .collect();

    Ok(Response::OK(items))
}
