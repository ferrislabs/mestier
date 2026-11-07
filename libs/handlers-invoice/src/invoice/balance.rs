use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::InvoiceId;

use crate::{paths::InvoiceBalancePath, require_view_invoices, response::InvoiceBalanceResponse};

#[utoipa::path(
    get,
    path = "/api/v1/invoices/{invoice_id}/balance",
    operation_id = "getInvoiceBalance",
    tag = super::super::TAG,
    params(
        ("invoice_id" = InvoiceId, Path, description = "Invoice identifier"),
    ),
    responses(
        (status = 200, description = "What remains to be collected on this invoice", body = inline(DataEnvelope<InvoiceBalanceResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Invoice not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    InvoiceBalancePath { invoice_id }: InvoiceBalancePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<InvoiceBalanceResponse>, ApiError> {
    let invoice = state.usecase.get_invoice(invoice_id).await?;
    require_view_invoices(&state, &identity, invoice.organization_id).await?;

    let settlement = invoice.derived.settlement;

    Ok(Response::OK(InvoiceBalanceResponse {
        gross_cents: invoice.gross_cents,
        credited_cents: settlement.credited_cents,
        paid_cents: settlement.paid_cents,
        remaining_cents: settlement.balance_cents,
    }))
}
