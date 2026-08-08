use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, Response, resolve_user_id};

use crate::{EmptyResponse, paths::DeliveryReplayPath, require_org_membership};

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/automation/deliveries/{delivery_id}/replay",
    operation_id = "replayAutomationDelivery",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("delivery_id" = uuid::Uuid, Path, description = "Delivery identifier"),
    ),
    responses(
        (status = 204, description = "Queued again, with its attempt count reset"),
        (status = 404, description = "No such delivery in this organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: DeliveryReplayPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<EmptyResponse>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;
    let actor = resolve_user_id(&state, &identity).await?;

    state
        .usecase
        .acting_as(actor)
        .replay_delivery(path.organization_id, path.delivery_id)
        .await?;

    Ok(Response::NoContent)
}
