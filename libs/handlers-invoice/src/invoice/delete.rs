use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{AppState, Response};
use mestier_core::InvoiceId;

use crate::{
    EmptyResponse, invoice_error::InvoiceFailure, paths::InvoicePath, require_invoice_membership,
};

#[utoipa::path(
    delete,
    path = "/api/v1/invoices/{invoice_id}",
    operation_id = "deleteInvoice",
    tag = super::super::TAG,
    params(
        ("invoice_id" = InvoiceId, Path, description = "Invoice identifier"),
    ),
    responses(
        (status = 204, description = "Draft invoice deleted, no number consumed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Invoice not found"),
        (status = 409, description = "The invoice is not a draft (E_INVOICE_LOCKED)"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    InvoicePath { invoice_id }: InvoicePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<EmptyResponse>, InvoiceFailure> {
    require_invoice_membership(&state, &identity, invoice_id).await?;
    let (user_id, actor) = handlers::resolve_actor(&state, &identity).await?;

    state
        .usecase
        .acting_as(user_id)
        .soft_delete_invoice(invoice_id, actor)
        .await??;

    Ok(Response::NoContent)
}
