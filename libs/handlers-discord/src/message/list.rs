use auth::Identity;
use axum::{
    Extension,
    extract::{Query, State},
};
use discord::MessageId;
use handlers::{ApiError, AppState, DataEnvelope, Response};
use serde::Deserialize;

use crate::{paths::ChannelMessagesPath, require_org_membership, response::MessageResponse};

#[derive(Debug, Deserialize)]
pub struct MessageCursorQuery {
    pub before: Option<MessageId>,
    pub after: Option<MessageId>,
    pub limit: Option<u64>,
}

impl MessageCursorQuery {
    pub fn effective_limit(&self) -> u64 {
        self.limit.unwrap_or(50).min(100)
    }
}

#[utoipa::path(
	get,
	path = "/api/v1/chat/channels/{channel_id}/messages",
	operation_id = "listMessages",
	tag = super::super::TAG,
	params(
		("channel_id" = discord::ChannelId, Path, description = "Channel identifier"),
		("before" = Option<discord::MessageId>, Query, description = "Return messages before this id"),
		("after"  = Option<discord::MessageId>, Query, description = "Return messages after this id"),
		("limit"  = Option<u64>, Query, description = "Max messages to return (1–100, default 50)"),
	),
	responses(
		(status = 200, description = "Message history", body = inline(DataEnvelope<Vec<MessageResponse>>)),
		(status = 401, description = "Unauthorized"),
		(status = 403, description = "Forbidden"),
	),
	security(("bearer_auth" = []))
)]
pub async fn handler(
    path: ChannelMessagesPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(cursor): Query<MessageCursorQuery>,
) -> Result<Response<Vec<MessageResponse>>, ApiError> {
    let channel = state.usecase.get_channel(path.channel_id).await?;
    require_org_membership(&state, &identity, channel.organization_id).await?;

    let messages = state
        .usecase
        .list_messages(
            path.channel_id,
            cursor.before,
            cursor.after,
            cursor.effective_limit(),
        )
        .await?;

    let items: Vec<MessageResponse> = messages.into_iter().map(MessageResponse::from).collect();
    Ok(Response::OK(items))
}
