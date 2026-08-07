use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{EmployeeId, OrganizationId, WorkOrderId};

pub mod commands;
pub mod ports;
pub mod service;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct TimeEntryId(pub Uuid);

impl FromStr for TimeEntryId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(TimeEntryId)
    }
}

impl Display for TimeEntryId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Phase for site photos attached to a clocking session.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TimeEntryPhotoPhase {
    Before,
    During,
    After,
}

impl TimeEntryPhotoPhase {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Before => "BEFORE",
            Self::During => "DURING",
            Self::After => "AFTER",
        }
    }
}

impl Display for TimeEntryPhotoPhase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for TimeEntryPhotoPhase {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "BEFORE" => Ok(Self::Before),
            "DURING" => Ok(Self::During),
            "AFTER" => Ok(Self::After),
            other => Err(format!("invalid time entry photo phase `{other}`")),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimeEntry {
    pub id: TimeEntryId,
    pub organization_id: OrganizationId,
    pub work_order_id: WorkOrderId,
    pub employee_id: EmployeeId,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    /// Opaque file keys from `POST /api/v1/files`.
    pub photos_before: Vec<String>,
    pub photos_during: Vec<String>,
    pub photos_after: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TimeEntry {
    pub fn is_open(&self) -> bool {
        self.ended_at.is_none()
    }

    pub fn photos_mut(&mut self, phase: TimeEntryPhotoPhase) -> &mut Vec<String> {
        match phase {
            TimeEntryPhotoPhase::Before => &mut self.photos_before,
            TimeEntryPhotoPhase::During => &mut self.photos_during,
            TimeEntryPhotoPhase::After => &mut self.photos_after,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn time_entry_id_parses_uuid() {
        let uuid = Uuid::new_v4();
        let parsed = TimeEntryId::from_str(&uuid.to_string()).unwrap();

        assert_eq!(parsed.0, uuid);
    }

    #[test]
    fn time_entry_photo_phase_round_trips() {
        for phase in [
            TimeEntryPhotoPhase::Before,
            TimeEntryPhotoPhase::During,
            TimeEntryPhotoPhase::After,
        ] {
            assert_eq!(
                phase.as_str().parse::<TimeEntryPhotoPhase>().unwrap(),
                phase
            );
        }
    }
}
