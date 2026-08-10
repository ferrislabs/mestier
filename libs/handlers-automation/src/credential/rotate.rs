use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::{
    credential::require_credential,
    paths::CredentialRotatePath,
    response::{CredentialResponse, CredentialWithSecretResponse, secret_value},
};

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/automation/credentials/{credential_id}/rotate",
    operation_id = "rotateAutomationCredential",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("credential_id" = uuid::Uuid, Path, description = "Credential identifier"),
    ),
    responses(
        (status = 200, description = "Credential rotated — the fresh secret is visible in this response and never again", body = inline(DataEnvelope<CredentialWithSecretResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Credential not found"),
        (status = 409, description = "A supplied credential is replaced, never rotated"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    CredentialRotatePath {
        organization_id,
        credential_id,
    }: CredentialRotatePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<CredentialWithSecretResponse>, ApiError> {
    let existing = require_credential(&state, &identity, organization_id, credential_id).await?;

    let (rotated, plaintext) = state
        .usecase
        .rotate_credential(organization_id, credential_id)
        .await?;

    let secret = secret_value(existing.origin, &plaintext);
    Ok(Response::OK(CredentialWithSecretResponse {
        credential: CredentialResponse::from(rotated),
        secret,
    }))
}
