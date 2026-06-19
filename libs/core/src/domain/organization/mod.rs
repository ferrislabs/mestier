use crate::UserId;
use chrono::{DateTime, Utc};
pub use common::OrganizationId;

pub mod commands;
pub mod ports;
pub mod service;

#[derive(Debug, Clone)]
pub struct Organization {
    pub id: OrganizationId,
    pub name: String,
    pub slug: String,
    pub owner_id: UserId,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str::FromStr;
    use uuid::Uuid;

    #[test]
    fn organization_id_parses_uuid() {
        let uuid = Uuid::new_v4();
        let parsed = OrganizationId::from_str(&uuid.to_string()).unwrap();

        assert_eq!(parsed.0, uuid);
    }

    #[test]
    fn organization_id_rejects_invalid_uuid() {
        assert!(OrganizationId::from_str("not-a-uuid").is_err());
    }
}
