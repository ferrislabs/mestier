use auth::Identity;
use axum::{Extension, extract::State};
use common::UserId;
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::{paths::ReactionPath, require_org_membership};

#[utoipa::path(
	get,
	path = "/api/v1/chat/messages/{message_id}/reactions/{emoji}",
	operation_id = "listReactors",
	tag = super::super::TAG,
	params(
		("message_id" = discord::MessageId, Path, description = "Message identifier"),
		("emoji" = String, Path, description = "Unicode emoji (URL-encoded)"),
	),
	responses(
		(status = 200, description = "List of users who reacted with this emoji", body = inline(DataEnvelope<Vec<UserId>>)),
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
) -> Result<Response<Vec<UserId>>, ApiError> {
    let message = state.usecase.get_message(path.message_id).await?;
    require_org_membership(&state, &identity, message.organization_id).await?;

    let user_ids: Vec<UserId> = message
        .reactions
        .into_iter()
        .find(|r| r.emoji == path.emoji)
        .map(|r| r.user_ids)
        .unwrap_or_default();

    Ok(Response::OK(user_ids))
}
