use auth::Identity;
use axum::{
    Extension,
    extract::{Query, State},
};
use chrono::{DateTime, Utc};
use discord::{ChannelId, MessageId, NotificationId};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, ToSchema};
use uuid::Uuid;

use crate::{paths::OrgNotificationsPath, require_org_membership};

const DEFAULT_LIMIT: i64 = 50;
const MAX_LIMIT: i64 = 100;

#[derive(Debug, Deserialize, IntoParams)]
pub struct NotificationListQuery {
    pub unread_only: Option<bool>,
    pub before: Option<Uuid>,
    pub limit: Option<i64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct NotificationResponse {
    pub id: NotificationId,
    pub channel_id: ChannelId,
    pub message_id: MessageId,
    /// Notification kind: "MENTION" or "REPLY".
    pub kind: String,
    pub read_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl From<discord::Notification> for NotificationResponse {
    fn from(n: discord::Notification) -> Self {
        Self {
            id: n.id,
            channel_id: n.channel_id,
            message_id: n.message_id,
            kind: n.kind.to_string(),
            read_at: n.read_at,
            created_at: n.created_at,
        }
    }
}

fn resolve_limit(requested: Option<i64>) -> i64 {
    requested.unwrap_or(DEFAULT_LIMIT).clamp(1, MAX_LIMIT)
}

#[utoipa::path(
	get,
	path = "/api/v1/chat/organizations/{organization_id}/notifications",
	operation_id = "listNotifications",
	tag = super::super::TAG,
	params(
		("organization_id" = common::OrganizationId, Path, description = "Organization identifier"),
		NotificationListQuery,
	),
	responses(
		(status = 200, description = "Caller's notifications in this org (newest first)", body = inline(DataEnvelope<Vec<NotificationResponse>>)),
		(status = 401, description = "Unauthorized"),
		(status = 403, description = "Forbidden"),
	),
	security(("bearer_auth" = []))
)]
pub async fn handler(
    path: OrgNotificationsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Query(query): Query<NotificationListQuery>,
) -> Result<Response<Vec<NotificationResponse>>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;

    // Resolve the caller via find_user_by_sub → user.id (DB users.id).
    // Do NOT use identity.user_id() — it parses the OIDC sub, which differs
    // from users.id; notification queries filter on users.id so the wrong id
    // returns nothing.
    let user = state
        .usecase
        .find_user_by_sub(identity.id())
        .await?
        .ok_or(ApiError::Forbidden)?;

    let limit = resolve_limit(query.limit);
    let before = query.before.map(NotificationId);

    let notifications = state
        .usecase
        .list_notifications(
            user.id,
            path.organization_id,
            query.unread_only.unwrap_or(false),
            before,
            limit,
        )
        .await?;

    let items: Vec<NotificationResponse> = notifications
        .into_iter()
        .map(NotificationResponse::from)
        .collect();
    Ok(Response::OK(items))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;

    #[test]
    fn resolve_limit_none_returns_default() {
        assert_eq!(resolve_limit(None), DEFAULT_LIMIT);
    }

    #[test]
    fn resolve_limit_some_within_bounds() {
        assert_eq!(resolve_limit(Some(10)), 10);
    }

    #[test]
    fn resolve_limit_exceeds_max_is_capped() {
        assert_eq!(resolve_limit(Some(500)), MAX_LIMIT);
    }

    #[test]
    fn resolve_limit_zero_is_floored_to_one() {
        assert_eq!(resolve_limit(Some(0)), 1);
    }

    #[test]
    fn resolve_limit_negative_is_floored_to_one() {
        assert_eq!(resolve_limit(Some(-5)), 1);
    }

    #[test]
    fn notification_response_serializes_kind_as_string_and_omits_org_and_user() {
        use chrono::Utc;
        use discord::{ChannelId, MessageId, NotificationId};

        let id = NotificationId::from_str("018f0000-0000-7000-8000-000000000001").unwrap();
        let ch = ChannelId::from_str("018f0000-0000-7000-8000-000000000002").unwrap();
        let msg = MessageId::from_str("018f0000-0000-7000-8000-000000000003").unwrap();
        let resp = NotificationResponse {
            id,
            channel_id: ch,
            message_id: msg,
            kind: "MENTION".to_owned(),
            read_at: None,
            created_at: Utc::now(),
        };
        let json = serde_json::to_string(&resp).expect("must serialize");
        assert!(
            json.contains("MENTION"),
            "kind must appear as MENTION in JSON; got: {json}"
        );
        assert!(
            json.contains("channel_id"),
            "channel_id field must appear; got: {json}"
        );
        assert!(
            json.contains("message_id"),
            "message_id field must appear; got: {json}"
        );
        assert!(
            !json.contains("organization_id"),
            "org_id must NOT be exposed; got: {json}"
        );
        assert!(
            !json.contains("user_id"),
            "user_id must NOT be exposed; got: {json}"
        );
    }
}
