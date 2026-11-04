use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::CreateApiKeyCommand;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    api_key::require_key_manager, paths::ApiKeysPath, response::CreatedSupportApiKeyResponse,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateSupportApiKeyRequest {
    #[schema(example = "Contact form")]
    pub name: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/support/api-keys",
    operation_id = "createSupportApiKey",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    request_body = CreateSupportApiKeyRequest,
    responses(
        (status = 201, description = "The key was created; its key field is the only time the plaintext is ever returned", body = inline(DataEnvelope<CreatedSupportApiKeyResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 409, description = "Blank or too long name"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    ApiKeysPath { organization_id }: ApiKeysPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateSupportApiKeyRequest>,
) -> Result<Response<CreatedSupportApiKeyResponse>, ApiError> {
    let actor = require_key_manager(&state, &identity, organization_id).await?;

    let created = state
        .usecase
        .create_support_api_key(CreateApiKeyCommand {
            actor,
            organization_id,
            name: payload.name,
        })
        .await?;

    Ok(Response::Created(created.into()))
}
