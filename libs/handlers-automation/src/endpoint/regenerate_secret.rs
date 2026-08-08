use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_user_id};

use crate::{paths::EndpointSecretPath, require_endpoint_membership, response::SecretResponse};

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/automation/endpoints/{endpoint_id}/secret",
    operation_id = "regenerateWebhookSecret",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("endpoint_id" = uuid::Uuid, Path, description = "Endpoint identifier"),
    ),
    responses(
        (status = 200, description = "A new secret, shown once. The previous one stops working immediately.", body = inline(DataEnvelope<SecretResponse>)),
        (status = 404, description = "Endpoint not found"),
        (status = 409, description = "The instance has no automation secret key configured"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: EndpointSecretPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<SecretResponse>, ApiError> {
    require_endpoint_membership(&state, &identity, path.organization_id, path.endpoint_id).await?;
    let actor = resolve_user_id(&state, &identity).await?;

    let secret = state
        .usecase
        .acting_as(actor)
        .regenerate_webhook_secret(path.endpoint_id)
        .await?;

    Ok(Response::OK(SecretResponse { secret }))
}
