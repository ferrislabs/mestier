use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{OrganizationId, UserId};

pub mod commands;
pub mod ports;
pub mod service;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
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
    /// `None` means the rate is not set yet; `Some(0)` means genuinely free.
    ///
    /// The distinction matters: an employee record created on the fly while
    /// assigning a work order has no rate, and a cost computation must refuse
    /// to produce a figure rather than silently sum it as zero.
    pub hourly_rate_cents: Option<i32>,
    /// Contractual weekly base. Deliberately not derived from the sum of the
    /// employee's work slots — the gap between the two is the information.
    pub weekly_contract_minutes: i32,
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
