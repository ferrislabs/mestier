use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{User, UserId};

#[derive(Debug, Clone)]
pub struct UserRow {
    pub id: Uuid,
    pub email: String,
    pub username: String,
    pub display_name: String,
    pub sub: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<UserRow> for User {
    fn from(row: UserRow) -> Self {
        Self {
            id: UserId(row.id),
            email: row.email,
            username: row.username,
            name: row.display_name,
            sub: row.sub,
            created_at: row.created_at,
            updated_at: row.updated_at,
        }
    }
}
