use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, Response, resolve_user_id};

use crate::{EmptyResponse, paths::EndpointPath, require_endpoint_membership};

#[utoipa::path(
    delete,
    path = "/api/v1/organizations/{organization_id}/automation/endpoints/{endpoint_id}",
    operation_id = "deleteWebhookEndpoint",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("endpoint_id" = uuid::Uuid, Path, description = "Endpoint identifier"),
    ),
    responses(
        (status = 204, description = "Endpoint deleted with its subscription"),
        (status = 404, description = "Endpoint not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: EndpointPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<EmptyResponse>, ApiError> {
    require_endpoint_membership(&state, &identity, path.organization_id, path.endpoint_id).await?;
    let actor = resolve_user_id(&state, &identity).await?;

    state
        .usecase
        .acting_as(actor)
        .delete_webhook_endpoint(path.endpoint_id)
        .await?;

    Ok(Response::NoContent)
}
