use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::ServiceRateId;

use crate::{paths::ServiceRatePath, require_org_membership, response::ServiceRateResponse};

#[utoipa::path(
    get,
    path = "/api/v1/service-rates/{service_rate_id}",
    operation_id = "getServiceRate",
    tag = super::super::TAG,
    params(
        ("service_rate_id" = ServiceRateId, Path, description = "Service rate identifier"),
    ),
    responses(
        (status = 200, description = "Service rate details", body = inline(DataEnvelope<ServiceRateResponse>)),
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
) -> Result<Response<ServiceRateResponse>, ApiError> {
    let service_rate = state.usecase.get_service_rate(service_rate_id).await?;
    require_org_membership(&state, &identity, service_rate.organization_id).await?;

    Ok(Response::OK(service_rate.into()))
}
