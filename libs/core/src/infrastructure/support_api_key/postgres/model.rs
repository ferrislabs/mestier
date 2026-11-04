use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{OrganizationId, SupportApiKey, SupportApiKeyId};

pub struct SupportApiKeyRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub name: String,
    pub prefix: String,
    pub secret_hash: Vec<u8>,
    pub created_at: DateTime<Utc>,
    pub last_used_at: Option<DateTime<Utc>>,
    pub revoked_at: Option<DateTime<Utc>>,
}

impl From<SupportApiKeyRow> for SupportApiKey {
    fn from(row: SupportApiKeyRow) -> Self {
        Self {
            id: SupportApiKeyId(row.id),
            organization_id: OrganizationId(row.org_id),
            name: row.name,
            prefix: row.prefix,
            secret_hash: row.secret_hash,
            created_at: row.created_at,
            last_used_at: row.last_used_at,
            revoked_at: row.revoked_at,
        }
    }
}
