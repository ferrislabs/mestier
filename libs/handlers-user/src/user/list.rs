use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::{
    paths::UsersPath,
    require_known_user,
    response::{UserResponse, user_to_response},
};

#[utoipa::path(
	get,
	path = "/api/v1/users",
	operation_id = "listUsers",
	tag = super::super::TAG,
	responses(
		(status = 200, description = "List of active users", body = inline(DataEnvelope<Vec<UserResponse>>)),
		(status = 401, description = "Unauthorized"),
		(status = 403, description = "Forbidden"),
	),
	security(("bearer_auth" = []))
)]
pub async fn handler(
    _path: UsersPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<Vec<UserResponse>>, ApiError> {
    require_known_user(&state, &identity).await?;

    let users = state.usecase.list_users().await?;
    let items: Vec<UserResponse> = users.into_iter().map(user_to_response).collect();

    Ok(Response::OK(items))
}

#[cfg(test)]
mod tests {
    use chrono::Utc;
    use mestier_core::{User, UserId};
    use uuid::Uuid;

    use crate::response::user_to_response;

    fn sample_user() -> User {
        User {
            id: UserId(Uuid::new_v4()),
            sub: "iam-abc-123".into(),
            email: "bob@example.com".into(),
            username: "bob".into(),
            name: "Bob Smith".into(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
            deleted_at: None,
        }
    }

    #[test]
    fn user_to_response_maps_sub_to_id() {
        let r = user_to_response(sample_user());
        assert_eq!(r.id, "iam-abc-123");
        assert_eq!(r.email, "bob@example.com");
        assert_eq!(r.username, "bob");
        assert_eq!(r.name, Some("Bob Smith".into()));
        assert!(r.enabled);
    }

    #[test]
    fn user_to_response_empty_name_becomes_none() {
        let mut u = sample_user();
        u.name = String::new();
        let r = user_to_response(u);
        assert!(r.name.is_none());
    }
}
