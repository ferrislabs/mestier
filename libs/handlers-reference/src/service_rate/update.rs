use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::{ServiceRateId, ServiceRateUnit, UpdateServiceRateCommand};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::ServiceRatePath, require_org_membership, response::ServiceRateResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateServiceRateRequest {
    pub label: String,
    pub unit: ServiceRateUnit,
    pub rate_cents: i32,
    #[serde(default)]
    pub default_vat_rate_bp: Option<i32>,
}

#[utoipa::path(
    patch,
    path = "/api/v1/service-rates/{service_rate_id}",
    operation_id = "updateServiceRate",
    tag = super::super::TAG,
    params(
        ("service_rate_id" = ServiceRateId, Path, description = "Service rate identifier"),
    ),
    request_body = UpdateServiceRateRequest,
    responses(
        (status = 200, description = "Service rate updated", body = inline(DataEnvelope<ServiceRateResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Service rate not found"),
        (status = 409, description = "Service rate conflict"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    ServiceRatePath { service_rate_id }: ServiceRatePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<UpdateServiceRateRequest>,
) -> Result<Response<ServiceRateResponse>, ApiError> {
    let current = state.usecase.get_service_rate(service_rate_id).await?;
    require_org_membership(&state, &identity, current.organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let service_rate = state
        .usecase
        .acting_as(user_id)
        .update_service_rate(UpdateServiceRateCommand {
            actor,
            id: service_rate_id,
            label: payload.label,
            unit: payload.unit,
            rate_cents: payload.rate_cents,
            default_vat_rate_bp: payload.default_vat_rate_bp,
        })
        .await?;

    Ok(Response::OK(service_rate.into()))
}
