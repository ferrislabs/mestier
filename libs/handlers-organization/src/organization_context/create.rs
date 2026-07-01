use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::CreateOrganizationContextCommand;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
	organization_context::response::OrganizationContextResponse,
	paths::OrganizationContextsPath, require_org_membership,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateOrganizationContextRequest {
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
	post,
	path = "/api/v1/organizations/{organization_id}/organization-contexts",
	operation_id = "createOrganizationContext",
	tag = super::super::TAG,
	params(
		("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
	),
	request_body = CreateOrganizationContextRequest,
	responses(
		(status = 201, description = "Organization context created", body = inline(DataEnvelope<OrganizationContextResponse>)),
		(status = 400, description = "Validation failed"),
		(status = 401, description = "Unauthorized"),
		(status = 403, description = "Forbidden"),
		(status = 409, description = "Organization context conflict"),
	),
	security(("bearer_auth" = []))
)]
pub async fn handler(
	path: OrganizationContextsPath,
	State(state): State<AppState>,
	Extension(identity): Extension<Identity>,
	Json(payload): Json<CreateOrganizationContextRequest>,
) -> Result<Response<OrganizationContextResponse>, ApiError> {
	require_org_membership(&state, &identity, path.organization_id).await?;

	let organization_context = state
		.usecase
		.create_organization_context(CreateOrganizationContextCommand {
			org_id: path.organization_id,
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

	Ok(Response::Created(organization_context.into()))
}
