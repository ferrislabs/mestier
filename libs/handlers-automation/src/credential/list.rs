use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::{paths::CredentialsPath, require_org_membership, response::CredentialResponse};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/automation/credentials",
    operation_id = "listAutomationCredentials",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    responses(
        (status = 200, description = "Credentials for this organization — never their secret", body = inline(DataEnvelope<Vec<CredentialResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: CredentialsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<CredentialResponse>>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;

    let credentials = state.usecase.list_credentials(path.organization_id).await?;
    let body: Vec<CredentialResponse> = credentials
        .into_iter()
        .map(CredentialResponse::from)
        .collect();

    Ok(Response::OK(body))
}
