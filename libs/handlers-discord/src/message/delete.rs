use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, IdentityExt, Response};

use crate::{EmptyResponse, paths::MessagePath, require_permission};

#[utoipa::path(
	delete,
	path = "/api/v1/chat/messages/{message_id}",
	operation_id = "deleteMessage",
	tag = super::super::TAG,
	params(("message_id" = discord::MessageId, Path, description = "Message identifier")),
	responses(
		(status = 204, description = "Message deleted"),
		(status = 401, description = "Unauthorized"),
		(status = 403, description = "Forbidden — author or message.delete_any required"),
		(status = 404, description = "Message not found"),
	),
	security(("bearer_auth" = []))
)]
pub async fn handler(
    path: MessagePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<EmptyResponse>, ApiError> {
    let message = state.usecase.get_message(path.message_id).await?;
    let user_id = identity.user_id()?;

    // Allow if the caller is the message author; otherwise require moderator permission.
    if message.author_user_id != Some(user_id) {
        require_permission(
            &state,
            &identity,
            message.organization_id,
            "message.delete_any",
        )
        .await?;
    }

    state.usecase.delete_message(path.message_id).await?;
    Ok(Response::NoContent)
}
