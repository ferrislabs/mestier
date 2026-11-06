use auth::Identity;
use axum::{Extension, Json, extract::State};
use chrono::{DateTime, Utc};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::UpdateCredentialCommand;
use serde::Deserialize;
use serde_json::Value;
use utoipa::ToSchema;

use crate::{
    credential::{credential_failure, require_credential},
    paths::CredentialPath,
    request::deserialize_present,
    response::CredentialResponse,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateCredentialRequest {
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    #[schema(value_type = Option<Object>)]
    pub data: Option<Value>,
    #[serde(default, deserialize_with = "deserialize_present")]
    #[schema(value_type = Option<DateTime<Utc>>)]
    pub expires_at: Option<Option<DateTime<Utc>>>,
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
        (status = 200, description = "Credential updated. A secret absent from data is kept, a secret present is replaced, an explicit null is refused; never carries a secret", body = inline(DataEnvelope<CredentialResponse>)),
        (status = 400, description = "Fields that do not match the kind's descriptor"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Credential not found"),
        (status = 409, description = "Name already used for this kind"),
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
            description: payload.description,
            data: payload.data,
            expires_at: payload.expires_at,
        })
        .await
        .map_err(credential_failure)?;

    let usage = state.usecase.credential_usage(organization_id).await?;
    let used_by = usage.get(&credential_id).copied().unwrap_or(0);
    Ok(Response::OK(CredentialResponse::new(
        updated,
        used_by,
        Utc::now(),
    )))
}
