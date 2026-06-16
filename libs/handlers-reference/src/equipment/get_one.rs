use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::EquipmentId;

use crate::{paths::EquipmentPath, require_org_membership, response::EquipmentResponse};

#[utoipa::path(
    get,
    path = "/api/v1/equipment/{equipment_id}",
    operation_id = "getEquipment",
    tag = super::super::TAG,
    params(
        ("equipment_id" = EquipmentId, Path, description = "Equipment identifier"),
    ),
    responses(
        (status = 200, description = "Equipment details", body = inline(DataEnvelope<EquipmentResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Equipment not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    EquipmentPath { equipment_id }: EquipmentPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<EquipmentResponse>, ApiError> {
    let equipment = state.usecase.get_equipment(equipment_id).await?;
    require_org_membership(&state, &identity, equipment.organization_id).await?;

    Ok(Response::OK(equipment.into()))
}
