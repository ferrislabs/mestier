use auth::Identity;
use axum::{
    Extension, Json, Router,
    extract::{Query, State},
    middleware::from_fn_with_state,
    response::{IntoResponse, Redirect, Response as AxumResponse},
    routing::get,
};
use chrono::Utc;
use handlers::{ApiError, AppState, DataEnvelope, Response, rate_limit::rate_limit_middleware};
use http::{HeaderValue, header};
use mestier_core::{
    CredentialError, OauthCallback, StartOauthCommand, is_valid_redirect_uri, is_valid_return_to,
    with_oauth_outcome,
};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    credential::{credential_failure, require_credential},
    paths::{CredentialOauthDisconnectPath, CredentialOauthStartPath},
    response::CredentialResponse,
};

const LOCAL_ORIGIN: &str = "http://localhost:3000";

#[derive(Debug, Deserialize, ToSchema)]
pub struct StartOauthRequest {
    pub redirect_uri: String,
    pub return_to: String,
}

#[derive(Debug, PartialEq, Serialize, ToSchema)]
pub struct StartOauthResponse {
    pub authorization_url: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/automation/credentials/{credential_id}/oauth/start",
    operation_id = "startAutomationCredentialOauth",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("credential_id" = uuid::Uuid, Path, description = "Credential identifier"),
    ),
    request_body = StartOauthRequest,
    responses(
        (status = 200, description = "The address of the provider's authorization page. It carries a single-use state, bound to this organization, this credential and the caller, valid ten minutes", body = inline(DataEnvelope<StartOauthResponse>)),
        (status = 400, description = "The credential kind is not connected to an account, or an address is not acceptable"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Credential not found"),
        (status = 409, description = "The provider did not answer, or too many authorizations are pending"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn start(
    CredentialOauthStartPath {
        organization_id,
        credential_id,
    }: CredentialOauthStartPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<StartOauthRequest>,
) -> Result<Response<StartOauthResponse>, ApiError> {
    require_credential(&state, &identity, organization_id, credential_id).await?;

    let mut origins = vec![LOCAL_ORIGIN.to_owned()];
    origins.extend(state.args.server.allowed_origins.iter().cloned());
    if !is_valid_redirect_uri(&payload.redirect_uri) {
        return Err(credential_failure(
            CredentialError::InvalidRedirectUri.into(),
        ));
    }
    if !is_valid_return_to(&payload.return_to, &origins) {
        return Err(credential_failure(CredentialError::InvalidReturnTo.into()));
    }
    let member = state
        .usecase
        .find_user_by_sub(identity.id())
        .await?
        .ok_or(ApiError::Forbidden)?;

    let authorization_url = state
        .usecase
        .start_oauth_authorization(StartOauthCommand {
            org_id: organization_id,
            credential_id,
            member_id: member.id.0,
            redirect_uri: payload.redirect_uri,
            return_to: payload.return_to,
        })
        .await
        .map_err(credential_failure)?;

    Ok(Response::OK(StartOauthResponse { authorization_url }))
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/automation/credentials/{credential_id}/oauth/disconnect",
    operation_id = "disconnectAutomationCredentialOauth",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("credential_id" = uuid::Uuid, Path, description = "Credential identifier"),
    ),
    responses(
        (status = 200, description = "The tokens are erased; the credential is back to not connected", body = inline(DataEnvelope<CredentialResponse>)),
        (status = 400, description = "The credential kind is not connected to an account"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Credential not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn disconnect(
    CredentialOauthDisconnectPath {
        organization_id,
        credential_id,
    }: CredentialOauthDisconnectPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<CredentialResponse>, ApiError> {
    require_credential(&state, &identity, organization_id, credential_id).await?;

    let disconnected = state
        .usecase
        .disconnect_credential_account(organization_id, credential_id)
        .await
        .map_err(credential_failure)?;
    let usage = state.usecase.credential_usage(organization_id).await?;
    let used_by = usage.get(&credential_id).copied().unwrap_or(0);

    Ok(Response::OK(CredentialResponse::new(
        disconnected,
        used_by,
        Utc::now(),
    )))
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CallbackQuery {
    #[serde(default)]
    pub state: Option<String>,
    #[serde(default)]
    pub code: Option<String>,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub oauth_token: Option<String>,
    #[serde(default)]
    pub oauth_verifier: Option<String>,
}

pub fn callback_router(state: &AppState) -> Router<AppState> {
    Router::new()
        .route("/oauth/callback", get(callback))
        .layer(from_fn_with_state(state.clone(), rate_limit_middleware))
}

fn private(mut response: AxumResponse) -> AxumResponse {
    let headers = response.headers_mut();
    headers.insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    headers.insert(
        header::REFERRER_POLICY,
        HeaderValue::from_static("no-referrer"),
    );
    response
}

#[utoipa::path(
    get,
    path = "/oauth/callback",
    operation_id = "completeAutomationCredentialOauth",
    tag = super::super::TAG,
    description = "Where the provider sends the browser back. Public: it is not under /api/v1 and takes no bearer token; the single-use state it carries decides which credential is connected. It exchanges the code for tokens, stores them sealed, then redirects to the address given at start with oauth=ok or oauth=error. A state that is unknown, already used or older than ten minutes answers 400.",
    params(
        ("state" = Option<String>, Query, description = "The state issued at start"),
        ("code" = Option<String>, Query, description = "The authorization code (OAuth2)"),
        ("error" = Option<String>, Query, description = "Set by a provider that refused the authorization"),
        ("oauth_token" = Option<String>, Query, description = "The request token (OAuth1)"),
        ("oauth_verifier" = Option<String>, Query, description = "The verifier (OAuth1)"),
    ),
    responses(
        (status = 303, description = "Redirected to the address given at start, with oauth=ok or oauth=error"),
        (status = 400, description = "The state is missing, unknown, already used or expired"),
        (status = 429, description = "Too many requests"),
    ),
)]
pub async fn callback(
    State(state): State<AppState>,
    Query(query): Query<CallbackQuery>,
) -> AxumResponse {
    let Some(oauth_state) = query.state.filter(|value| !value.is_empty()) else {
        return private(
            ApiError::BadRequest("the authorization is not valid".to_owned()).into_response(),
        );
    };
    let callback = OauthCallback {
        state: oauth_state,
        code: query.code,
        provider_refused: query.error.is_some(),
        oauth_token: query.oauth_token,
        oauth_verifier: query.oauth_verifier,
    };

    match state.usecase.complete_oauth_authorization(callback).await {
        Ok(outcome) => private(
            Redirect::to(&with_oauth_outcome(&outcome.return_to, outcome.connected))
                .into_response(),
        ),
        Err(common::CoreError::NotFound) => private(
            ApiError::BadRequest("the authorization is not valid".to_owned()).into_response(),
        ),
        Err(error) => private(ApiError::from(error).into_response()),
    }
}
