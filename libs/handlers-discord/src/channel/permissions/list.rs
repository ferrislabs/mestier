use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::{paths::ChannelPermissionsPath, require_permission, response::OverwriteResponse};

#[utoipa::path(
    get,
    path = "/api/v1/chat/channels/{channel_id}/permissions",
    operation_id = "listChannelPermissions",
    tag = super::super::super::TAG,
    params(("channel_id" = discord::ChannelId, Path, description = "Channel identifier")),
    responses(
        (status = 200, description = "List of permission overwrites for this channel", body = inline(DataEnvelope<Vec<OverwriteResponse>>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden — requires org-level MANAGE_CHANNELS"),
        (status = 404, description = "Channel not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: ChannelPermissionsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<OverwriteResponse>>, ApiError> {
    // Gate on org-level channel.manage so an admin denied VIEW_CHANNEL can still
    // manage this channel's overwrites (no lockout).
    let channel = state.usecase.get_channel(path.channel_id).await?;
    require_permission(&state, &identity, channel.organization_id, "channel.manage").await?;

    let overwrites = state
        .usecase
        .list_channel_overwrites(path.channel_id)
        .await?;
    let items: Vec<OverwriteResponse> = overwrites
        .into_iter()
        .map(OverwriteResponse::from)
        .collect();
    Ok(Response::OK(items))
}

#[cfg(test)]
mod tests {
    use crate::response::OverwriteResponse;
    use chrono::Utc;
    use common::OrganizationId;
    use discord::{ChannelId, ChannelPermissionOverwrite, OverwriteId, OverwriteTarget};
    use std::str::FromStr;

    const UUID_A: &str = "550e8400-e29b-41d4-a716-446655440000";
    const UUID_B: &str = "6ba7b810-9dad-11d1-80b4-00c04fd430c8";
    const UUID_C: &str = "6ba7b811-9dad-11d1-80b4-00c04fd430c8";

    #[test]
    fn overwrite_list_maps_to_response_without_id() {
        let now = Utc::now();
        let channel_id = ChannelId::from_str(UUID_A).unwrap();
        let org_id = OrganizationId::from_str(UUID_B).unwrap();
        let overwrite_id = OverwriteId::from_str(UUID_C).unwrap();

        let overwrite = ChannelPermissionOverwrite {
            id: overwrite_id,
            channel_id,
            organization_id: org_id,
            target: OverwriteTarget::Everyone,
            allow: 32,
            deny: 64,
            created_at: now,
            updated_at: now,
        };
        let resp = OverwriteResponse::from(overwrite);
        let json = serde_json::to_string(&resp).expect("must serialize");
        assert!(
            !json.contains(UUID_C),
            "overwrite id must not appear in response: {json}"
        );
        assert_eq!(resp.target_type, "everyone");
        assert_eq!(resp.allow, 32);
        assert_eq!(resp.deny, 64);
    }
}
