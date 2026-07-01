use auth::Identity;
use axum::{Extension, extract::State};
use chrono::Utc;
use common::sign_document_token;
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::InvoiceId;
use serde::Serialize;
use utoipa::ToSchema;

use crate::{paths::InvoiceSharePath, require_invoice_membership};

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct ShareLinkResponse {
	pub token: String,
	pub path: String,
	pub expires_at: i64,
}

#[utoipa::path(
	post,
	path = "/api/v1/invoices/{invoice_id}/share",
	operation_id = "shareInvoice",
	tag = super::super::TAG,
	params(
		("invoice_id" = InvoiceId, Path, description = "Invoice identifier"),
	),
	responses(
		(status = 200, description = "Share link generated", body = inline(DataEnvelope<ShareLinkResponse>)),
		(status = 401, description = "Unauthorized"),
		(status = 403, description = "Forbidden"),
		(status = 404, description = "Invoice not found"),
		(status = 409, description = "Invoice has not been issued yet"),
	),
	security(("bearer_auth" = []))
)]
pub async fn handler(
	InvoiceSharePath { invoice_id }: InvoiceSharePath,
	State(state): State<AppState>,
	Extension(identity): Extension<Identity>,
) -> Result<Response<ShareLinkResponse>, ApiError> {
	let invoice = require_invoice_membership(&state, &identity, invoice_id).await?;

	if invoice.pdf_key.is_none() {
		return Err(ApiError::Conflict(
			"invoice has not been issued yet".to_owned(),
		));
	}

	let expires_at = Utc::now().timestamp() + 7 * 24 * 60 * 60;

	let token = sign_document_token(
		invoice_id.0,
		expires_at,
		state.document_signing_secret.as_bytes(),
	);

	let path = format!("/api/v1/public/documents/{token}");

	Ok(Response::OK(ShareLinkResponse {
		token,
		path,
		expires_at,
	}))
}
