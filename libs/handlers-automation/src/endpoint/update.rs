use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_user_id};
use mestier_core::UpdateWebhookEndpointCommand;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::EndpointPath, require_endpoint_membership, response::WebhookEndpointResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateEndpointRequest {
    pub url: String,
    pub description: Option<String>,
    pub enabled: bool,
    pub event_names: Vec<String>,
}

#[utoipa::path(
    patch,
    path = "/api/v1/organizations/{organization_id}/automation/endpoints/{endpoint_id}",
    operation_id = "updateWebhookEndpoint",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("endpoint_id" = uuid::Uuid, Path, description = "Endpoint identifier"),
    ),
    request_body = UpdateEndpointRequest,
    responses(
        (status = 200, description = "Endpoint updated", body = inline(DataEnvelope<WebhookEndpointResponse>)),
        (status = 400, description = "Unknown event name or invalid URL"),
        (status = 404, description = "Endpoint not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: EndpointPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<UpdateEndpointRequest>,
) -> Result<Response<WebhookEndpointResponse>, ApiError> {
    require_endpoint_membership(&state, &identity, path.organization_id, path.endpoint_id).await?;
    let actor = resolve_user_id(&state, &identity).await?;

    let event_names = payload.event_names.clone();
    let updated = state
        .usecase
        .acting_as(actor)
        .update_webhook_endpoint(UpdateWebhookEndpointCommand {
            id: path.endpoint_id,
            url: payload.url,
            description: payload.description,
            enabled: payload.enabled,
            event_names: payload.event_names,
        })
        .await?;

    Ok(Response::OK(WebhookEndpointResponse::new(
        updated,
        event_names,
    )))
}
