use std::str::FromStr;

use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{DepositBasis, InvoiceId};
use rust_decimal::Decimal;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::InvoiceDepositPath, require_invoice_membership, response::InvoiceResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateDepositInvoiceRequest {
	/// "PERCENT" or "FIXED"
	pub basis: String,
	/// Decimal string — percentage (e.g. "30") or fixed amount in cents (e.g. "50000")
	pub value: String,
}

fn parse_basis(s: &str) -> Result<DepositBasis, ApiError> {
	match s {
		"PERCENT" => Ok(DepositBasis::Percent),
		"FIXED" => Ok(DepositBasis::Fixed),
		other => Err(ApiError::Validation(format!(
			"deposit basis must be PERCENT or FIXED, got `{other}`"
		))),
	}
}

#[utoipa::path(
	post,
	path = "/api/v1/invoices/{invoice_id}/deposit-invoice",
	operation_id = "createDepositInvoice",
	tag = super::super::TAG,
	params(
		("invoice_id" = InvoiceId, Path, description = "Parent invoice identifier"),
	),
	request_body = CreateDepositInvoiceRequest,
	responses(
		(status = 201, description = "Deposit invoice created", body = inline(DataEnvelope<InvoiceResponse>)),
		(status = 400, description = "Validation failed"),
		(status = 401, description = "Unauthorized"),
		(status = 403, description = "Forbidden"),
		(status = 404, description = "Parent invoice not found"),
		(status = 409, description = "Conflict"),
	),
	security(("bearer_auth" = []))
)]
pub async fn handler(
	InvoiceDepositPath { invoice_id }: InvoiceDepositPath,
	State(state): State<AppState>,
	Extension(identity): Extension<Identity>,
	Json(payload): Json<CreateDepositInvoiceRequest>,
) -> Result<Response<InvoiceResponse>, ApiError> {
	require_invoice_membership(&state, &identity, invoice_id).await?;

	let basis = parse_basis(&payload.basis)?;
	let value = Decimal::from_str(&payload.value)
		.map_err(|_| ApiError::Validation("deposit value must be a decimal number".to_owned()))?;

	let invoice = state
		.usecase
		.create_deposit_invoice(invoice_id, basis, value)
		.await?;

	Ok(Response::Created(invoice.into()))
}
