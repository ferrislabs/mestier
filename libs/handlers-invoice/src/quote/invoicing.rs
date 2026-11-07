use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::QuoteId;

use crate::{paths::QuoteInvoicingPath, require_view_invoices, response::QuoteInvoicingResponse};

#[utoipa::path(
    get,
    path = "/api/v1/quotes/{quote_id}/invoicing",
    operation_id = "getQuoteInvoicing",
    tag = super::super::TAG,
    params(
        ("quote_id" = QuoteId, Path, description = "Quote identifier"),
    ),
    responses(
        (status = 200, description = "What can still be invoiced on the quote, and its active invoices", body = inline(DataEnvelope<QuoteInvoicingResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Quote not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    QuoteInvoicingPath { quote_id }: QuoteInvoicingPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<QuoteInvoicingResponse>, ApiError> {
    let view = state.usecase.quote_invoicing(quote_id).await?;
    require_view_invoices(&state, &identity, view.organization_id).await?;

    Ok(Response::OK(view.into()))
}
