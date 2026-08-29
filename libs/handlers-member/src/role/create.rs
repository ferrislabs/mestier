use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::{CreateRoleCommand, Permissions};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::RolesPath, response::RoleResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateRoleRequest {
    pub name: String,
    /// Bit names, e.g. `["VIEW_PLANNING", "MANAGE_PLANNING"]` — see
    /// `mestier_core::domain::role::Permissions::NAMED`.
    pub permissions: Vec<String>,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/roles",
    operation_id = "createRole",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
    ),
    request_body = CreateRoleRequest,
    responses(
        (status = 201, description = "Role created", body = inline(DataEnvelope<RoleResponse>)),
        (status = 400, description = "Unknown permission name"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 409, description = "A role with this name already exists in the organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    RolesPath { organization_id }: RolesPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateRoleRequest>,
) -> Result<Response<RoleResponse>, ApiError> {
    let (user_id, actor) = resolve_actor(&state, &identity).await?;
    let permissions =
        Permissions::from_names(&payload.permissions).map_err(ApiError::Validation)?;

    let role = state
        .usecase
        .acting_as(user_id)
        .create_role(CreateRoleCommand {
            actor,
            organization_id,
            name: payload.name,
            permissions,
        })
        .await?;

    Ok(Response::Created(role.into()))
}
