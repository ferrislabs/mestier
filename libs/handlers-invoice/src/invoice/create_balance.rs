use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::InvoiceId;

use crate::{paths::InvoiceBalancePath, require_invoice_membership, response::InvoiceResponse};

#[utoipa::path(
	post,
	path = "/api/v1/invoices/{invoice_id}/balance-invoice",
	operation_id = "createBalanceInvoice",
	tag = super::super::TAG,
	params(
		("invoice_id" = InvoiceId, Path, description = "Parent invoice identifier"),
	),
	responses(
		(status = 201, description = "Balance invoice created", body = inline(DataEnvelope<InvoiceResponse>)),
		(status = 401, description = "Unauthorized"),
		(status = 403, description = "Forbidden"),
		(status = 404, description = "Parent invoice not found"),
		(status = 409, description = "Remaining amount is zero or all already invoiced"),
	),
	security(("bearer_auth" = []))
)]
pub async fn handler(
	InvoiceBalancePath { invoice_id }: InvoiceBalancePath,
	State(state): State<AppState>,
	Extension(identity): Extension<Identity>,
) -> Result<Response<InvoiceResponse>, ApiError> {
	require_invoice_membership(&state, &identity, invoice_id).await?;

	let invoice = state
		.usecase
		.create_balance_invoice(invoice_id)
		.await?;

	Ok(Response::Created(invoice.into()))
}
