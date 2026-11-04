use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, Response};
use mestier_core::RevokeApiKeyCommand;

use crate::{api_key::require_key_manager, paths::ApiKeyPath, response::EmptyResponse};

#[utoipa::path(
    delete,
    path = "/api/v1/organizations/{organization_id}/support/api-keys/{key_id}",
    operation_id = "revokeSupportApiKey",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("key_id" = mestier_core::SupportApiKeyId, Path, description = "API key identifier"),
    ),
    responses(
        (status = 204, description = "The key is revoked, whether or not it already was"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Key not found in the organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    ApiKeyPath {
        organization_id,
        key_id,
    }: ApiKeyPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<EmptyResponse>, ApiError> {
    let actor = require_key_manager(&state, &identity, organization_id).await?;

    state
        .usecase
        .revoke_support_api_key(RevokeApiKeyCommand {
            actor,
            organization_id,
            key_id,
        })
        .await?;

    Ok(Response::NoContent)
}
