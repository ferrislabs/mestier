use std::str::FromStr;

use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{ServiceRateId, ServiceRateUnit, UpdateServiceRateCommand};
use rust_decimal::Decimal;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::ServiceRatePath, require_org_membership, response::ServiceRateResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateServiceRateRequest {
    pub label: String,
    pub unit: ServiceRateUnit,
    pub rate_cents: i32,
    pub vat_rate: Option<String>,
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

    let vat_rate = match payload.vat_rate {
        Some(s) => Decimal::from_str(&s).map_err(|_| {
            ApiError::Validation("service rate vat_rate must be decimal".to_owned())
        })?,
        None => Decimal::from(20u32),
    };

    let service_rate = state
        .usecase
        .update_service_rate(UpdateServiceRateCommand {
            id: service_rate_id,
            label: payload.label,
            unit: payload.unit,
            rate_cents: payload.rate_cents,
            vat_rate,
        })
        .await?;

    Ok(Response::OK(service_rate.into()))
}
