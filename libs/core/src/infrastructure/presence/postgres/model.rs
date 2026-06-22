use std::str::FromStr;

use chrono::{DateTime, Utc};
use common::CoreError;
use discord::{OrganizationId, Presence, PresenceStatus, UserId};
use uuid::Uuid;

#[derive(Debug, Clone)]
pub struct PresenceRow {
    pub org_id: Uuid,
    pub user_id: Uuid,
    pub status: String,
    pub updated_at: DateTime<Utc>,
}

impl TryFrom<PresenceRow> for Presence {
    type Error = CoreError;

    fn try_from(r: PresenceRow) -> Result<Self, Self::Error> {
        let status = PresenceStatus::from_str(&r.status)
            .map_err(|e| CoreError::Internal(format!("invalid presence_status in db: {e}")))?;
        Ok(Self {
            organization_id: OrganizationId(r.org_id),
            user_id: UserId(r.user_id),
            status,
            updated_at: r.updated_at,
        })
    }
}
