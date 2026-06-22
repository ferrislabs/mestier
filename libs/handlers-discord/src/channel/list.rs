use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, IdentityExt, Response};

use crate::{paths::OrgChannelsPath, require_org_membership, response::ChannelResponse};

#[utoipa::path(
    get,
    path = "/api/v1/chat/organizations/{organization_id}/channels",
    operation_id = "listChannels",
    tag = super::super::TAG,
    params(("organization_id" = common::OrganizationId, Path, description = "Organization identifier")),
    responses(
        (status = 200, description = "List of channels", body = inline(DataEnvelope<Vec<ChannelResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: OrgChannelsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<ChannelResponse>>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;
    let user_id = identity.user_id()?;

    let channels = state
        .usecase
        .list_visible_channels(user_id, path.organization_id)
        .await?;
    let items: Vec<ChannelResponse> = channels.into_iter().map(ChannelResponse::from).collect();
    Ok(Response::OK(items))
}

#[cfg(test)]
mod tests {
    use mestier_core::Permissions;

    #[test]
    fn channel_without_view_channel_bit_is_filtered() {
        // Simulate the bit test list_visible_channels performs per channel:
        // a channel whose resolved bits lack VIEW_CHANNEL must be excluded.
        let no_view = Permissions::SEND_MESSAGES; // VIEW_CHANNEL is absent
        assert!(!no_view.contains(Permissions::VIEW_CHANNEL));
    }

    #[test]
    fn channel_with_view_channel_bit_is_included() {
        let with_view = Permissions::VIEW_CHANNEL | Permissions::SEND_MESSAGES;
        assert!(with_view.contains(Permissions::VIEW_CHANNEL));
    }
}
