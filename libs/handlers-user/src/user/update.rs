use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use iam::{IamProvider, IamUpdateUser, IamUserId};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::UserPath, require_known_user, response::UserResponse};

#[derive(Debug, Deserialize, ToSchema)]
pub struct UpdateUserRequest {
    pub email: Option<String>,
    pub username: Option<String>,
    pub name: Option<String>,
    pub enabled: Option<bool>,
}

#[utoipa::path(
	patch,
	path = "/api/v1/users/{id}",
	operation_id = "updateUser",
	tag = super::super::TAG,
	params(
		("id" = String, Path, description = "IAM user identifier"),
	),
	request_body = UpdateUserRequest,
	responses(
		(status = 200, description = "User updated", body = inline(DataEnvelope<UserResponse>)),
		(status = 400, description = "Bad request"),
		(status = 401, description = "Unauthorized"),
		(status = 403, description = "Forbidden"),
		(status = 404, description = "User not found"),
		(status = 409, description = "Conflict"),
		(status = 422, description = "Validation failed"),
		(status = 500, description = "IAM unavailable"),
	),
	security(("bearer_auth" = []))
)]
pub async fn handler(
    UserPath { id }: UserPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<UpdateUserRequest>,
) -> Result<Response<UserResponse>, ApiError> {
    require_known_user(&state, &identity).await?;

    let iam_user = state
        .iam
        .update_user(
            &IamUserId(id),
            IamUpdateUser {
                email: payload.email,
                username: payload.username,
                name: payload.name,
                enabled: payload.enabled,
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::OK(iam_user.into()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use iam::{IamError, IamUser, IamUserId};

    #[test]
    fn iam_user_maps_to_user_response_with_email_verified() {
        let iam_user = IamUser {
            id: IamUserId("iam-abc".into()),
            email: "new@example.com".into(),
            username: "alice".into(),
            name: None,
            email_verified: true,
            enabled: false,
        };

        let response: UserResponse = iam_user.into();
        assert_eq!(response.email, "new@example.com");
        assert!(response.email_verified);
        assert!(!response.enabled);
    }

    #[test]
    fn update_user_maps_not_found_to_404() {
        let err: ApiError = IamError::NotFound.into();
        assert!(matches!(err, ApiError::NotFound));
    }

    #[test]
    fn update_user_maps_conflict_to_409() {
        let err: ApiError = IamError::Conflict("username taken".into()).into();
        assert!(matches!(err, ApiError::Conflict(_)));
    }

    #[test]
    fn update_user_maps_unauthorized_to_external_service() {
        let err: ApiError = IamError::Unauthorized.into();
        assert!(matches!(err, ApiError::ExternalService(_)));
    }
}
