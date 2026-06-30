use std::str::FromStr;

use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::UpsertBillingSettingsCommand;
use rust_decimal::Decimal;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
	paths::BillingSettingsPath,
	require_org_membership,
	response::BillingSettingsResponse,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpsertBillingSettingsRequest {
	pub payment_terms_days: i32,
	pub late_penalty_rate: Option<String>,
	pub recovery_indemnity_cents: i32,
	pub default_deposit_basis: Option<String>,
	pub default_deposit_value: Option<String>,
	pub default_vat_rate: Option<String>,
	pub iban: Option<String>,
	pub bic: Option<String>,
	pub siret: Option<String>,
	pub rcs: Option<String>,
	pub ape: Option<String>,
	pub vat_intracom: Option<String>,
	pub footer: Option<String>,
}

#[utoipa::path(
    put,
    path = "/api/v1/organizations/{organization_id}/billing-settings",
    operation_id = "upsertBillingSettings",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    request_body = UpsertBillingSettingsRequest,
    responses(
        (status = 200, description = "Billing settings saved", body = inline(DataEnvelope<BillingSettingsResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
	path: BillingSettingsPath,
	State(state): State<AppState>,
	Extension(identity): Extension<Identity>,
	Json(payload): Json<UpsertBillingSettingsRequest>,
) -> Result<Response<BillingSettingsResponse>, ApiError> {
	require_org_membership(&state, &identity, path.organization_id).await?;

	let parse_decimal = |s: Option<String>, field: &str| -> Result<Decimal, ApiError> {
		match s {
			Some(v) => Decimal::from_str(&v)
				.map_err(|_| ApiError::Validation(format!("{field} must be a decimal"))),
			None => Ok(Decimal::ZERO),
		}
	};

	let parse_decimal_opt =
		|s: Option<String>, field: &str| -> Result<Option<Decimal>, ApiError> {
			match s {
				Some(v) => Decimal::from_str(&v)
					.map(Some)
					.map_err(|_| ApiError::Validation(format!("{field} must be a decimal"))),
				None => Ok(None),
			}
		};

	let late_penalty_rate = parse_decimal(payload.late_penalty_rate, "late_penalty_rate")?;
	let default_vat_rate = match payload.default_vat_rate {
		Some(s) => Decimal::from_str(&s)
			.map_err(|_| ApiError::Validation("default_vat_rate must be a decimal".to_owned()))?,
		None => Decimal::from(20u32),
	};
	let default_deposit_value =
		parse_decimal_opt(payload.default_deposit_value, "default_deposit_value")?;

	let settings = state
		.usecase
		.upsert_billing_settings(UpsertBillingSettingsCommand {
			org_id: path.organization_id,
			payment_terms_days: payload.payment_terms_days,
			late_penalty_rate,
			recovery_indemnity_cents: payload.recovery_indemnity_cents,
			default_deposit_basis: payload.default_deposit_basis,
			default_deposit_value,
			default_vat_rate,
			iban: payload.iban,
			bic: payload.bic,
			siret: payload.siret,
			rcs: payload.rcs,
			ape: payload.ape,
			vat_intracom: payload.vat_intracom,
			footer: payload.footer,
		})
		.await?;

	Ok(Response::OK(settings.into()))
}
