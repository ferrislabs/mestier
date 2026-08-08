use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::{paths::EndpointsPath, require_org_membership, response::WebhookEndpointResponse};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/automation/endpoints",
    operation_id = "listWebhookEndpoints",
    tag = super::super::TAG,
    params(("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier")),
    responses(
        (status = 200, description = "Endpoints of the organization", body = inline(DataEnvelope<Vec<WebhookEndpointResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: EndpointsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<WebhookEndpointResponse>>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;

    let endpoints = state
        .usecase
        .list_webhook_endpoints(path.organization_id)
        .await?;

    let mut responses = Vec::with_capacity(endpoints.len());
    for endpoint in endpoints {
        let (_, event_names) = state.usecase.get_webhook_endpoint(endpoint.id).await?;
        responses.push(WebhookEndpointResponse::new(endpoint, event_names));
    }

    Ok(Response::OK(responses))
}
