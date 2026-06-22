use auth::Identity;
use axum::{Extension, Json, extract::State};
use discord::{CategoryId, UpdateChannelCommand};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use serde::Deserialize;
use utoipa::ToSchema;

use mestier_core::Permissions;

use crate::{paths::ChannelPath, require_channel_permission, response::ChannelResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateChannelRequest {
    pub name: String,
    pub topic: Option<String>,
    pub position: i32,
    pub category_id: Option<CategoryId>,
}

#[utoipa::path(
    patch,
    path = "/api/v1/chat/channels/{channel_id}",
    operation_id = "updateChannel",
    tag = super::super::TAG,
    params(("channel_id" = discord::ChannelId, Path, description = "Channel identifier")),
    request_body = UpdateChannelRequest,
    responses(
        (status = 200, description = "Channel updated", body = inline(DataEnvelope<ChannelResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden — requires MANAGE_CHANNELS"),
        (status = 404, description = "Channel not found"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    path: ChannelPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<UpdateChannelRequest>,
) -> Result<Response<ChannelResponse>, ApiError> {
    require_channel_permission(
        &state,
        &identity,
        path.channel_id,
        Permissions::MANAGE_CHANNELS,
    )
    .await?;

    if payload.name.trim().is_empty() {
        return Err(ApiError::Validation(
            "channel name must not be blank".into(),
        ));
    }

    let updated = state
        .usecase
        .update_channel(UpdateChannelCommand {
            id: path.channel_id,
            category_id: payload.category_id,
            name: payload.name,
            topic: payload.topic,
            position: payload.position,
        })
        .await?;

    Ok(Response::OK(updated.into()))
}

#[cfg(test)]
mod tests {
    use mestier_core::Permissions;

    #[test]
    fn manage_channels_bit_is_8() {
        assert_eq!(Permissions::MANAGE_CHANNELS.bits(), 8);
    }

    #[test]
    fn effective_without_manage_channels_cannot_update() {
        // A member with only VIEW+SEND cannot manage channels.
        let member_effective = Permissions::VIEW_CHANNEL | Permissions::SEND_MESSAGES;
        assert!(!member_effective.contains(Permissions::MANAGE_CHANNELS));
    }

    #[test]
    fn effective_with_manage_channels_can_update() {
        let manager_effective =
            Permissions::VIEW_CHANNEL | Permissions::SEND_MESSAGES | Permissions::MANAGE_CHANNELS;
        assert!(manager_effective.contains(Permissions::MANAGE_CHANNELS));
    }
}
