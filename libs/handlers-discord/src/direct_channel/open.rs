use auth::Identity;
use axum::{Extension, Json, extract::State};
use common::UserId;
use handlers::{ApiError, AppState, DataEnvelope, Response};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    paths::OrgDirectChannelsPath, require_org_membership, resolve_user_id,
    response::DirectChannelResponse,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct OpenDirectChannelRequest {
    pub user_id: UserId,
}

#[utoipa::path(
    post,
    path = "/api/v1/chat/organizations/{organization_id}/direct-channels",
    operation_id = "openDirectChannel",
    tag = super::super::TAG,
    params(("organization_id" = common::OrganizationId, Path, description = "Organization identifier")),
    request_body = OpenDirectChannelRequest,
    responses(
        (status = 200, description = "The conversation with that member, created on first use", body = inline(DataEnvelope<DirectChannelResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "The other person is not a member of this organization"),
        (status = 409, description = "A direct message needs two different people"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: OrgDirectChannelsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<OpenDirectChannelRequest>,
) -> Result<Response<DirectChannelResponse>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;
    let caller = resolve_user_id(&state, &identity).await?;

    let direct = state
        .usecase
        .open_direct_channel(path.organization_id, caller, payload.user_id)
        .await?;
    Ok(Response::OK(DirectChannelResponse::for_user(
        direct, caller,
    )?))
}
