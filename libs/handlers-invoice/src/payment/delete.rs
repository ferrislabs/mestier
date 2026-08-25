use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, Response, resolve_user_id};
use mestier_core::{DeleteInvoicePaymentCommand, InvoicePaymentId};

use crate::{EmptyResponse, paths::InvoicePaymentPath, require_payment_membership};

#[utoipa::path(
    delete,
    path = "/api/v1/invoice-payments/{invoice_payment_id}",
    operation_id = "deleteInvoicePayment",
    tag = super::super::TAG,
    params(
        ("invoice_payment_id" = InvoicePaymentId, Path, description = "Payment identifier"),
    ),
    responses(
        (status = 204, description = "Payment soft-deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Payment not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    InvoicePaymentPath { invoice_payment_id }: InvoicePaymentPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<EmptyResponse>, ApiError> {
    require_payment_membership(&state, &identity, invoice_payment_id).await?;
    let actor = resolve_user_id(&state, &identity).await?;

    state
        .usecase
        .acting_as(actor)
        .delete_invoice_payment(DeleteInvoicePaymentCommand {
            id: invoice_payment_id,
            deleted_by: actor,
        })
        .await?;

    Ok(Response::NoContent)
}
