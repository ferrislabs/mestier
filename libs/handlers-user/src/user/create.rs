use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use iam::{IamCreateUser, IamProvider};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::UsersPath, require_known_user, response::UserResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateUserRequest {
    pub email: String,
    pub username: String,
    pub name: Option<String>,
    /// When true, ask the IAM to send a credential-setup e-mail.
    pub send_invite_email: bool,
}

#[utoipa::path(
	post,
	path = "/api/v1/users",
	operation_id = "createUser",
	tag = super::super::TAG,
	request_body = CreateUserRequest,
	responses(
		(status = 201, description = "User created", body = inline(DataEnvelope<UserResponse>)),
		(status = 400, description = "Bad request"),
		(status = 401, description = "Unauthorized"),
		(status = 403, description = "Forbidden"),
		(status = 409, description = "User already exists"),
		(status = 422, description = "Validation failed"),
		(status = 500, description = "IAM unavailable"),
	),
	security(("bearer_auth" = []))
)]
pub async fn handler(
    _path: UsersPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<CreateUserRequest>,
) -> Result<Response<UserResponse>, ApiError> {
    require_known_user(&state, &identity).await?;

    let iam_user = state
        .iam
        .create_user(IamCreateUser {
            email: payload.email,
            username: payload.username,
            name: payload.name,
            send_invite_email: payload.send_invite_email,
        })
        .await
        .map_err(ApiError::from)?;

    Ok(Response::Created(iam_user.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use iam::{IamError, IamUser, IamUserId};

    #[test]
    fn iam_user_maps_to_user_response_correctly() {
        let iam_user = IamUser {
            id: IamUserId("iam-abc".into()),
            email: "alice@example.com".into(),
            username: "alice".into(),
            name: Some("Alice".into()),
            email_verified: false,
            enabled: true,
        };

        let response: UserResponse = iam_user.into();
        assert_eq!(response.id, "iam-abc");
        assert_eq!(response.email, "alice@example.com");
        assert_eq!(response.name, Some("Alice".into()));
        assert!(!response.email_verified);
        assert!(response.enabled);
    }

    #[test]
    fn iam_user_without_name_maps_to_none() {
        let iam_user = IamUser {
            id: IamUserId("iam-xyz".into()),
            email: "bob@example.com".into(),
            username: "bob".into(),
            name: None,
            email_verified: true,
            enabled: true,
        };

        let response: UserResponse = iam_user.into();
        assert!(response.name.is_none());
        assert!(response.email_verified);
    }

    #[test]
    fn create_user_maps_iam_conflict_to_409() {
        let err: ApiError = IamError::Conflict("email taken".into()).into();
        assert!(matches!(err, ApiError::Conflict(_)));
    }

    #[test]
    fn create_user_maps_iam_invalid_input_to_422() {
        let err: ApiError = IamError::InvalidInput("bad username".into()).into();
        assert!(matches!(err, ApiError::UnprocessableEntity(_)));
    }
}
