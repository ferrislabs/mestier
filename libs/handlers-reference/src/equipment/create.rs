use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::CreateEquipmentCommand;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::EquipmentCollectionPath, require_org_membership, response::EquipmentResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateEquipmentRequest {
    pub name: String,
    pub hourly_rate_cents: i32,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/equipment",
    operation_id = "createEquipment",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    request_body = CreateEquipmentRequest,
    responses(
        (status = 201, description = "Equipment created", body = inline(DataEnvelope<EquipmentResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 409, description = "Equipment conflict"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: EquipmentCollectionPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateEquipmentRequest>,
) -> Result<Response<EquipmentResponse>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;

    let equipment = state
        .usecase
        .create_equipment(CreateEquipmentCommand {
            organization_id: path.organization_id,
            name: payload.name,
            hourly_rate_cents: payload.hourly_rate_cents,
        })
        .await?;

    Ok(Response::Created(equipment.into()))
}
