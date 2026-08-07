use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{EmployeeId, OrganizationId};

pub mod commands;
pub mod ports;
pub mod service;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct EmployeeAbsenceId(pub Uuid);

impl FromStr for EmployeeAbsenceId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(EmployeeAbsenceId)
    }
}

impl Display for EmployeeAbsenceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Congé, arrêt ou indisponibilité: the same business fact — the person is
/// not available on this slot — with a different qualification. One column,
/// not one table per kind (see the planning module design doc).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AbsenceKind {
    Leave,
    Sick,
    Unavailable,
}

impl AbsenceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Leave => "LEAVE",
            Self::Sick => "SICK",
            Self::Unavailable => "UNAVAILABLE",
        }
    }
}

impl Display for AbsenceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for AbsenceKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "LEAVE" => Ok(Self::Leave),
            "SICK" => Ok(Self::Sick),
            "UNAVAILABLE" => Ok(Self::Unavailable),
            other => Err(format!("invalid absence kind `{other}`")),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct EmployeeAbsence {
    pub id: EmployeeAbsenceId,
    pub organization_id: OrganizationId,
    pub employee_id: EmployeeId,
    pub kind: AbsenceKind,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub all_day: bool,
    pub note: Option<String>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn employee_absence_id_parses_uuid() {
        let uuid = Uuid::new_v4();
        let parsed = EmployeeAbsenceId::from_str(&uuid.to_string()).unwrap();

        assert_eq!(parsed.0, uuid);
    }

    #[test]
    fn employee_absence_id_rejects_invalid_uuid() {
        assert!(EmployeeAbsenceId::from_str("not-a-uuid").is_err());
    }

    #[test]
    fn absence_kind_parses_known_values() {
        assert_eq!("LEAVE".parse::<AbsenceKind>().unwrap(), AbsenceKind::Leave);
        assert_eq!("SICK".parse::<AbsenceKind>().unwrap(), AbsenceKind::Sick);
        assert_eq!(
            "UNAVAILABLE".parse::<AbsenceKind>().unwrap(),
            AbsenceKind::Unavailable
        );
    }

    #[test]
    fn absence_kind_rejects_unknown_values() {
        assert!("HOLIDAY".parse::<AbsenceKind>().is_err());
    }

    #[test]
    fn absence_kind_round_trips_through_as_str() {
        for kind in [
            AbsenceKind::Leave,
            AbsenceKind::Sick,
            AbsenceKind::Unavailable,
        ] {
            assert_eq!(kind.as_str().parse::<AbsenceKind>().unwrap(), kind);
        }
    }
}
