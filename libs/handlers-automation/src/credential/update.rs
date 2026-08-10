use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::UpdateCredentialCommand;
use serde::Deserialize;
use serde_json::Value;
use utoipa::ToSchema;

use crate::{credential::require_credential, paths::CredentialPath, response::CredentialResponse};

/// Both fields optional and applied only when present. `data: None` renames
/// without touching the sealed bytes; `Some` replaces them after
/// re-validating against the credential's scheme — mirrors
/// `mestier_core::UpdateCredentialCommand`'s own doc comment.
#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateCredentialRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub data: Option<Value>,
}

#[utoipa::path(
    patch,
    path = "/api/v1/organizations/{organization_id}/automation/credentials/{credential_id}",
    operation_id = "updateAutomationCredential",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("credential_id" = uuid::Uuid, Path, description = "Credential identifier"),
    ),
    request_body = UpdateCredentialRequest,
    responses(
        (status = 200, description = "Credential updated — never carries the secret", body = inline(DataEnvelope<CredentialResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Credential not found"),
        (status = 409, description = "Invalid data for this credential's scheme"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    CredentialPath {
        organization_id,
        credential_id,
    }: CredentialPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<UpdateCredentialRequest>,
) -> Result<Response<CredentialResponse>, ApiError> {
    require_credential(&state, &identity, organization_id, credential_id).await?;

    let updated = state
        .usecase
        .update_credential(UpdateCredentialCommand {
            org_id: organization_id,
            id: credential_id,
            name: payload.name,
            data: payload.data,
        })
        .await?;

    Ok(Response::OK(CredentialResponse::from(updated)))
}
