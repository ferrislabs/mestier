use auth::Identity;
use axum::{Extension, extract::State};
use discord::AddReactionCommand;
use handlers::{ApiError, AppState, IdentityExt, Response};

use crate::{EmptyResponse, paths::ReactionPath, require_org_membership};

#[utoipa::path(
	put,
	path = "/api/v1/chat/messages/{message_id}/reactions/{emoji}",
	operation_id = "addReaction",
	tag = super::super::TAG,
	params(
		("message_id" = discord::MessageId, Path, description = "Message identifier"),
		("emoji" = String, Path, description = "Unicode emoji (URL-encoded)"),
	),
	responses(
		(status = 204, description = "Reaction added"),
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

    let user_id = identity.user_id()?;

    state
        .usecase
        .add_reaction(AddReactionCommand {
            message_id: path.message_id,
            emoji: path.emoji,
            user_id,
        })
        .await?;

    Ok(Response::NoContent)
}
