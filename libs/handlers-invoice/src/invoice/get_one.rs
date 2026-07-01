use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::InvoiceId;

use crate::{paths::InvoicePath, require_invoice_membership, response::InvoiceResponse};

#[utoipa::path(
	get,
	path = "/api/v1/invoices/{invoice_id}",
	operation_id = "getOneInvoice",
	tag = super::super::TAG,
	params(
		("invoice_id" = InvoiceId, Path, description = "Invoice identifier"),
	),
	responses(
		(status = 200, description = "Invoice details", body = inline(DataEnvelope<InvoiceResponse>)),
		(status = 401, description = "Unauthorized"),
		(status = 403, description = "Forbidden"),
		(status = 404, description = "Invoice not found"),
	),
	security(("bearer_auth" = []))
)]
pub async fn handler(
	InvoicePath { invoice_id }: InvoicePath,
	State(state): State<AppState>,
	Extension(identity): Extension<Identity>,
) -> Result<Response<InvoiceResponse>, ApiError> {
	let invoice = require_invoice_membership(&state, &identity, invoice_id).await?;

	Ok(Response::OK(invoice.into()))
}
