use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_user_id};
use mestier_core::{EquipmentId, UpdateEquipmentCommand};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::EquipmentPath, require_org_membership, response::EquipmentResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateEquipmentRequest {
    pub name: String,
    pub hourly_rate_cents: i32,
}

#[utoipa::path(
    patch,
    path = "/api/v1/equipment/{equipment_id}",
    operation_id = "updateEquipment",
    tag = super::super::TAG,
    params(
        ("equipment_id" = EquipmentId, Path, description = "Equipment identifier"),
    ),
    request_body = UpdateEquipmentRequest,
    responses(
        (status = 200, description = "Equipment updated", body = inline(DataEnvelope<EquipmentResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Equipment not found"),
        (status = 409, description = "Equipment conflict"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    EquipmentPath { equipment_id }: EquipmentPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<UpdateEquipmentRequest>,
) -> Result<Response<EquipmentResponse>, ApiError> {
    let current = state.usecase.get_equipment(equipment_id).await?;
    require_org_membership(&state, &identity, current.organization_id).await?;
    let actor = resolve_user_id(&state, &identity).await?;

    let equipment = state
        .usecase
        .acting_as(actor)
        .update_equipment(UpdateEquipmentCommand {
            id: equipment_id,
            name: payload.name,
            hourly_rate_cents: payload.hourly_rate_cents,
        })
        .await?;

    Ok(Response::OK(equipment.into()))
}
