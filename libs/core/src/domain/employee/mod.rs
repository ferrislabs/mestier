use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{OrganizationId, UserId, domain::member::format_person_name};

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
    pub last_name: String,
    /// `None` means "not provided" — distinct from an empty string, which
    /// would be a `NULL` in disguise. Unlike `Customer::first_name`, this is
    /// nullable: the `20260807000008_split_employee_name` migration backfills
    /// every existing employee's free-text `name` into `last_name` and cannot
    /// safely guess a first name from it (splitting on whitespace turns "Le
    /// Guen" into "Le"/"Guen"), so it leaves `first_name` unset rather than
    /// produce a wrong split that looks correct.
    pub first_name: Option<String>,
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

impl Employee {
    /// The one display format for an employee, everywhere: "{last_name}
    /// {first_name}". See
    /// [`format_person_name`](crate::domain::member::format_person_name),
    /// the single place this formatting lives — shared with [`Member`]'s
    /// own `display_name` so a person's name reads the same whether it
    /// comes from a standalone seat or the HR profile attached to one.
    ///
    /// [`Member`]: crate::domain::member::Member
    pub fn display_name(&self) -> String {
        format_person_name(&self.last_name, self.first_name.as_deref())
    }
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

    #[test]
    fn employee_display_name_delegates_to_format_person_name() {
        let now = Utc::now();
        let employee = Employee {
            id: EmployeeId(Uuid::new_v4()),
            organization_id: OrganizationId(Uuid::new_v4()),
            user_id: None,
            last_name: "Parmantier".to_owned(),
            first_name: Some("Baptiste".to_owned()),
            hourly_rate_cents: None,
            weekly_contract_minutes: 0,
            deleted_at: None,
            created_at: now,
            updated_at: now,
        };

        assert_eq!(employee.display_name(), "Parmantier Baptiste");
    }
}
