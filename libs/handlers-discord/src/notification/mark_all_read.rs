use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, Response};

use crate::{EmptyResponse, paths::OrgNotificationsReadAllPath, require_org_membership};

#[utoipa::path(
	put,
	path = "/api/v1/chat/organizations/{organization_id}/notifications/read-all",
	operation_id = "markAllNotificationsRead",
	tag = super::super::TAG,
	params(("organization_id" = common::OrganizationId, Path, description = "Organization identifier")),
	responses(
		(status = 204, description = "All unread notifications in this org marked read (no-op if none)"),
		(status = 401, description = "Unauthorized"),
		(status = 403, description = "Forbidden"),
	),
	security(("bearer_auth" = []))
)]
pub async fn handler(
    path: OrgNotificationsReadAllPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<EmptyResponse>, ApiError> {
    require_org_membership(&state, &identity, path.organization_id).await?;

    // Resolve the caller via find_user_by_sub → user.id (DB users.id).
    // Do NOT use identity.user_id() — it parses the OIDC sub, which differs
    // from users.id; the UPDATE filters by users.id so the wrong id would silently
    // update zero rows instead of all the caller's notifications.
    let user = state
        .usecase
        .find_user_by_sub(identity.id())
        .await?
        .ok_or(ApiError::Forbidden)?;

    state
        .usecase
        .mark_all_notifications_read(user.id, path.organization_id)
        .await?;

    Ok(Response::NoContent)
}

#[cfg(test)]
mod tests {
    #[test]
    fn mark_all_read_path_segments_do_not_conflict_with_list_path() {
        let list_path = "/api/v1/chat/organizations/{organization_id}/notifications";
        let read_all_path = "/api/v1/chat/organizations/{organization_id}/notifications/read-all";
        assert_ne!(
            list_path, read_all_path,
            "list and mark-all-read paths must be distinct"
        );
        assert!(
            read_all_path.ends_with("/read-all"),
            "mark-all-read path must end with /read-all; got: {read_all_path}"
        );
    }
}
