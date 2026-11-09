use auth::Identity;
use axum::{Extension, Json, extract::State};
use discord::{ChannelId, NotificationLevel};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use crate::{
    EmptyResponse, Permissions,
    paths::{ChannelNotificationLevelPath, OrgNotificationLevelsPath},
    require_channel_permission, require_org_membership, resolve_user_id,
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct SetNotificationLevelRequest {
    pub level: NotificationLevel,
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct NotificationLevelResponse {
    pub channel_id: ChannelId,
    pub level: NotificationLevel,
}

#[utoipa::path(
	put,
	path = "/api/v1/chat/channels/{channel_id}/notification-level",
	operation_id = "setNotificationLevel",
	tag = super::super::TAG,
	params(("channel_id" = discord::ChannelId, Path, description = "Channel identifier")),
	request_body = SetNotificationLevelRequest,
	responses(
		(status = 204, description = "The caller's level for this channel is recorded"),
		(status = 401, description = "Unauthorized"),
		(status = 403, description = "Forbidden"),
		(status = 404, description = "Channel not found"),
	),
	security(("bearer_auth" = []))
)]
pub async fn set_handler(
    path: ChannelNotificationLevelPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<SetNotificationLevelRequest>,
) -> Result<Response<EmptyResponse>, ApiError> {
    require_channel_permission(
        &state,
        &identity,
        path.channel_id,
        Permissions::VIEW_CHANNEL,
    )
    .await?;
    let user_id = resolve_user_id(&state, &identity).await?;

    state
        .usecase
        .set_notification_level(user_id, path.channel_id, payload.level)
        .await?;

    Ok(Response::NoContent)
}

#[utoipa::path(
	get,
	path = "/api/v1/chat/organizations/{organization_id}/notification-levels",
	operation_id = "listNotificationLevels",
	tag = super::super::TAG,
	params(("organization_id" = common::OrganizationId, Path, description = "Organization identifier")),
	responses(
		(status = 200, description = "The caller's explicit levels; a channel not listed has its default", body = inline(DataEnvelope<Vec<NotificationLevelResponse>>)),
		(status = 401, description = "Unauthorized"),
		(status = 403, description = "Forbidden"),
	),
	security(("bearer_auth" = []))
)]
pub async fn list_handler(
    path: OrgNotificationLevelsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<NotificationLevelResponse>>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;
    let user_id = resolve_user_id(&state, &identity).await?;

    let levels = state
        .usecase
        .list_notification_levels(user_id, path.organization_id)
        .await?;

    Ok(Response::OK(
        levels
            .into_iter()
            .map(|(channel_id, level)| NotificationLevelResponse { channel_id, level })
            .collect(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_request_reads_a_level_in_capitals() {
        let request: SetNotificationLevelRequest =
            serde_json::from_str(r#"{"level":"NOTHING"}"#).expect("a valid level");

        assert_eq!(request.level, NotificationLevel::Nothing);
    }

    #[test]
    fn an_unknown_level_is_refused() {
        assert!(
            serde_json::from_str::<SetNotificationLevelRequest>(r#"{"level":"LOUD"}"#).is_err()
        );
    }
}
