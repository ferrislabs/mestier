use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, Utc};
use serde::Serialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{OrganizationId, UserId};

pub mod commands;
pub mod ports;
pub mod service;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, ToSchema)]
pub struct EmployeeId(pub Uuid);

impl FromStr for EmployeeId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(EmployeeId)
    }
}

impl Display for EmployeeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Employee {
    pub id: EmployeeId,
    pub organization_id: OrganizationId,
    pub user_id: Option<UserId>,
    pub name: String,
    pub hourly_rate_cents: i32,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn employee_id_parses_uuid() {
        let uuid = Uuid::new_v4();
        let parsed = EmployeeId::from_str(&uuid.to_string()).unwrap();

        assert_eq!(parsed.0, uuid);
    }

    #[test]
    fn employee_id_rejects_invalid_uuid() {
        assert!(EmployeeId::from_str("not-a-uuid").is_err());
    }
}
