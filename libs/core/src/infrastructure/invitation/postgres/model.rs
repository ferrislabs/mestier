use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{
    UserId,
    domain::{
        invitation::{Invitation, InvitationId},
        member::MemberId,
        organization::OrganizationId,
    },
};

#[derive(Debug, Clone)]
pub struct InvitationRow {
    pub id: Uuid,
    pub organization_id: Uuid,
    pub member_id: Option<Uuid>,
    pub token_hash: Vec<u8>,
    pub expires_at: DateTime<Utc>,
    pub consumed_at: Option<DateTime<Utc>>,
    pub consumed_by_user_id: Option<Uuid>,
    pub created_by_user_id: Uuid,
    pub created_at: DateTime<Utc>,
}

impl From<InvitationRow> for Invitation {
    fn from(row: InvitationRow) -> Self {
        Self {
            id: InvitationId(row.id),
            organization_id: OrganizationId(row.organization_id),
            member_id: row.member_id.map(MemberId),
            token_hash: row.token_hash,
            expires_at: row.expires_at,
            consumed_at: row.consumed_at,
            consumed_by_user_id: row.consumed_by_user_id.map(UserId),
            created_by_user_id: UserId(row.created_by_user_id),
            created_at: row.created_at,
        }
    }
}
