use auth::Identity;
use axum::{Extension, Json, extract::State};
use discord::{ChannelId, MessageId, UnreadChannel};
use handlers::{ApiError, AppState};
use serde::Serialize;
use utoipa::ToSchema;

use crate::{paths::OrgUnreadPath, require_org_membership, resolve_user_id};

#[derive(Debug, Serialize, ToSchema)]
pub struct UnreadResponse {
    pub channel_ids: Vec<ChannelId>,
    pub channels: Vec<UnreadChannelResponse>,
}

#[derive(Debug, Serialize, ToSchema)]
pub struct UnreadChannelResponse {
    pub channel_id: ChannelId,
    pub unread_count: i64,
    pub first_unread_message_id: MessageId,
}

impl From<UnreadChannel> for UnreadChannelResponse {
    fn from(unread: UnreadChannel) -> Self {
        Self {
            channel_id: unread.channel_id,
            unread_count: unread.unread_count,
            first_unread_message_id: unread.first_unread_message_id,
        }
    }
}

#[utoipa::path(
	get,
	path = "/api/v1/chat/organizations/{organization_id}/unread",
	operation_id = "listUnreadChannels",
	tag = super::super::TAG,
	params(("organization_id" = common::OrganizationId, Path, description = "Organization identifier")),
	responses(
		(status = 200, description = "Caller's unread channel ids in this org", body = UnreadResponse),
		(status = 401, description = "Unauthorized"),
		(status = 403, description = "Forbidden"),
	),
	security(("bearer_auth" = []))
)]
pub async fn handler(
    path: OrgUnreadPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Json<UnreadResponse>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;
    let user_id = resolve_user_id(&state, &identity).await?;

    let summaries = state
        .usecase
        .list_unread_summaries(user_id, path.organization_id)
        .await?;
    let channel_ids = summaries.iter().map(|unread| unread.channel_id).collect();

    Ok(Json(UnreadResponse {
        channel_ids,
        channels: summaries.into_iter().map(Into::into).collect(),
    }))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unread_response_serializes_channel_ids() {
        use std::str::FromStr;

        let id_a = ChannelId::from_str("018f0000-0000-7000-8000-000000000001").unwrap();
        let id_b = ChannelId::from_str("018f0000-0000-7000-8000-000000000002").unwrap();
        let resp = UnreadResponse {
            channel_ids: vec![id_a, id_b],
            channels: vec![],
        };
        let json = serde_json::to_string(&resp).expect("must serialize");
        assert!(
            json.contains("018f0000-0000-7000-8000-000000000001"),
            "first channel id must appear in JSON; got: {json}"
        );
        assert!(
            json.contains("018f0000-0000-7000-8000-000000000002"),
            "second channel id must appear in JSON; got: {json}"
        );
        assert!(
            json.contains("channel_ids"),
            "field key 'channel_ids' must appear in JSON; got: {json}"
        );
        assert_eq!(resp.channel_ids.len(), 2);
    }
}
