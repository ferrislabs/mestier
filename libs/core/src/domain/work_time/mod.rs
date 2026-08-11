use std::fmt::Display;
use std::str::FromStr;

use chrono::{DateTime, NaiveDate, Utc};
use common::CoreError;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{EmployeeId, MemberId, OrganizationId};

pub mod commands;
pub mod ports;
pub mod service;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct EmployeeRhythmId(pub Uuid);

impl FromStr for EmployeeRhythmId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(EmployeeRhythmId)
    }
}

impl Display for EmployeeRhythmId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct RhythmSlotId(pub Uuid);

impl FromStr for RhythmSlotId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(RhythmSlotId)
    }
}

impl Display for RhythmSlotId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct WorkSlotId(pub Uuid);

impl FromStr for WorkSlotId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(WorkSlotId)
    }
}

impl Display for WorkSlotId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// One weekly-recurring slot of a rhythm version: `weekday` is ISO (Monday =
/// 1, Sunday = 7), `starts_minute`/`ends_minute` count minutes from midnight
/// in the organization's local time — mirrors `employee_rhythm_slots`
/// exactly, no `TIME`, no float.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RhythmSlot {
    pub id: RhythmSlotId,
    pub rhythm_id: EmployeeRhythmId,
    pub weekday: i16,
    pub starts_minute: i16,
    pub ends_minute: i16,
}

/// A recurring weekly rhythm, versioned by `effective_from`/`effective_to`.
/// `effective_to = None` means the version is currently open — the one in
/// effect until a later version closes it. An employee's history keeps every
/// past version rather than overwriting it, so planning and payroll stay
/// reconcilable after an organization changes hours.
#[derive(Debug, Clone, PartialEq)]
pub struct EmployeeRhythm {
    pub id: EmployeeRhythmId,
    pub organization_id: OrganizationId,
    pub employee_id: EmployeeId,
    pub effective_from: NaiveDate,
    pub effective_to: Option<NaiveDate>,
    pub slots: Vec<RhythmSlot>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A concrete work slot pinned to a real date. When one or more exist for a
/// given `work_date`, they take priority over whatever the rhythm says for
/// that weekday — see [`service::expand_work_slots`].
///
/// Keyed on the member, not the profile: someone with no contract still
/// declares the days they work, and that has to show up on the planning grid.
/// The rhythm is the opposite case — it is a contract translated into
/// recurring slots, so it stays keyed on [`EmployeeRhythm::employee_id`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WorkSlot {
    pub id: WorkSlotId,
    pub organization_id: OrganizationId,
    pub member_id: MemberId,
    pub work_date: NaiveDate,
    pub starts_minute: i16,
    pub ends_minute: i16,
}

/// A minute-of-day interval, the unit [`service::expand_work_slots`]
/// produces for each day it has data for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MinuteInterval {
    pub starts_minute: i16,
    pub ends_minute: i16,
}

/// An inclusive `[from, to]` window of calendar days — the unit both the
/// `work-time` read/write endpoints and `expand_work_slots` operate on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DateRange {
    pub from: NaiveDate,
    pub to: NaiveDate,
}

impl DateRange {
    pub fn new(from: NaiveDate, to: NaiveDate) -> Result<Self, CoreError> {
        if to < from {
            return Err(CoreError::Conflict(
                "date range `to` must not be before `from`".to_owned(),
            ));
        }

        Ok(Self { from, to })
    }

    /// Inclusive day count spanned by the range: a single-day range
    /// (`from == to`) spans 1 day.
    pub fn span_days(&self) -> i64 {
        (self.to - self.from).num_days() + 1
    }

    pub fn contains(&self, date: NaiveDate) -> bool {
        date >= self.from && date <= self.to
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn employee_rhythm_id_parses_uuid() {
        let uuid = Uuid::new_v4();
        let parsed = EmployeeRhythmId::from_str(&uuid.to_string()).unwrap();

        assert_eq!(parsed.0, uuid);
    }

    #[test]
    fn employee_rhythm_id_rejects_invalid_uuid() {
        assert!(EmployeeRhythmId::from_str("not-a-uuid").is_err());
    }

    #[test]
    fn employee_work_slot_id_parses_uuid() {
        let uuid = Uuid::new_v4();
        let parsed = WorkSlotId::from_str(&uuid.to_string()).unwrap();

        assert_eq!(parsed.0, uuid);
    }

    #[test]
    fn date_range_rejects_to_before_from() {
        let from = NaiveDate::from_ymd_opt(2026, 8, 10).unwrap();
        let to = NaiveDate::from_ymd_opt(2026, 8, 1).unwrap();

        let err = DateRange::new(from, to).unwrap_err();

        assert!(matches!(err, CoreError::Conflict(_)));
    }

    #[test]
    fn date_range_accepts_a_single_day() {
        let day = NaiveDate::from_ymd_opt(2026, 8, 10).unwrap();

        let range = DateRange::new(day, day).unwrap();

        assert_eq!(range.span_days(), 1);
    }

    #[test]
    fn date_range_span_days_counts_both_bounds_inclusive() {
        let from = NaiveDate::from_ymd_opt(2026, 1, 1).unwrap();
        let to = NaiveDate::from_ymd_opt(2026, 4, 2).unwrap();

        let range = DateRange::new(from, to).unwrap();

        assert_eq!(range.span_days(), 92);
    }

    #[test]
    fn date_range_contains_checks_both_bounds_inclusive() {
        let from = NaiveDate::from_ymd_opt(2026, 8, 1).unwrap();
        let to = NaiveDate::from_ymd_opt(2026, 8, 10).unwrap();
        let range = DateRange::new(from, to).unwrap();

        assert!(range.contains(from));
        assert!(range.contains(to));
        assert!(!range.contains(from - chrono::Duration::days(1)));
        assert!(!range.contains(to + chrono::Duration::days(1)));
    }
}
