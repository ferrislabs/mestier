use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::{
	paths::BillingSettingsPath,
	require_org_membership,
	response::BillingSettingsResponse,
};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/billing-settings",
    operation_id = "getBillingSettings",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    responses(
        (status = 200, description = "Billing settings (or defaults if none saved yet)", body = inline(DataEnvelope<BillingSettingsResponse>)),
        (status = 204, description = "No billing settings exist yet for this organization"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
	path: BillingSettingsPath,
	State(state): State<AppState>,
	Extension(identity): Extension<Identity>,
) -> Result<Response<BillingSettingsResponse>, ApiError> {
	require_org_membership(&state, &identity, path.organization_id).await?;

	let settings = state
		.usecase
		.get_billing_settings(path.organization_id)
		.await?;

	match settings {
		Some(s) => Ok(Response::OK(s.into())),
		None => Ok(Response::NoContent),
	}
}
