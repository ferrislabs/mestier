use auth::Identity;
use axum::{Extension, extract::State};
use discord::StartTypingCommand;
use handlers::{ApiError, AppState, Response};

use crate::{EmptyResponse, paths::ChannelTypingPath, require_org_membership, resolve_user_id};

#[utoipa::path(
	post,
	path = "/api/v1/chat/channels/{channel_id}/typing",
	operation_id = "startTyping",
	tag = super::super::TAG,
	params(("channel_id" = discord::ChannelId, Path, description = "Channel identifier")),
	responses(
		(status = 204, description = "Typing indicator broadcast (no DB write)"),
		(status = 401, description = "Unauthorized"),
		(status = 403, description = "Forbidden"),
	),
	security(("bearer_auth" = []))
)]
pub async fn handler(
    path: ChannelTypingPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<EmptyResponse>, ApiError> {
    let channel = state.usecase.get_channel(path.channel_id).await?;
    require_org_membership(&state, &identity, channel.organization_id).await?;
    let user_id = resolve_user_id(&state, &identity).await?;

    state
        .usecase
        .start_typing(StartTypingCommand {
            organization_id: channel.organization_id,
            channel_id: path.channel_id,
            user_id,
        })
        .await?;

    Ok(Response::NoContent)
}
