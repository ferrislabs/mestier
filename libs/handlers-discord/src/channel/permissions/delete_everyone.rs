use auth::Identity;
use axum::{Extension, extract::State};
use discord::{DeleteChannelOverwrite, OverwriteTarget};
use handlers::{ApiError, AppState, Response, resolve_user_id};

use crate::{EmptyResponse, paths::ChannelOverwriteEveryonePath, require_permission};

#[utoipa::path(
    delete,
    path = "/api/v1/chat/channels/{channel_id}/permissions/everyone",
    operation_id = "deleteEveryoneOverwrite",
    tag = super::super::super::TAG,
    params(("channel_id" = discord::ChannelId, Path, description = "Channel identifier")),
    responses(
        (status = 204, description = "EVERYONE overwrite deleted (or was already absent)"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden — requires org-level MANAGE_CHANNELS"),
        (status = 404, description = "Channel not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: ChannelOverwriteEveryonePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<EmptyResponse>, ApiError> {
    let channel = state.usecase.get_channel(path.channel_id).await?;
    require_permission(&state, &identity, channel.organization_id, "channel.manage").await?;
    let actor = resolve_user_id(&state, &identity).await?;

    state
        .usecase
        .acting_as(actor)
        .delete_channel_overwrite(DeleteChannelOverwrite {
            channel_id: path.channel_id,
            target: OverwriteTarget::Everyone,
        })
        .await?;

    Ok(Response::NoContent)
}

#[cfg(test)]
mod tests {
    use discord::OverwriteTarget;

    #[test]
    fn everyone_target_is_unit_variant() {
        // Ensure the Everyone variant is zero-size (no inner id) — the delete path
        // must not require a target_id.
        let target = OverwriteTarget::Everyone;
        assert!(matches!(target, OverwriteTarget::Everyone));
    }
}
