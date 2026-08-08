use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_user_id};
use mestier_core::{CreateServiceRateCommand, ServiceRateUnit};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::ServiceRatesPath, require_org_membership, response::ServiceRateResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateServiceRateRequest {
    pub label: String,
    pub unit: ServiceRateUnit,
    pub rate_cents: i32,
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
    let actor = resolve_user_id(&state, &identity).await?;

    let service_rate = state
        .usecase
        .acting_as(actor)
        .create_service_rate(CreateServiceRateCommand {
            organization_id: path.organization_id,
            label: payload.label,
            unit: payload.unit,
            rate_cents: payload.rate_cents,
        })
        .await?;

    Ok(Response::Created(service_rate.into()))
}
