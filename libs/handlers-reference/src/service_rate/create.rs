use std::{collections::HashMap, str::FromStr};

use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{CreateServiceRateCommand, ServiceRateUnit};
use rust_decimal::Decimal;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::ServiceRatesPath, require_org_membership, response::ServiceRateResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateServiceRateRequest {
    pub label: String,
    pub unit: ServiceRateUnit,
    pub rate_cents: i32,
    pub vat_rate: Option<String>,
    pub custom_fields: Option<HashMap<String, String>>,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/service-rates",
    operation_id = "createServiceRate",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    request_body = CreateServiceRateRequest,
    responses(
        (status = 201, description = "Service rate created", body = inline(DataEnvelope<ServiceRateResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 409, description = "Service rate conflict"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: ServiceRatesPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateServiceRateRequest>,
) -> Result<Response<ServiceRateResponse>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;

    let vat_rate = match payload.vat_rate {
        Some(s) => Decimal::from_str(&s).map_err(|_| {
            ApiError::Validation("service rate vat_rate must be decimal".to_owned())
        })?,
        None => Decimal::from(20u32),
    };

    let service_rate = state
        .usecase
        .create_service_rate(CreateServiceRateCommand {
            organization_id: path.organization_id,
            label: payload.label,
            unit: payload.unit,
            rate_cents: payload.rate_cents,
            vat_rate,
            custom_fields: payload.custom_fields.unwrap_or_default(),
        })
        .await?;

    Ok(Response::Created(service_rate.into()))
}
