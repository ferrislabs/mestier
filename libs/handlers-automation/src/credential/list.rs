use auth::Identity;
use axum::{Extension, extract::State};
use chrono::Utc;
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::{paths::CredentialsPath, require_view_automation, response::CredentialResponse};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/automation/credentials",
    operation_id = "listAutomationCredentials",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    responses(
        (status = 200, description = "Credentials for this organization with their real status and how many workflows use them, never their secret", body = inline(DataEnvelope<Vec<CredentialResponse>>)),
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
    require_view_automation(&state, &identity, path.organization_id).await?;

    let credentials = state.usecase.list_credentials(path.organization_id).await?;
    let usage = state.usecase.credential_usage(path.organization_id).await?;
    let now = Utc::now();
    let body: Vec<CredentialResponse> = credentials
        .into_iter()
        .map(|credential| {
            let used_by = usage.get(&credential.id).copied().unwrap_or(0);
            CredentialResponse::new(credential, used_by, now)
        })
        .collect();

    Ok(Response::OK(body))
}
