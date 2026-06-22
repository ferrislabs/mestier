use auth::Identity;
use axum::{Extension, Json, extract::State};
use discord::{MarkChannelReadCommand, MessageId};
use handlers::{ApiError, AppState, Response};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{EmptyResponse, paths::ChannelReadPath, require_org_membership, resolve_user_id};

#[derive(Debug, Deserialize, ToSchema)]
pub struct MarkChannelReadRequest {
    pub message_id: MessageId,
}

#[utoipa::path(
	put,
	path = "/api/v1/chat/channels/{channel_id}/read",
	operation_id = "markChannelRead",
	tag = super::super::TAG,
	params(("channel_id" = discord::ChannelId, Path, description = "Channel identifier")),
	request_body = MarkChannelReadRequest,
	responses(
		(status = 204, description = "Read marker advanced (or no-op if already up-to-date)"),
		(status = 401, description = "Unauthorized"),
		(status = 403, description = "Forbidden"),
		(status = 404, description = "Message not found in this channel"),
	),
	security(("bearer_auth" = []))
)]
pub async fn handler(
    path: ChannelReadPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<MarkChannelReadRequest>,
) -> Result<Response<EmptyResponse>, ApiError> {
    let channel = state.usecase.get_channel(path.channel_id).await?;
    require_org_membership(&state, &identity, channel.organization_id).await?;
    let user_id = resolve_user_id(&state, &identity).await?;

    state
        .usecase
        .mark_channel_read(MarkChannelReadCommand {
            organization_id: channel.organization_id,
            channel_id: path.channel_id,
            user_id,
            message_id: payload.message_id,
        })
        .await?;

    Ok(Response::NoContent)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mark_channel_read_request_deserializes_message_id() {
        let json = r#"{"message_id":"018f0000-0000-7000-8000-000000000001"}"#;
        let req: MarkChannelReadRequest =
            serde_json::from_str(json).expect("must deserialize MarkChannelReadRequest");
        assert_eq!(
            req.message_id.to_string(),
            "018f0000-0000-7000-8000-000000000001"
        );
    }
}
