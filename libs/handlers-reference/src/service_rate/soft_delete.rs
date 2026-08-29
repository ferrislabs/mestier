use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, Response, resolve_actor};
use mestier_core::ServiceRateId;

use crate::{EmptyResponse, paths::ServiceRatePath, require_org_membership};

#[utoipa::path(
    delete,
    path = "/api/v1/service-rates/{service_rate_id}",
    operation_id = "deleteServiceRate",
    tag = super::super::TAG,
    params(
        ("service_rate_id" = ServiceRateId, Path, description = "Service rate identifier"),
    ),
    responses(
        (status = 204, description = "Service rate soft-deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Service rate not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    ServiceRatePath { service_rate_id }: ServiceRatePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<EmptyResponse>, ApiError> {
    let current = state.usecase.get_service_rate(service_rate_id).await?;
    require_org_membership(&state, &identity, current.organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    state
        .usecase
        .acting_as(user_id)
        .soft_delete_service_rate(actor, service_rate_id)
        .await?;

    Ok(Response::NoContent)
}
