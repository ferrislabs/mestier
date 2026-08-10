use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, Response};

use crate::{credential::require_credential, paths::CredentialPath};

#[derive(Debug, serde::Serialize, PartialEq)]
pub struct EmptyResponse;

#[utoipa::path(
    delete,
    path = "/api/v1/organizations/{organization_id}/automation/credentials/{credential_id}",
    operation_id = "deleteAutomationCredential",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("credential_id" = uuid::Uuid, Path, description = "Credential identifier"),
    ),
    responses(
        (status = 204, description = "Credential deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Credential not found"),
        (status = 409, description = "Still referenced by a workflow"),
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
) -> Result<Response<EmptyResponse>, ApiError> {
    require_credential(&state, &identity, organization_id, credential_id).await?;

    state
        .usecase
        .delete_credential(organization_id, credential_id)
        .await?;

    Ok(Response::NoContent)
}
