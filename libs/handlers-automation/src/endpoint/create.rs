use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_user_id};
use mestier_core::CreateWebhookEndpointCommand;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    paths::EndpointsPath, require_org_membership, response::CreatedWebhookEndpointResponse,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateEndpointRequest {
    pub url: String,
    pub description: Option<String>,
    /// Must exist in the event catalogue. A free-text name would create an
    /// endpoint that looks healthy and never fires.
    pub event_names: Vec<String>,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/automation/endpoints",
    operation_id = "createWebhookEndpoint",
    tag = super::super::TAG,
    params(("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier")),
    request_body = CreateEndpointRequest,
    responses(
        (status = 201, description = "Endpoint created; the secret is returned once and never again", body = inline(DataEnvelope<CreatedWebhookEndpointResponse>)),
        (status = 400, description = "Unknown event name or invalid URL"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 409, description = "The instance has no automation secret key configured"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: EndpointsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateEndpointRequest>,
) -> Result<Response<CreatedWebhookEndpointResponse>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;
    let actor = resolve_user_id(&state, &identity).await?;

    let event_names = payload.event_names.clone();
    let created = state
        .usecase
        .acting_as(actor)
        .create_webhook_endpoint(CreateWebhookEndpointCommand {
            org_id: path.organization_id,
            url: payload.url,
            description: payload.description,
            event_names: payload.event_names,
        })
        .await?;

    Ok(Response::Created(CreatedWebhookEndpointResponse::new(
        created,
        event_names,
    )))
}
