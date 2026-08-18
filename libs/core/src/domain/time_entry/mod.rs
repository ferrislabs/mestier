//! Field clocking: the time an employee actually spent on a task.
//!
//! Deliberately separate from `work_time`, which describes contractual
//! capacity. That one answers "when is this person supposed to be available";
//! this one answers "what did they work on, and for how long". M6 costs a job
//! from the second, never the first.

use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{EmployeeId, OrganizationId, TaskId};

pub mod commands;
pub mod events;
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct DayLogId(pub Uuid);

impl FromStr for DayLogId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(DayLogId)
    }
}

impl Display for DayLogId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct TimeEntryPhotoId(pub Uuid);

impl FromStr for TimeEntryPhotoId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(TimeEntryPhotoId)
    }
}

impl Display for TimeEntryPhotoId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// When a photo was taken relative to the work done in this session.
///
/// The phase is the reason these photos exist: a before/after pair is what
/// shows a customer what was done. Stored, never inferred from a timestamp.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum TimeEntryPhotoPhase {
    Before,
    During,
    After,
}

impl TimeEntryPhotoPhase {
    /// Every variant, for exhaustive iteration and round-trip tests.
    pub const ALL: [TimeEntryPhotoPhase; 3] = [Self::Before, Self::During, Self::After];

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
        Self::ALL
            .into_iter()
            .find(|phase| phase.as_str() == s)
            .ok_or_else(|| format!("invalid time entry photo phase `{s}`"))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct TimeEntryPhoto {
    pub id: TimeEntryPhotoId,
    pub organization_id: OrganizationId,
    pub time_entry_id: TimeEntryId,
    pub phase: TimeEntryPhotoPhase,
    pub storage_key: String,
    pub created_at: DateTime<Utc>,
}

/// One stretch of work on one task by one employee.
///
/// `ended_at` is `None` while the work is in progress. At most one entry per
/// employee may be in that state, which the database enforces with a partial
/// unique index; the service refuses first so the caller gets a usable error
/// instead of a constraint violation.
#[derive(Debug, Clone, PartialEq)]
pub struct TimeEntry {
    pub id: TimeEntryId,
    pub organization_id: OrganizationId,
    pub task_id: TaskId,
    pub employee_id: EmployeeId,
    pub started_at: DateTime<Utc>,
    pub ended_at: Option<DateTime<Utc>>,
    pub photos: Vec<TimeEntryPhoto>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl TimeEntry {
    pub fn is_running(&self) -> bool {
        self.ended_at.is_none()
    }

    /// Minutes worked, or `None` while the entry is still running.
    ///
    /// Truncating rather than rounding: a cost built from these must never
    /// exceed the time actually recorded.
    pub fn worked_minutes(&self) -> Option<i64> {
        self.ended_at
            .map(|ended_at| (ended_at - self.started_at).num_minutes())
    }
}

/// The employee's declaration that their working day is over.
///
/// One per employee per day. `work_date` is the calendar day in the
/// organization's timezone, not in UTC: an entry closed at 23:30 in Paris
/// belongs to that day, and would land on the next one if read as UTC.
#[derive(Debug, Clone, PartialEq)]
pub struct DayLog {
    pub id: DayLogId,
    pub organization_id: OrganizationId,
    pub employee_id: EmployeeId,
    pub work_date: NaiveDate,
    pub ended_at: DateTime<Utc>,
    pub created_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_phase_round_trips_through_its_wire_form() {
        for phase in TimeEntryPhotoPhase::ALL {
            assert_eq!(phase.as_str().parse::<TimeEntryPhotoPhase>(), Ok(phase));
        }
    }

    /// `as_str` is hand-written and `Serialize` is derived from `rename_all`.
    /// Nothing makes the two agree, so a mismatch would put one spelling in
    /// the database and another in the JSON.
    #[test]
    fn the_serialized_form_is_the_stored_form() {
        for phase in TimeEntryPhotoPhase::ALL {
            assert_eq!(
                serde_json::to_value(phase).expect("a phase serializes"),
                serde_json::Value::String(phase.as_str().to_owned()),
            );
        }
    }

    #[test]
    fn a_phase_outside_the_enum_is_refused() {
        assert!("AFTERWARDS".parse::<TimeEntryPhotoPhase>().is_err());
    }
}
