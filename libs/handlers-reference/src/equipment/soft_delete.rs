use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, Response, resolve_user_id};
use mestier_core::EquipmentId;

use crate::{EmptyResponse, paths::EquipmentPath, require_org_membership};

#[utoipa::path(
    delete,
    path = "/api/v1/equipment/{equipment_id}",
    operation_id = "deleteEquipment",
    tag = super::super::TAG,
    params(
        ("equipment_id" = EquipmentId, Path, description = "Equipment identifier"),
    ),
    responses(
        (status = 204, description = "Equipment soft-deleted"),
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
) -> Result<Response<EmptyResponse>, ApiError> {
    let current = state.usecase.get_equipment(equipment_id).await?;
    require_org_membership(&state, &identity, current.organization_id).await?;
    let actor = resolve_user_id(&state, &identity).await?;
    state
        .usecase
        .acting_as(actor)
        .soft_delete_equipment(equipment_id)
        .await?;

    Ok(Response::NoContent)
}
