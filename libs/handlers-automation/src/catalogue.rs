//! `GET /connectors` and `GET /events`: the two static catalogues that make
//! the workflow editor data-driven. Both are pure — served straight from
//! `mestier_core::connector_catalogue`/`event_catalogue`, no database.

use auth::Identity;
use axum::{Extension, Router, extract::State};
use axum_extra::routing::RouterExt;
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{connector_catalogue, credential_kinds, event_catalogue};

use crate::{
    paths::{ConnectorsPath, CredentialKindsPath, EventsPath},
    require_view_automation,
    response::{
        AuthSchemeResponse, ConnectorDescriptorResponse, ConnectorsResponse,
        CredentialKindResponse, EventDescriptorResponse,
    },
};

pub fn router(_state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_get(connectors)
        .typed_get(events)
        .typed_get(credential_kinds_handler)
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/automation/connectors",
    operation_id = "listConnectors",
    tag = super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    responses(
        (status = 200, description = "The connector catalogue and the auth schemes it references", body = inline(DataEnvelope<ConnectorsResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn connectors(
    ConnectorsPath { organization_id }: ConnectorsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<ConnectorsResponse>, ApiError> {
    require_view_automation(&state, &identity, organization_id).await?;

    let catalogue = connector_catalogue();
    let body = ConnectorsResponse {
        connectors: catalogue
            .descriptors()
            .map(ConnectorDescriptorResponse::from)
            .collect(),
        auth_schemes: credential_kinds()
            .iter()
            .map(AuthSchemeResponse::from)
            .collect(),
    };

    Ok(Response::OK(body))
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/automation/events",
    operation_id = "listAutomationEvents",
    tag = super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    responses(
        (status = 200, description = "The event catalogue: every event a workflow can trigger from", body = inline(DataEnvelope<Vec<EventDescriptorResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn events(
    EventsPath { organization_id }: EventsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<EventDescriptorResponse>>, ApiError> {
    require_view_automation(&state, &identity, organization_id).await?;

    let catalogue = event_catalogue();
    let body: Vec<EventDescriptorResponse> = catalogue
        .descriptors()
        .map(EventDescriptorResponse::from)
        .collect();

    Ok(Response::OK(body))
}

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/automation/credential-kinds",
    operation_id = "listCredentialKinds",
    tag = super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    responses(
        (status = 200, description = "Every credential kind with its French label, category, usages, fields (default, placeholder, hint, conditional display) and whether an executor consumes it", body = inline(DataEnvelope<Vec<CredentialKindResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn credential_kinds_handler(
    CredentialKindsPath { organization_id }: CredentialKindsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<CredentialKindResponse>>, ApiError> {
    require_view_automation(&state, &identity, organization_id).await?;

    let body: Vec<CredentialKindResponse> = credential_kinds()
        .iter()
        .map(CredentialKindResponse::from)
        .collect();

    Ok(Response::OK(body))
}
