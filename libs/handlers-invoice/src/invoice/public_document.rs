use axum::{
	body::Body,
	extract::State,
	response::{IntoResponse, Response},
};
use chrono::Utc;
use common::{DocumentTokenError, verify_document_token};
use handlers::{ApiError, AppState};
use http::{
	StatusCode,
	header::{CONTENT_DISPOSITION, CONTENT_TYPE},
};
use mestier_core::InvoiceId;
use uuid::Uuid;

use crate::paths::PublicDocumentPath;

#[utoipa::path(
	get,
	path = "/api/v1/public/documents/{token}",
	operation_id = "getPublicDocument",
	tag = super::super::TAG,
	params(
		("token" = String, Path, description = "Signed document access token"),
	),
	responses(
		(status = 200, description = "Invoice PDF", content_type = "application/pdf"),
		(status = 400, description = "Malformed or expired token"),
		(status = 404, description = "Invoice not found or PDF not available"),
	),
)]
pub async fn handler(
	PublicDocumentPath { token }: PublicDocumentPath,
	State(state): State<AppState>,
) -> Result<Response, ApiError> {
	let now_unix = Utc::now().timestamp();

	let doc_id: Uuid = verify_document_token(
		&token,
		state.document_signing_secret.as_bytes(),
		now_unix,
	)
	.map_err(|e| match e {
		DocumentTokenError::Expired => ApiError::BadRequest("token expired".to_owned()),
		DocumentTokenError::Malformed | DocumentTokenError::InvalidSignature => {
			ApiError::BadRequest("invalid token".to_owned())
		}
	})?;

	let invoice_id = InvoiceId(doc_id);
	let invoice = state.usecase.get_invoice(invoice_id).await?;

	let pdf_key = invoice.pdf_key.ok_or(ApiError::NotFound)?;

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
				format!("attachment; filename=\"{filename}.pdf\""),
			),
		],
		Body::from(file.bytes),
	)
		.into_response())
}
