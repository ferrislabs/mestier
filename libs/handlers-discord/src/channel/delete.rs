use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, Response};

use crate::{EmptyResponse, paths::ChannelPath, require_permission};

#[utoipa::path(
    delete,
    path = "/api/v1/chat/channels/{channel_id}",
    operation_id = "deleteChannel",
    tag = super::super::TAG,
    params(("channel_id" = discord::ChannelId, Path, description = "Channel identifier")),
    responses(
        (status = 204, description = "Channel deleted"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden — requires MANAGE_CHANNELS"),
        (status = 404, description = "Channel not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: ChannelPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<EmptyResponse>, ApiError> {
    let existing = state.usecase.get_channel(path.channel_id).await?;
    require_permission(
        &state,
        &identity,
        existing.organization_id,
        "channel.manage",
    )
    .await?;
    state.usecase.delete_channel(path.channel_id).await?;
    Ok(Response::NoContent)
}
