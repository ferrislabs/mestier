use chrono::{DateTime, Utc};
use mestier_core::{Organization, OrganizationId, User, UserId};
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct OrganizationResponse {
    pub id: OrganizationId,
    pub name: String,
    pub slug: String,
    pub owner_id: UserId,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Organization> for OrganizationResponse {
    fn from(org: Organization) -> Self {
        Self {
            id: org.id,
            name: org.name,
            slug: org.slug,
            owner_id: org.owner_id,
            created_at: org.created_at,
            updated_at: org.updated_at,
        }
    }
}

/// Lightweight user representation returned by admin user endpoints.
#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct UserResponse {
    pub id: UserId,
    pub email: String,
    pub username: String,
    pub name: String,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            email: user.email,
            username: user.username,
            name: user.name,
        }
    }
}
