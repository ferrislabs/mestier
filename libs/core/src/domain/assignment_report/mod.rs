use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{MemberId, OrganizationId, TaskAssignmentId};

pub mod commands;
pub mod events;
pub mod ports;
pub mod service;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct AssignmentReportId(pub Uuid);

impl FromStr for AssignmentReportId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(AssignmentReportId)
    }
}

impl Display for AssignmentReportId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The report's arbitration state.
///
/// Not a boolean "resolved" flag next to an "outcome": a report that has not
/// been looked at yet is a distinct state from one a manager decided against
/// applying, and callers (the field app's "pending, amendable" view, the
/// planning module's pending count) need to tell the two apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AssignmentReportResolution {
    Pending,
    Applied,
    Dismissed,
}

impl AssignmentReportResolution {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "PENDING",
            Self::Applied => "APPLIED",
            Self::Dismissed => "DISMISSED",
        }
    }
}

impl Display for AssignmentReportResolution {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for AssignmentReportResolution {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "PENDING" => Ok(Self::Pending),
            "APPLIED" => Ok(Self::Applied),
            "DISMISSED" => Ok(Self::Dismissed),
            other => Err(format!("invalid assignment report resolution `{other}`")),
        }
    }
}

/// One worker's word that a task's actual duration diverged from the plan —
/// see ADR 0002 and the migration comment on `assignment_reports`.
///
/// `resolved_by`/`resolved_at` are `None` exactly when `resolution` is
/// `Pending`, mirroring the database's own equivalence CHECK.
/// `resolution_note` stays independently optional: a manager may resolve
/// with or without leaving a word back to the worker, whichever the
/// resolution.
#[derive(Debug, Clone, PartialEq)]
pub struct AssignmentReport {
    pub id: AssignmentReportId,
    pub organization_id: OrganizationId,
    pub task_assignment_id: TaskAssignmentId,
    pub reported_minutes: u32,
    pub comment: Option<String>,
    pub reported_by: MemberId,
    pub resolution: AssignmentReportResolution,
    pub resolved_by: Option<MemberId>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub resolution_note: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl AssignmentReport {
    pub fn is_pending(&self) -> bool {
        self.resolution == AssignmentReportResolution::Pending
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assignment_report_id_parses_uuid() {
        let uuid = Uuid::new_v4();
        let parsed = AssignmentReportId::from_str(&uuid.to_string()).unwrap();

        assert_eq!(parsed.0, uuid);
    }

    #[test]
    fn assignment_report_id_rejects_invalid_uuid() {
        assert!(AssignmentReportId::from_str("not-a-uuid").is_err());
    }

    #[test]
    fn assignment_report_id_displays_as_its_uuid() {
        let uuid = Uuid::new_v4();
        let id = AssignmentReportId(uuid);

        assert_eq!(id.to_string(), uuid.to_string());
    }

    #[test]
    fn resolution_round_trips_through_its_string_form() {
        for resolution in [
            AssignmentReportResolution::Pending,
            AssignmentReportResolution::Applied,
            AssignmentReportResolution::Dismissed,
        ] {
            let parsed = AssignmentReportResolution::from_str(resolution.as_str()).unwrap();
            assert_eq!(parsed, resolution);
        }
    }

    #[test]
    fn resolution_rejects_an_unknown_string() {
        assert!(AssignmentReportResolution::from_str("SOMETHING_ELSE").is_err());
    }
}
