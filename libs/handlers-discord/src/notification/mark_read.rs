use auth::Identity;
use axum::{Extension, extract::State};
use discord::MarkNotificationRead;
use handlers::{ApiError, AppState, Response};

use crate::{EmptyResponse, paths::NotificationReadPath};

#[utoipa::path(
	put,
	path = "/api/v1/chat/notifications/{notification_id}/read",
	operation_id = "markNotificationRead",
	tag = super::super::TAG,
	params(("notification_id" = discord::NotificationId, Path, description = "Notification identifier")),
	responses(
		(status = 204, description = "Notification marked read (no-op if already read or not owned by caller)"),
		(status = 401, description = "Unauthorized"),
		(status = 403, description = "Forbidden"),
	),
	security(("bearer_auth" = []))
)]
pub async fn handler(
    path: NotificationReadPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<EmptyResponse>, ApiError> {
    // Resolve the caller via find_user_by_sub → user.id (DB users.id).
    // Do NOT use identity.user_id() — it parses the OIDC sub, which differs
    // from users.id; the repo filters by users.id so the wrong id is a silent no-op.
    let user = state
        .usecase
        .find_user_by_sub(identity.id())
        .await?
        .ok_or(ApiError::Forbidden)?;

    state
        .usecase
        .acting_as(user.id)
        .mark_notification_read(MarkNotificationRead {
            notification_id: path.notification_id,
            user_id: user.id,
        })
        .await?;

    Ok(Response::NoContent)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use discord::NotificationId;

    #[test]
    fn notification_id_roundtrips_through_str() {
        let raw = "018f0000-0000-7000-8000-000000000042";
        let id = NotificationId::from_str(raw).expect("must parse a valid UUIDv7 notification id");
        assert_eq!(
            id.to_string(),
            raw,
            "NotificationId Display must round-trip the original string"
        );
    }

    #[test]
    fn mark_read_is_no_op_for_non_owner() {
        use common::UserId;
        use discord::MarkNotificationRead;

        let notification_id =
            NotificationId::from_str("018f0000-0000-7000-8000-000000000001").unwrap();
        let user_id = UserId::from_str("018f0000-0000-7000-8000-000000000002").unwrap();
        let cmd = MarkNotificationRead {
            notification_id,
            user_id,
        };
        assert_eq!(
            cmd.user_id.to_string(),
            "018f0000-0000-7000-8000-000000000002"
        );
        assert_eq!(
            cmd.notification_id.to_string(),
            "018f0000-0000-7000-8000-000000000001"
        );
    }
}
