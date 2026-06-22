use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::{paths::OrgChannelsPath, require_org_membership, response::ChannelResponse};

#[utoipa::path(
    get,
    path = "/api/v1/chat/organizations/{organization_id}/channels",
    operation_id = "listChannels",
    tag = super::super::TAG,
    params(("organization_id" = common::OrganizationId, Path, description = "Organization identifier")),
    responses(
        (status = 200, description = "List of channels", body = inline(DataEnvelope<Vec<ChannelResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: OrgChannelsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<ChannelResponse>>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;

    let channels = state.usecase.list_channels(path.organization_id).await?;
    let items: Vec<ChannelResponse> = channels.into_iter().map(ChannelResponse::from).collect();
    Ok(Response::OK(items))
}
