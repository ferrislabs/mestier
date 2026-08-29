use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::{Permissions, UpdateRoleCommand};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::RolePath, response::RoleResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateRoleRequest {
    pub name: String,
    /// Bit names — see `CreateRoleRequest::permissions`. Full replacement,
    /// not a patch: the editor always posts back the complete set it read.
    pub permissions: Vec<String>,
}

#[utoipa::path(
    patch,
    path = "/api/v1/roles/{role_id}",
    operation_id = "updateRole",
    tag = super::super::TAG,
    params(
        ("role_id" = mestier_core::RoleId, Path, description = "Role identifier"),
    ),
    request_body = UpdateRoleRequest,
    responses(
        (status = 200, description = "Role updated", body = inline(DataEnvelope<RoleResponse>)),
        (status = 400, description = "Unknown permission name"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Role not found"),
        (status = 409, description = "The role is seeded and cannot be renamed, or the new name is already taken"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    RolePath { role_id }: RolePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<UpdateRoleRequest>,
) -> Result<Response<RoleResponse>, ApiError> {
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    let permissions =
        Permissions::from_names(&payload.permissions).map_err(ApiError::Validation)?;

    let role = state
        .usecase
        .acting_as(user_id)
        .update_role(UpdateRoleCommand {
            actor,
            role_id,
            name: payload.name,
            permissions,
        })
        .await?;

    Ok(Response::OK(role.into()))
}
