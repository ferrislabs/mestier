use auth::Identity;
use axum::{Extension, Json, extract::State};
use discord::UpdateMessageCommand;
use handlers::{ApiError, AppState, DataEnvelope, IdentityExt, Response};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::MessagePath, require_org_membership, response::MessageResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateMessageRequest {
    pub content: String,
}

#[utoipa::path(
	patch,
	path = "/api/v1/chat/messages/{message_id}",
	operation_id = "updateMessage",
	tag = super::super::TAG,
	params(("message_id" = discord::MessageId, Path, description = "Message identifier")),
	request_body = UpdateMessageRequest,
	responses(
		(status = 200, description = "Message updated", body = inline(DataEnvelope<MessageResponse>)),
		(status = 400, description = "Validation failed"),
		(status = 401, description = "Unauthorized"),
		(status = 403, description = "Forbidden — author only"),
		(status = 404, description = "Message not found"),
	),
	security(("bearer_auth" = []))
)]
pub async fn handler(
    path: MessagePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<UpdateMessageRequest>,
) -> Result<Response<MessageResponse>, ApiError> {
    let message = state.usecase.get_message(path.message_id).await?;
    require_org_membership(&state, &identity, message.organization_id).await?;

    let user_id = identity.user_id()?;

    let updated = state
        .usecase
        .update_message(UpdateMessageCommand {
            id: path.message_id,
            requester: user_id,
            content: payload.content,
            components: None,
        })
        .await?;

    Ok(Response::OK(updated.into()))
}
