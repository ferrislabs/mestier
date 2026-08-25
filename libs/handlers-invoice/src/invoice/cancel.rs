use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_user_id};
use mestier_core::{CancelInvoiceCommand, InvoiceId};

use crate::{paths::InvoiceCancelPath, require_invoice_membership, response::InvoiceResponse};

#[utoipa::path(
    post,
    path = "/api/v1/invoices/{invoice_id}/cancel",
    operation_id = "cancelInvoice",
    tag = super::super::TAG,
    params(
        ("invoice_id" = InvoiceId, Path, description = "Invoice identifier"),
    ),
    responses(
        (status = 200, description = "Invoice cancelled", body = inline(DataEnvelope<InvoiceResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Invoice not found"),
        (status = 409, description = "Invoice is already cancelled, paid or partially paid"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    InvoiceCancelPath { invoice_id }: InvoiceCancelPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<InvoiceResponse>, ApiError> {
    require_invoice_membership(&state, &identity, invoice_id).await?;
    let actor = resolve_user_id(&state, &identity).await?;

    let invoice = state
        .usecase
        .acting_as(actor)
        .cancel_invoice(CancelInvoiceCommand { id: invoice_id })
        .await?;

    Ok(Response::OK(invoice.into()))
}
