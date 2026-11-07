use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{AppState, DataEnvelope, Response};
use mestier_core::{InvoiceId, RemindInvoiceCommand};

use crate::{
    invoice_error::InvoiceFailure, paths::InvoiceRemindPath, require_invoice_membership,
    response::InvoiceResponse,
};

#[utoipa::path(
    post,
    path = "/api/v1/invoices/{invoice_id}/remind",
    operation_id = "remindInvoice",
    tag = super::super::TAG,
    params(
        ("invoice_id" = InvoiceId, Path, description = "Invoice identifier"),
    ),
    responses(
        (status = 200, description = "Reminder recorded with the days late, the balance is unchanged", body = inline(DataEnvelope<InvoiceResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Invoice not found"),
        (status = 409, description = "The invoice is not issued (E_CONFLICT)"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    InvoiceRemindPath { invoice_id }: InvoiceRemindPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<InvoiceResponse>, InvoiceFailure> {
    require_invoice_membership(&state, &identity, invoice_id).await?;
    let (user_id, actor) = handlers::resolve_actor(&state, &identity).await?;

    let invoice = state
        .usecase
        .acting_as(user_id)
        .remind_invoice(RemindInvoiceCommand {
            id: invoice_id,
            actor,
            recorded_by: user_id,
        })
        .await??;

    Ok(Response::OK(invoice.into()))
}
