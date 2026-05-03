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
    pub user_id: Uuid,
    pub joined_at: DateTime<Utc>,
}

impl From<MemberRow> for Member {
    fn from(row: MemberRow) -> Self {
        Self {
            id: MemberId(row.id),
            organization_id: OrganizationId(row.organization_id),
            user_id: UserId(row.user_id),
            joined_at: row.joined_at,
        }
    }
}
