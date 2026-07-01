use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::QuoteId;

use crate::{paths::QuoteConvertPath, require_org_membership, response::InvoiceResponse};

async fn require_quote_org_membership(
	state: &AppState,
	identity: &Identity,
	quote_id: QuoteId,
) -> Result<(), ApiError> {
	let quote = state.usecase.get_quote(quote_id).await?;
	require_org_membership(state, identity, quote.organization_id).await
}

#[utoipa::path(
	post,
	path = "/api/v1/quotes/{quote_id}/convert-to-invoice",
	operation_id = "convertQuoteToInvoice",
	tag = super::super::TAG,
	params(
		("quote_id" = QuoteId, Path, description = "Quote identifier"),
	),
	responses(
		(status = 201, description = "Invoice created from quote", body = inline(DataEnvelope<InvoiceResponse>)),
		(status = 401, description = "Unauthorized"),
		(status = 403, description = "Forbidden"),
		(status = 404, description = "Quote not found"),
		(status = 409, description = "Conversion conflict"),
	),
	security(("bearer_auth" = []))
)]
pub async fn handler(
	QuoteConvertPath { quote_id }: QuoteConvertPath,
	State(state): State<AppState>,
	Extension(identity): Extension<Identity>,
) -> Result<Response<InvoiceResponse>, ApiError> {
	require_quote_org_membership(&state, &identity, quote_id).await?;

	let invoice = state
		.usecase
		.convert_quote_to_invoice(quote_id)
		.await?;

	Ok(Response::Created(invoice.into()))
}
