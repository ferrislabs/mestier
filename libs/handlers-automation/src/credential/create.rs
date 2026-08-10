use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{CreateCredentialCommand, CredentialOrigin};
use serde::Deserialize;
use serde_json::Value;
use utoipa::ToSchema;

use crate::{
    paths::CredentialsPath,
    require_org_membership,
    response::{CredentialResponse, CredentialWithSecretResponse, secret_value},
};

#[derive(Debug, Clone, Copy, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum CredentialOriginRequest {
    Supplied,
    Generated,
}

impl From<CredentialOriginRequest> for CredentialOrigin {
    fn from(value: CredentialOriginRequest) -> Self {
        match value {
            CredentialOriginRequest::Supplied => Self::Supplied,
            CredentialOriginRequest::Generated => Self::Generated,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateCredentialRequest {
    pub kind: String,
    pub name: String,
    pub origin: CredentialOriginRequest,
    /// Required for a `Supplied` credential (what the caller is entering),
    /// ignored for a `Generated` one (Mestier fills it in itself).
    #[serde(default)]
    pub data: Option<Value>,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/automation/credentials",
    operation_id = "createAutomationCredential",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    request_body = CreateCredentialRequest,
    responses(
        (status = 201, description = "Credential created — the secret is visible in this response and never again", body = inline(DataEnvelope<CredentialWithSecretResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 409, description = "Unknown scheme, invalid data, or no automation secret key configured"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: CredentialsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateCredentialRequest>,
) -> Result<Response<CredentialWithSecretResponse>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;

    let origin: CredentialOrigin = payload.origin.into();
    let (credential, plaintext) = state
        .usecase
        .create_credential(CreateCredentialCommand {
            org_id: path.organization_id,
            kind: payload.kind,
            name: payload.name,
            origin,
            data: payload.data,
        })
        .await?;

    let secret = secret_value(origin, &plaintext);
    Ok(Response::Created(CredentialWithSecretResponse {
        credential: CredentialResponse::from(credential),
        secret,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn origin_supplied_deserializes_from_snake_case() {
        let parsed: CredentialOriginRequest =
            serde_json::from_value(json_value("supplied")).expect("`supplied` must deserialize");
        assert!(matches!(parsed, CredentialOriginRequest::Supplied));
    }

    #[test]
    fn origin_generated_deserializes_from_snake_case() {
        let parsed: CredentialOriginRequest =
            serde_json::from_value(json_value("generated")).expect("`generated` must deserialize");
        assert!(matches!(parsed, CredentialOriginRequest::Generated));
    }

    #[test]
    fn an_unknown_origin_is_rejected_not_ignored() {
        let result: Result<CredentialOriginRequest, _> =
            serde_json::from_value(json_value("not_an_origin"));
        assert!(result.is_err());
    }

    fn json_value(s: &str) -> serde_json::Value {
        serde_json::Value::String(s.to_string())
    }
}
