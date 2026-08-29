use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, Response, resolve_actor};

use crate::{EmptyResponse, paths::RolePath};

#[utoipa::path(
    delete,
    path = "/api/v1/roles/{role_id}",
    operation_id = "deleteRole",
    tag = super::super::TAG,
    params(
        ("role_id" = mestier_core::RoleId, Path, description = "Role identifier"),
    ),
    responses(
        (status = 204, description = "Role deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Role not found"),
        (status = 409, description = "The role is seeded, or still assigned to at least one member"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    RolePath { role_id }: RolePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<EmptyResponse>, ApiError> {
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    state
        .usecase
        .acting_as(user_id)
        .delete_role(role_id, actor)
        .await?;

    Ok(Response::NoContent)
}
