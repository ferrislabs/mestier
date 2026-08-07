use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, Response};
use iam::{IamProvider, IamUpdateUser, IamUserId};

use crate::{EmptyResponse, paths::UserPath, require_known_user};

#[utoipa::path(
	delete,
	path = "/api/v1/users/{id}",
	operation_id = "disableUser",
	tag = super::super::TAG,
	params(
		("id" = String, Path, description = "IAM user identifier"),
	),
	responses(
		(status = 204, description = "User disabled in IAM"),
		(status = 401, description = "Unauthorized"),
		(status = 403, description = "Forbidden"),
		(status = 404, description = "User not found"),
		(status = 500, description = "IAM unavailable"),
	),
	security(("bearer_auth" = []))
)]
pub async fn handler(
    UserPath { id }: UserPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<EmptyResponse>, ApiError> {
    require_known_user(&state, &identity).await?;

    // Disable in IAM; the webhook handler (#50) will reconcile the local copy.
    state
        .iam
        .update_user(
            &IamUserId(id),
            IamUpdateUser {
                enabled: Some(false),
                ..Default::default()
            },
        )
        .await
        .map_err(ApiError::from)?;

    Ok(Response::NoContent)
}

#[cfg(test)]
mod tests {
    use super::*;
    use iam::IamError;

    #[test]
    fn disable_maps_not_found_to_404() {
        let err: ApiError = IamError::NotFound.into();
        assert!(matches!(err, ApiError::NotFound));
    }

    #[test]
    fn disable_maps_unavailable_to_external_service() {
        let err: ApiError = IamError::Unavailable("connection refused".into()).into();
        assert!(matches!(err, ApiError::ExternalService(_)));
    }

    #[test]
    fn disable_maps_forbidden_to_external_service() {
        let err: ApiError = IamError::Forbidden.into();
        assert!(matches!(err, ApiError::ExternalService(_)));
    }

    #[test]
    fn iam_update_default_has_enabled_none() {
        let cmd = IamUpdateUser::default();
        assert!(cmd.enabled.is_none());
    }

    #[test]
    fn disable_command_sets_enabled_false() {
        let cmd = IamUpdateUser {
            enabled: Some(false),
            ..Default::default()
        };
        assert_eq!(cmd.enabled, Some(false));
        assert!(cmd.email.is_none());
        assert!(cmd.username.is_none());
        assert!(cmd.name.is_none());
    }
}
