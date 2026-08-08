use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{
    UserId,
    domain::{
        member::{Member, MemberId},
        organization::OrganizationId,
    },
};

#[derive(Debug, Clone)]
pub struct MemberRow {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub user_id: Option<Uuid>,
    pub last_name: String,
    pub first_name: Option<String>,
    pub joined_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

impl From<MemberRow> for Member {
    fn from(row: MemberRow) -> Self {
        Self {
            id: MemberId(row.id),
            organization_id: OrganizationId(row.organization_id),
            user_id: row.user_id.map(UserId),
            last_name: row.last_name,
            first_name: row.first_name,
            joined_at: row.joined_at,
            created_at: row.created_at,
            deleted_at: row.deleted_at,
        }
    }
}
