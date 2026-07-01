use std::str::FromStr;

use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{
	CreateInvoiceCommand, CustomerContextId, CustomerId, InvoiceLineCommand, InvoiceType,
	LegalMentionTemplateId, OrganizationContextId, ServiceRateId, ServiceRateUnit,
};
use rust_decimal::Decimal;
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
	paths::InvoicesPath, require_invoice_targets, require_org_membership,
	response::InvoiceResponse,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct InvoiceLineRequest {
	pub service_rate_id: Option<ServiceRateId>,
	pub label: String,
	pub quantity: String,
	pub unit: ServiceRateUnit,
	pub unit_price_cents: i32,
	pub vat_rate: Option<String>,
	pub notes: Option<String>,
	pub photo_keys: Vec<String>,
}

impl TryFrom<InvoiceLineRequest> for InvoiceLineCommand {
	type Error = ApiError;

	fn try_from(value: InvoiceLineRequest) -> Result<Self, Self::Error> {
		let quantity = Decimal::from_str(&value.quantity)
			.map_err(|_| ApiError::Validation("invoice line quantity must be decimal".to_owned()))?;

		let vat_rate = match value.vat_rate {
			Some(s) => Decimal::from_str(&s).map_err(|_| {
				ApiError::Validation("invoice line vat_rate must be decimal".to_owned())
			})?,
			None => Decimal::from(20u32),
		};

		Ok(Self {
			service_rate_id: value.service_rate_id,
			label: value.label,
			quantity,
			unit: value.unit,
			unit_price_cents: value.unit_price_cents,
			vat_rate,
			notes: value.notes,
			photo_keys: value.photo_keys,
		})
	}
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateInvoiceRequest {
	pub title: String,
	pub customer_id: CustomerId,
	pub customer_context_id: CustomerContextId,
	#[serde(default)]
	pub emitter_context_id: Option<Uuid>,
	pub lines: Vec<InvoiceLineRequest>,
	#[serde(default)]
	pub legal_mention_template_ids: Option<Vec<Uuid>>,
}

pub fn into_legal_mention_template_ids(ids: Option<Vec<Uuid>>) -> Vec<LegalMentionTemplateId> {
	ids.unwrap_or_default()
		.into_iter()
		.map(LegalMentionTemplateId)
		.collect()
}

pub fn into_line_commands(
	lines: Vec<InvoiceLineRequest>,
) -> Result<Vec<InvoiceLineCommand>, ApiError> {
	lines.into_iter().map(TryInto::try_into).collect()
}

#[utoipa::path(
	post,
	path = "/api/v1/organizations/{organization_id}/invoices",
	operation_id = "createInvoice",
	tag = super::super::TAG,
	params(
		("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
	),
	request_body = CreateInvoiceRequest,
	responses(
		(status = 201, description = "Invoice created", body = inline(DataEnvelope<InvoiceResponse>)),
		(status = 400, description = "Validation failed"),
		(status = 401, description = "Unauthorized"),
		(status = 403, description = "Forbidden"),
		(status = 409, description = "Invoice conflict"),
	),
	security(("bearer_auth" = []))
)]
pub async fn handler(
	path: InvoicesPath,
	State(state): State<AppState>,
	Extension(identity): Extension<Identity>,
	Json(payload): Json<CreateInvoiceRequest>,
) -> Result<Response<InvoiceResponse>, ApiError> {
	require_org_membership(&state, &identity, path.organization_id).await?;
	require_invoice_targets(
		&state,
		path.organization_id,
		payload.customer_id,
		payload.customer_context_id,
	)
	.await?;

	let invoice = state
		.usecase
		.create_invoice(CreateInvoiceCommand {
			org_id: path.organization_id,
			title: payload.title,
			customer_id: payload.customer_id,
			customer_context_id: payload.customer_context_id,
			emitter_context_id: payload.emitter_context_id.map(OrganizationContextId),
			invoice_type: InvoiceType::Standard,
			source_quote_id: None,
			parent_invoice_id: None,
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

	Ok(Response::Created(invoice.into()))
}
