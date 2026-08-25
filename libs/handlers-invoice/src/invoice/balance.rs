use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{InvoiceId, InvoiceStatus};

use crate::{
    paths::InvoiceBalancePath, require_invoice_membership, response::InvoiceBalanceResponse,
};

/// "What remains" — computed here, in Rust, before the JSON is built
/// (CLAUDE.md: every price calculation lives in the backend, never
/// recomputed in the browser). `credited_cents` filters out draft and
/// cancelled credit notes the same way `InvoiceService::sum_already_issued_
/// cents` does; `paid_cents` sums every payment `list_invoice_payments`
/// returns, which is already only the non-deleted ones (see the query
/// backing `payment::list::handler`).
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
    let invoice = require_invoice_membership(&state, &identity, invoice_id).await?;

    let credit_notes = state.usecase.get_invoice_credit_notes(invoice_id).await?;
    let credited_cents: i64 = credit_notes
        .into_iter()
        .filter(|credit_note| {
            !matches!(
                credit_note.status,
                InvoiceStatus::Draft | InvoiceStatus::Cancelled
            )
        })
        .map(|credit_note| i64::from(credit_note.gross_cents))
        .sum();

    let payments = state.usecase.list_invoice_payments(invoice_id).await?;
    let paid_cents: i64 = payments
        .into_iter()
        .map(|payment| i64::from(payment.amount_cents))
        .sum();

    let remaining_cents = i64::from(invoice.gross_cents) - credited_cents - paid_cents;

    Ok(Response::OK(InvoiceBalanceResponse {
        gross_cents: invoice.gross_cents,
        credited_cents,
        paid_cents,
        remaining_cents,
    }))
}
