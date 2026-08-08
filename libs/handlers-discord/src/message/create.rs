use auth::Identity;
use axum::{Extension, Json, extract::State};
use discord::{CreateMessageCommand, MessageAuthor};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use serde::Deserialize;
use utoipa::ToSchema;

use mestier_core::Permissions;

use crate::{
    paths::ChannelMessagesPath, require_channel_permission, require_org_membership,
    resolve_user_id, response::MessageResponse,
};

/// Per-attachment descriptor included in a message create request.
/// The client uploads each file via `POST /api/v1/files?folder=attachments` first,
/// then references the returned `key` as `storage_key` here.
#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateMessageAttachment {
    pub storage_key: String,
    pub filename: String,
    pub mime_type: String,
    pub size_bytes: i64,
}

impl From<CreateMessageAttachment> for discord::AttachmentInput {
    fn from(a: CreateMessageAttachment) -> Self {
        discord::AttachmentInput {
            storage_key: a.storage_key,
            filename: a.filename,
            mime_type: a.mime_type,
            size_bytes: a.size_bytes,
        }
    }
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateMessageRequest {
    pub content: String,
    #[serde(default)]
    pub attachments: Vec<CreateMessageAttachment>,
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
    require_channel_permission(
        &state,
        &identity,
        path.channel_id,
        Permissions::SEND_MESSAGES,
    )
    .await?;

    if payload.content.trim().is_empty() {
        return Err(ApiError::Validation(
            "message content must not be blank".into(),
        ));
    }

    let user_id = resolve_user_id(&state, &identity).await?;

    let message = state
        .usecase
        .acting_as(user_id)
        .create_message(CreateMessageCommand {
            organization_id: channel.organization_id,
            channel_id: path.channel_id,
            author: MessageAuthor::User(user_id),
            content: payload.content,
            components: None,
            attachments: payload
                .attachments
                .into_iter()
                .map(discord::AttachmentInput::from)
                .collect(),
        })
        .await?;

    Ok(Response::Created(message.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_message_request_defaults_attachments_to_empty() {
        // Simulate deserializing a JSON body that has no "attachments" key.
        let json = r#"{"content": "hello world"}"#;
        let req: CreateMessageRequest =
            serde_json::from_str(json).expect("must deserialize without attachments field");
        assert!(
            req.attachments.is_empty(),
            "attachments must default to empty Vec when absent from JSON"
        );
    }

    #[test]
    fn send_messages_bit_differs_from_view_channel_bit() {
        use mestier_core::Permissions;
        assert_ne!(
            Permissions::SEND_MESSAGES.bits(),
            Permissions::VIEW_CHANNEL.bits(),
            "SEND_MESSAGES and VIEW_CHANNEL must be distinct permission bits"
        );
        assert_eq!(Permissions::SEND_MESSAGES.bits(), 64);
    }

    #[test]
    fn effective_with_only_view_channel_lacks_send_messages() {
        use mestier_core::Permissions;
        let effective = Permissions::VIEW_CHANNEL; // e.g. EVERYONE deny SEND_MESSAGES
        assert!(effective.contains(Permissions::VIEW_CHANNEL));
        assert!(!effective.contains(Permissions::SEND_MESSAGES));
    }

    #[test]
    fn create_message_attachment_maps_to_attachment_input() {
        let dto = CreateMessageAttachment {
            storage_key: "prefix/attachments/018f0000".to_owned(),
            filename: "report.pdf".to_owned(),
            mime_type: "application/pdf".to_owned(),
            size_bytes: 102_400,
        };
        let input: discord::AttachmentInput = dto.into();
        assert_eq!(input.storage_key, "prefix/attachments/018f0000");
        assert_eq!(input.filename, "report.pdf");
        assert_eq!(input.mime_type, "application/pdf");
        assert_eq!(input.size_bytes, 102_400);
    }
}
