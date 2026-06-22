use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::{paths::ChannelPath, require_org_membership, response::ChannelResponse};

#[utoipa::path(
    get,
    path = "/api/v1/channels/{channel_id}",
    operation_id = "getChannel",
    tag = super::super::TAG,
    params(("channel_id" = discord::ChannelId, Path, description = "Channel identifier")),
    responses(
        (status = 200, description = "Channel detail", body = inline(DataEnvelope<ChannelResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Channel not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: ChannelPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<ChannelResponse>, ApiError> {
    let channel = state.usecase.get_channel(path.channel_id).await?;
    require_org_membership(&state, &identity, channel.organization_id).await?;
    Ok(Response::OK(channel.into()))
}
