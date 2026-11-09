use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::{
    paths::OrgDirectChannelsPath, require_org_membership, resolve_user_id,
    response::DirectChannelResponse,
};

#[utoipa::path(
    get,
    path = "/api/v1/chat/organizations/{organization_id}/direct-channels",
    operation_id = "listDirectChannels",
    tag = super::super::TAG,
    params(("organization_id" = common::OrganizationId, Path, description = "Organization identifier")),
    responses(
        (status = 200, description = "The caller's direct messages", body = inline(DataEnvelope<Vec<DirectChannelResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: OrgDirectChannelsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<DirectChannelResponse>>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;
    let caller = resolve_user_id(&state, &identity).await?;

    let directs = state
        .usecase
        .list_direct_channels(path.organization_id, caller)
        .await?;
    let items = directs
        .into_iter()
        .map(|direct| DirectChannelResponse::for_user(direct, caller))
        .collect::<Result<Vec<_>, _>>()?;
    Ok(Response::OK(items))
}
