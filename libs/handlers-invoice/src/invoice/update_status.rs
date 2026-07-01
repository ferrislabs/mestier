use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{InvoiceId, InvoiceStatus, UpdateInvoiceStatusCommand};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::InvoiceStatusPath, require_invoice_membership, response::InvoiceResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateInvoiceStatusRequest {
	pub status: InvoiceStatus,
}

#[utoipa::path(
	patch,
	path = "/api/v1/invoices/{invoice_id}/status",
	operation_id = "updateInvoiceStatus",
	tag = super::super::TAG,
	params(
		("invoice_id" = InvoiceId, Path, description = "Invoice identifier"),
	),
	request_body = UpdateInvoiceStatusRequest,
	responses(
		(status = 200, description = "Invoice status updated", body = inline(DataEnvelope<InvoiceResponse>)),
		(status = 400, description = "Validation failed"),
		(status = 401, description = "Unauthorized"),
		(status = 403, description = "Forbidden"),
		(status = 404, description = "Invoice not found"),
	),
	security(("bearer_auth" = []))
)]
pub async fn handler(
	InvoiceStatusPath { invoice_id }: InvoiceStatusPath,
	State(state): State<AppState>,
	Extension(identity): Extension<Identity>,
	Json(payload): Json<UpdateInvoiceStatusRequest>,
) -> Result<Response<InvoiceResponse>, ApiError> {
	require_invoice_membership(&state, &identity, invoice_id).await?;

	let invoice = state
		.usecase
		.update_invoice_status(UpdateInvoiceStatusCommand {
			id: invoice_id,
			status: payload.status,
		})
		.await?;

	Ok(Response::OK(invoice.into()))
}
