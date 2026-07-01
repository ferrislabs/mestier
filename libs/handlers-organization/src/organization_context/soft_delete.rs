use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, Response};
use mestier_core::OrganizationContextId;

use crate::{EmptyResponse, paths::OrganizationContextPath, require_org_membership};

#[utoipa::path(
	delete,
	path = "/api/v1/organization-contexts/{context_id}",
	operation_id = "deleteOrganizationContext",
	tag = super::super::TAG,
	params(
		("context_id" = OrganizationContextId, Path, description = "Organization context identifier"),
	),
	responses(
		(status = 204, description = "Organization context soft-deleted"),
		(status = 401, description = "Unauthorized"),
		(status = 403, description = "Forbidden"),
		(status = 404, description = "Organization context not found"),
	),
	security(("bearer_auth" = []))
)]
pub async fn handler(
	OrganizationContextPath { context_id }: OrganizationContextPath,
	State(state): State<AppState>,
	Extension(identity): Extension<Identity>,
) -> Result<Response<EmptyResponse>, ApiError> {
	let current = state
		.usecase
		.get_organization_context(context_id)
		.await?;
	require_org_membership(&state, &identity, current.org_id).await?;
	state
		.usecase
		.soft_delete_organization_context(context_id)
		.await?;

	Ok(Response::NoContent)
}
