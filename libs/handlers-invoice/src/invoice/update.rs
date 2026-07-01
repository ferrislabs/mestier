use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{CustomerContextId, CustomerId, InvoiceId, UpdateInvoiceCommand};
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
	invoice::create::{InvoiceLineRequest, into_legal_mention_template_ids, into_line_commands},
	paths::InvoicePath,
	require_invoice_membership, require_invoice_targets,
	response::InvoiceResponse,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateInvoiceRequest {
	pub title: String,
	pub customer_id: CustomerId,
	pub customer_context_id: CustomerContextId,
	pub lines: Vec<InvoiceLineRequest>,
	#[serde(default)]
	pub legal_mention_template_ids: Option<Vec<Uuid>>,
}

#[utoipa::path(
	patch,
	path = "/api/v1/invoices/{invoice_id}",
	operation_id = "updateInvoice",
	tag = super::super::TAG,
	params(
		("invoice_id" = InvoiceId, Path, description = "Invoice identifier"),
	),
	request_body = UpdateInvoiceRequest,
	responses(
		(status = 200, description = "Invoice updated", body = inline(DataEnvelope<InvoiceResponse>)),
		(status = 400, description = "Validation failed"),
		(status = 401, description = "Unauthorized"),
		(status = 403, description = "Forbidden"),
		(status = 404, description = "Invoice not found"),
		(status = 409, description = "Invoice conflict"),
	),
	security(("bearer_auth" = []))
)]
pub async fn handler(
	InvoicePath { invoice_id }: InvoicePath,
	State(state): State<AppState>,
	Extension(identity): Extension<Identity>,
	Json(payload): Json<UpdateInvoiceRequest>,
) -> Result<Response<InvoiceResponse>, ApiError> {
	let current = require_invoice_membership(&state, &identity, invoice_id).await?;
	require_invoice_targets(
		&state,
		current.org_id,
		payload.customer_id,
		payload.customer_context_id,
	)
	.await?;

	let invoice = state
		.usecase
		.update_invoice(UpdateInvoiceCommand {
			id: invoice_id,
			title: payload.title,
			customer_id: payload.customer_id,
			customer_context_id: payload.customer_context_id,
			deposit_basis: None,
			deposit_value: None,
			deposit_amount_cents: None,
			due_at: None,
			lines: into_line_commands(payload.lines)?,
			legal_mention_template_ids: into_legal_mention_template_ids(
				payload.legal_mention_template_ids,
			),
		})
		.await?;

	Ok(Response::OK(invoice.into()))
}
