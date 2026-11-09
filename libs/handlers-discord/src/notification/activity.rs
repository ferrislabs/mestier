use auth::Identity;
use axum::{
    Extension,
    extract::{Query, State},
};
use chrono::{DateTime, Utc};
use discord::{AuthorType, ChannelId, MessageId, NotificationId, UserId};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::application::discord_notification::ActivityEntry;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};

use crate::{paths::OrgActivityPath, require_org_membership, resolve_user_id};

const DEFAULT_LIMIT: i64 = 50;
const MAX_LIMIT: i64 = 100;
const EXCERPT_CHARS: usize = 200;

#[derive(Debug, Deserialize, IntoParams)]
pub struct ActivityQuery {
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct ActivityEntryResponse {
    /// Why this is here: "MENTION", "REPLY" or "UNREAD".
    pub kind: String,
    pub channel_id: ChannelId,
    pub message_id: MessageId,
    pub author_user_id: Option<UserId>,
    pub author_type: AuthorType,
    /// The start of the message, already readable by the caller.
    pub excerpt: String,
    pub created_at: DateTime<Utc>,
    pub read: bool,
    /// Set on a mention, to mark it read.
    pub notification_id: Option<NotificationId>,
    /// Set on an unread entry: how many messages wait in the channel.
    pub unread_count: Option<i64>,
}

impl From<ActivityEntry> for ActivityEntryResponse {
    fn from(entry: ActivityEntry) -> Self {
        Self {
            kind: entry.kind.as_str().to_owned(),
            channel_id: entry.channel_id,
            message_id: entry.message.id,
            author_user_id: entry.message.author_user_id,
            author_type: entry.message.author_type,
            excerpt: excerpt(&entry.message.content),
            created_at: entry.created_at,
            read: entry.read,
            notification_id: entry.notification_id,
            unread_count: entry.unread_count,
        }
    }
}

fn excerpt(content: &str) -> String {
    content.chars().take(EXCERPT_CHARS).collect()
}

#[utoipa::path(
	get,
	path = "/api/v1/chat/organizations/{organization_id}/activity",
	operation_id = "listActivity",
	tag = super::super::TAG,
	params(
		("organization_id" = common::OrganizationId, Path, description = "Organization identifier"),
		ActivityQuery,
	),
	responses(
		(status = 200, description = "What the caller has to read: mentions and unread messages of followed channels (newest first)", body = inline(DataEnvelope<Vec<ActivityEntryResponse>>)),
		(status = 401, description = "Unauthorized"),
		(status = 403, description = "Forbidden"),
	),
	security(("bearer_auth" = []))
)]
pub async fn handler(
    path: OrgActivityPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(query): Query<ActivityQuery>,
) -> Result<Response<Vec<ActivityEntryResponse>>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;
    let user_id = resolve_user_id(&state, &identity).await?;
    let limit = query.limit.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT);

    let visible = state
        .usecase
        .list_visible_channels(user_id, path.organization_id)
        .await?;
    let entries = state
        .usecase
        .list_activity(user_id, path.organization_id, visible, limit)
        .await?;

    Ok(Response::OK(
        entries
            .into_iter()
            .map(ActivityEntryResponse::from)
            .collect(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_excerpt_keeps_the_start_of_a_long_message() {
        let long = "é".repeat(EXCERPT_CHARS + 50);

        assert_eq!(excerpt(&long).chars().count(), EXCERPT_CHARS);
    }

    #[test]
    fn a_short_message_is_its_own_excerpt() {
        assert_eq!(excerpt("Bonjour"), "Bonjour");
    }
}
