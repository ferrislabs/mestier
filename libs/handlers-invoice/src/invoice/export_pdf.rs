use auth::Identity;
use axum::{
	Extension,
	body::Body,
	extract::State,
	response::{IntoResponse, Response},
};
use handlers::{ApiError, AppState};
use http::{
	StatusCode,
	header::{CONTENT_DISPOSITION, CONTENT_TYPE},
};
use mestier_core::InvoiceId;

use crate::{paths::InvoicePdfPath, require_invoice_membership};

#[utoipa::path(
	get,
	path = "/api/v1/invoices/{invoice_id}/pdf",
	operation_id = "downloadInvoicePdf",
	tag = super::super::TAG,
	params(
		("invoice_id" = InvoiceId, Path, description = "Invoice identifier"),
	),
	responses(
		(status = 200, description = "Invoice PDF", content_type = "application/pdf"),
		(status = 401, description = "Unauthorized"),
		(status = 403, description = "Forbidden"),
		(status = 404, description = "Invoice not found or not yet issued"),
	),
	security(("bearer_auth" = []))
)]
pub async fn handler(
	InvoicePdfPath { invoice_id }: InvoicePdfPath,
	State(state): State<AppState>,
	Extension(identity): Extension<Identity>,
) -> Result<Response, ApiError> {
	let invoice = require_invoice_membership(&state, &identity, invoice_id).await?;

	let pdf_key = invoice.pdf_key.ok_or_else(|| {
		ApiError::NotFound
	})?;

	let file = state.file_storage.get(&pdf_key).await?;

	let filename = invoice
		.reference
		.unwrap_or_else(|| invoice_id.0.to_string());

	Ok((
		StatusCode::OK,
		[
			(CONTENT_TYPE, "application/pdf".to_owned()),
			(
				CONTENT_DISPOSITION,
				format!("inline; filename=\"{filename}.pdf\""),
			),
		],
		Body::from(file.bytes),
	)
		.into_response())
}
