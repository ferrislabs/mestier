use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{AppState, DataEnvelope, Response};
use mestier_core::{InvoiceId, SendInvoiceCommand};

use crate::{
    invoice_error::InvoiceFailure, paths::InvoiceSendPath, require_invoice_membership,
    response::InvoiceResponse,
};

#[utoipa::path(
    post,
    path = "/api/v1/invoices/{invoice_id}/send",
    operation_id = "sendInvoice",
    tag = super::super::TAG,
    params(
        ("invoice_id" = InvoiceId, Path, description = "Invoice identifier"),
    ),
    responses(
        (status = 200, description = "Sending recorded: sent_at and a SENT event, no e-mail is delivered", body = inline(DataEnvelope<InvoiceResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Invoice not found"),
        (status = 409, description = "The invoice is not issued (E_CONFLICT)"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    InvoiceSendPath { invoice_id }: InvoiceSendPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<InvoiceResponse>, InvoiceFailure> {
    require_invoice_membership(&state, &identity, invoice_id).await?;
    let (user_id, actor) = handlers::resolve_actor(&state, &identity).await?;

    let invoice = state
        .usecase
        .acting_as(user_id)
        .send_invoice(SendInvoiceCommand {
            id: invoice_id,
            actor,
            recorded_by: user_id,
        })
        .await??;

    Ok(Response::OK(invoice.into()))
}
