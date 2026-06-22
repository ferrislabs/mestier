use auth::Identity;
use axum::{Extension, Json, extract::State};
use discord::{CreateMessageCommand, MessageAuthor};
use handlers::{ApiError, AppState, DataEnvelope, IdentityExt, Response};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::ChannelMessagesPath, require_org_membership, response::MessageResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateMessageRequest {
    pub content: String,
}

#[utoipa::path(
	post,
	path = "/api/v1/chat/channels/{channel_id}/messages",
	operation_id = "createMessage",
	tag = super::super::TAG,
	params(("channel_id" = discord::ChannelId, Path, description = "Channel identifier")),
	request_body = CreateMessageRequest,
	responses(
		(status = 201, description = "Message sent", body = inline(DataEnvelope<MessageResponse>)),
		(status = 400, description = "Validation failed"),
		(status = 401, description = "Unauthorized"),
		(status = 403, description = "Forbidden"),
	),
	security(("bearer_auth" = []))
)]
pub async fn handler(
    path: ChannelMessagesPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateMessageRequest>,
) -> Result<Response<MessageResponse>, ApiError> {
    let channel = state.usecase.get_channel(path.channel_id).await?;
    require_org_membership(&state, &identity, channel.organization_id).await?;

    if payload.content.trim().is_empty() {
        return Err(ApiError::Validation(
            "message content must not be blank".into(),
        ));
    }

    let user_id = identity.user_id()?;

    let message = state
        .usecase
        .create_message(CreateMessageCommand {
            organization_id: channel.organization_id,
            channel_id: path.channel_id,
            author: MessageAuthor::User(user_id),
            content: payload.content,
            components: None,
            attachments: vec![], // Plan 3 will add attachment support to this handler
        })
        .await?;

    Ok(Response::Created(message.into()))
}
