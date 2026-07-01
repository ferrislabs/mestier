use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{OrganizationContextId, UpdateOrganizationContextCommand};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
	organization_context::response::OrganizationContextResponse,
	paths::OrganizationContextPath, require_org_membership,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateOrganizationContextRequest {
	pub label: String,
	pub address_line: Option<String>,
	pub postal_code: Option<String>,
	pub city: Option<String>,
	pub country: Option<String>,
	pub siret: Option<String>,
	pub rcs: Option<String>,
	pub ape: Option<String>,
	pub vat_intracom: Option<String>,
	pub iban: Option<String>,
	pub bic: Option<String>,
}

#[utoipa::path(
	patch,
	path = "/api/v1/organization-contexts/{context_id}",
	operation_id = "updateOrganizationContext",
	tag = super::super::TAG,
	params(
		("context_id" = OrganizationContextId, Path, description = "Organization context identifier"),
	),
	request_body = UpdateOrganizationContextRequest,
	responses(
		(status = 200, description = "Organization context updated", body = inline(DataEnvelope<OrganizationContextResponse>)),
		(status = 400, description = "Validation failed"),
		(status = 401, description = "Unauthorized"),
		(status = 403, description = "Forbidden"),
		(status = 404, description = "Organization context not found"),
		(status = 409, description = "Organization context conflict"),
	),
	security(("bearer_auth" = []))
)]
pub async fn handler(
	OrganizationContextPath { context_id }: OrganizationContextPath,
	State(state): State<AppState>,
	Extension(identity): Extension<Identity>,
	Json(payload): Json<UpdateOrganizationContextRequest>,
) -> Result<Response<OrganizationContextResponse>, ApiError> {
	let current = state
		.usecase
		.get_organization_context(context_id)
		.await?;
	require_org_membership(&state, &identity, current.org_id).await?;

	let organization_context = state
		.usecase
		.update_organization_context(UpdateOrganizationContextCommand {
			id: context_id,
			label: payload.label,
			address_line: payload.address_line,
			postal_code: payload.postal_code,
			city: payload.city,
			country: payload.country,
			siret: payload.siret,
			rcs: payload.rcs,
			ape: payload.ape,
			vat_intracom: payload.vat_intracom,
			iban: payload.iban,
			bic: payload.bic,
		})
		.await?;

	Ok(Response::OK(organization_context.into()))
}
