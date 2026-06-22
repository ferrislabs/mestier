use auth::Identity;
use axum::{Extension, extract::State};
use discord::RemoveReactionCommand;
use handlers::{ApiError, AppState, Response};

use crate::{EmptyResponse, paths::ReactionPath, require_org_membership, resolve_user_id};

#[utoipa::path(
	delete,
	path = "/api/v1/chat/messages/{message_id}/reactions/{emoji}",
	operation_id = "removeReaction",
	tag = super::super::TAG,
	params(
		("message_id" = discord::MessageId, Path, description = "Message identifier"),
		("emoji" = String, Path, description = "Unicode emoji (URL-encoded)"),
	),
	responses(
		(status = 204, description = "Reaction removed"),
		(status = 401, description = "Unauthorized"),
		(status = 403, description = "Forbidden"),
		(status = 404, description = "Message not found"),
	),
	security(("bearer_auth" = []))
)]
pub async fn handler(
    path: ReactionPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<EmptyResponse>, ApiError> {
    let message = state.usecase.get_message(path.message_id).await?;
    require_org_membership(&state, &identity, message.organization_id).await?;

    let user_id = resolve_user_id(&state, &identity).await?;

    state
        .usecase
        .remove_reaction(RemoveReactionCommand {
            message_id: path.message_id,
            emoji: path.emoji,
            user_id,
        })
        .await?;

    Ok(Response::NoContent)
}
