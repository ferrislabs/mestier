use iam::IamUser;
use mestier_core::User;
use serde::Serialize;
use utoipa::ToSchema;

/// DTO returned by all user management endpoints.
///
/// Built from [`IamUser`] for write endpoints or from the local copy for the
/// list endpoint. The IAM `sub` / `IamUserId` is the stable external identifier.
#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct UserResponse {
    pub id: String,
    pub email: String,
    pub username: String,
    pub name: Option<String>,
    pub email_verified: bool,
    pub enabled: bool,
}

impl From<IamUser> for UserResponse {
    fn from(u: IamUser) -> Self {
        Self {
            id: u.id.0,
            email: u.email,
            username: u.username,
            name: u.name,
            email_verified: u.email_verified,
            enabled: u.enabled,
        }
    }
}

/// Build a [`UserResponse`] from a local [`User`] copy.
///
/// `email_verified` is not stored locally (IAM is the source of truth).
/// `enabled` is always `true` here because `list_active()` only returns rows
/// where `deleted_at IS NULL`.
pub fn user_to_response(u: User) -> UserResponse {
    UserResponse {
        id: u.sub,
        email: u.email,
        username: u.username,
        name: if u.name.is_empty() {
            None
        } else {
            Some(u.name)
        },
        email_verified: false,
        enabled: true,
    }
}
