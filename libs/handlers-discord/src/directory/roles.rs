use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::{paths::OrgChatRolesPath, require_permission, response::ChatRoleResponse};

#[utoipa::path(
    get,
    path = "/api/v1/chat/organizations/{organization_id}/roles",
    operation_id = "listChatRoles",
    tag = super::super::TAG,
    params(("organization_id" = common::OrganizationId, Path, description = "Organization identifier")),
    responses(
        (status = 200, description = "Roles a channel can be restricted to, by id and name", body = inline(DataEnvelope<Vec<ChatRoleResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden — requires MANAGE_CHANNELS"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: OrgChatRolesPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<ChatRoleResponse>>, ApiError> {
    require_permission(&state, &identity, path.organization_id, "channel.manage").await?;

    let roles = state.usecase.list_chat_roles(path.organization_id).await?;
    Ok(Response::OK(
        roles.into_iter().map(ChatRoleResponse::from).collect(),
    ))
}
