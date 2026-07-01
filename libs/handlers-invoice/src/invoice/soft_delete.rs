use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, Response};
use mestier_core::InvoiceId;

use crate::{EmptyResponse, paths::InvoicePath, require_invoice_membership};

#[utoipa::path(
	delete,
	path = "/api/v1/invoices/{invoice_id}",
	operation_id = "deleteInvoice",
	tag = super::super::TAG,
	params(
		("invoice_id" = InvoiceId, Path, description = "Invoice identifier"),
	),
	responses(
		(status = 204, description = "Invoice soft-deleted"),
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
) -> Result<Response<EmptyResponse>, ApiError> {
	require_invoice_membership(&state, &identity, invoice_id).await?;
	state.usecase.soft_delete_invoice(invoice_id).await?;

	Ok(Response::NoContent)
}
