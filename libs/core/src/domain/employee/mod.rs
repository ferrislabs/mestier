use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{MemberId, OrganizationId};

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

/// The contractual profile of a member — nothing else.
///
/// It used to be the profile of a *user*, carrying the person's name and a
/// `user_id`. That made someone plannable only once they had an HR record, and
/// forced the planning read model to reconcile two rosters. Since a member is a
/// seat that exists on its own, the person — their name, their account — lives
/// there, and what remains here is only what a contract says: a rate and a
/// weekly base.
///
/// A member may have no profile at all: they are still plannable, they simply
/// have no cost. That is why this is a separate row rather than columns on the
/// seat.
#[derive(Debug, Clone, PartialEq)]
pub struct Employee {
    pub id: EmployeeId,
    pub organization_id: OrganizationId,
    /// The seat this profile describes. At most one active profile per member,
    /// enforced by `uq_employees_member_active`; the composite foreign key on
    /// `(member_id, org_id)` makes a cross-organization profile impossible.
    pub member_id: MemberId,
    /// `None` means the rate is not set yet; `Some(0)` means genuinely free.
    ///
    /// The distinction matters: a cost computation must refuse to produce a
    /// figure for someone whose rate was never entered, rather than silently
    /// sum it as zero.
    pub hourly_rate_cents: Option<i32>,
    /// True when this person is not costed by the hour at all. Their clocked
    /// time still counts — profitability needs to know they worked — but at
    /// zero labour cost, and it never blocks a margin the way a genuinely
    /// missing rate does: the two are different kinds of absence.
    pub is_salaried: bool,
    /// Contractual weekly base. Deliberately not derived from the sum of the
    /// member's work slots — the gap between the two is the information.
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
