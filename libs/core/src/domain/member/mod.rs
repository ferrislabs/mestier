use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{UserId, domain::organization::OrganizationId};

pub mod commands;
pub mod ports;
pub mod service;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, ToSchema)]
pub struct MemberId(pub Uuid);

impl FromStr for MemberId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(MemberId)
    }
}

impl Display for MemberId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct Member {
    pub id: MemberId,
    pub organization_id: OrganizationId,
    pub user_id: UserId,
    pub joined_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn member_id_parses_uuid() {
        let uuid = Uuid::new_v4();
        let parsed = MemberId::from_str(&uuid.to_string()).unwrap();

        assert_eq!(parsed.0, uuid);
    }

    #[test]
    fn member_id_rejects_invalid_uuid() {
        assert!(MemberId::from_str("not-a-uuid").is_err());
    }
}
