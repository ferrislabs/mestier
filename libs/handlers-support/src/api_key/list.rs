use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::ListApiKeysCommand;

use crate::{api_key::require_key_manager, paths::ApiKeysPath, response::SupportApiKeyResponse};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/support/api-keys",
    operation_id = "listSupportApiKeys",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    responses(
        (status = 200, description = "The organization's keys, newest first, revoked ones included; never the secret", body = inline(DataEnvelope<Vec<SupportApiKeyResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    ApiKeysPath { organization_id }: ApiKeysPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<SupportApiKeyResponse>>, ApiError> {
    let actor = require_key_manager(&state, &identity, organization_id).await?;

    let keys = state
        .usecase
        .list_support_api_keys(ListApiKeysCommand {
            actor,
            organization_id,
        })
        .await?;

    Ok(Response::OK(
        keys.into_iter().map(SupportApiKeyResponse::from).collect(),
    ))
}
