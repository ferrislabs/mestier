use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::InvoiceId;

use crate::{paths::InvoiceIssuePath, require_invoice_membership, response::InvoiceResponse};

#[utoipa::path(
	post,
	path = "/api/v1/invoices/{invoice_id}/issue",
	operation_id = "issueInvoice",
	tag = super::super::TAG,
	params(
		("invoice_id" = InvoiceId, Path, description = "Invoice identifier"),
	),
	responses(
		(status = 200, description = "Invoice issued", body = inline(DataEnvelope<InvoiceResponse>)),
		(status = 401, description = "Unauthorized"),
		(status = 403, description = "Forbidden"),
		(status = 404, description = "Invoice not found"),
		(status = 409, description = "Invoice is not in DRAFT status"),
	),
	security(("bearer_auth" = []))
)]
pub async fn handler(
	InvoiceIssuePath { invoice_id }: InvoiceIssuePath,
	State(state): State<AppState>,
	Extension(identity): Extension<Identity>,
) -> Result<Response<InvoiceResponse>, ApiError> {
	require_invoice_membership(&state, &identity, invoice_id).await?;

	let invoice = state.usecase.issue_invoice(invoice_id).await?;

	Ok(Response::OK(invoice.into()))
}
