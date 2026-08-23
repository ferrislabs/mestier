//! A rule that repeats a task, and the horizon up to which it has already
//! been turned into real [`crate::Task`] rows.
//!
//! Nothing in `domain::task` used to repeat: a weekly meeting had to be
//! retyped every week, or its person-hours left the accounts. This module
//! models the rule (explicit columns per frequency — see
//! [`RecurrenceRule`] and its own doc for why an RRULE string was rejected)
//! and the pure arithmetic that turns it into calendar dates
//! ([`service::expand_occurrences`]). Materializing those dates into real
//! `tasks` rows, and moving `horizon_filled_to` forward, is
//! [`service::TaskRecurrenceService`]'s job.
//!
//! Every consumer of a task — `domain::planning`, `domain::profitability`,
//! conflict detection, worked hours — keeps reading ordinary `tasks` rows
//! and needs no change: that is the whole point of materializing instead of
//! expanding the rule at read time.

use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, NaiveDate, NaiveTime, Utc, Weekday};
use chrono_tz::Tz;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{CustomerContextId, CustomerId, MemberId, OrganizationId, ProjectId};

pub mod commands;
pub mod ports;
pub mod service;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct TaskRecurrenceId(pub Uuid);

impl FromStr for TaskRecurrenceId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(TaskRecurrenceId)
    }
}

impl Display for TaskRecurrenceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RecurrenceFrequency {
    Daily,
    Weekly,
    Monthly,
}

impl RecurrenceFrequency {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Daily => "DAILY",
            Self::Weekly => "WEEKLY",
            Self::Monthly => "MONTHLY",
        }
    }
}

impl Display for RecurrenceFrequency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for RecurrenceFrequency {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "DAILY" => Ok(Self::Daily),
            "WEEKLY" => Ok(Self::Weekly),
            "MONTHLY" => Ok(Self::Monthly),
            other => Err(format!("invalid recurrence frequency `{other}`")),
        }
    }
}

/// The rule a recurrence follows: explicit columns per frequency, not an
/// RRULE string.
///
/// RRULE-as-a-TEXT-column was the first shape considered, and rejected: it
/// can hold `"FREQ=NOPE;BYDAY=8"` just as easily as a valid rule, and every
/// reader (the expansion function, the API, the form) would have to parse
/// and re-validate a grammar before trusting it. Making invalid states
/// unrepresentable is the house rule (see `CLAUDE.md`); an enum-shaped rule
/// with the fields each variant actually needs (a weekday set for weekly, a
/// day-of-month for monthly) cannot express "every 8th day of the week" in
/// the first place, so there is nothing left to validate at read time. The
/// trade only favors RRULE once the product needs its full generality —
/// COUNT, BYSETPOS, several BYDAY-with-ordinal in one rule — and an
/// artisan's recurring meeting or maintenance round never does: daily,
/// weekly on a set of weekdays, or monthly on a day number covers the trade.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecurrenceRule {
    Daily,
    /// Never empty — a weekly recurrence with no weekday would produce
    /// nothing, which is not "weekly", it is "never". Enforced by
    /// [`commands::CreateTaskRecurrenceCommand`] validation and mirrored by
    /// `chk_task_recurrences_weekly_days`.
    Weekly {
        weekdays: Vec<Weekday>,
    },
    /// 1..=31. A month shorter than this clamps to its own last day (see
    /// [`service::expand_occurrences`]) rather than skipping the month.
    Monthly {
        day_of_month: u8,
    },
}

impl RecurrenceRule {
    pub fn frequency(&self) -> RecurrenceFrequency {
        match self {
            Self::Daily => RecurrenceFrequency::Daily,
            Self::Weekly { .. } => RecurrenceFrequency::Weekly,
            Self::Monthly { .. } => RecurrenceFrequency::Monthly,
        }
    }
}

/// ISO weekday numbering (1 = Monday .. 7 = Sunday) — the convention
/// `weekly_weekdays` is stored and exchanged in, both over the wire (#294's
/// API) and in the database (the infra adapter). Kept as free functions
/// rather than relying on `chrono::Weekday`'s own `u8` conversion: that one
/// is zero-based on Monday (`num_days_from_monday`), a different, easily
/// transposed convention, and a wire format is worth being explicit about
/// rather than trusting every caller remembers the offset.
pub fn weekday_from_iso(n: i16) -> Result<Weekday, String> {
    match n {
        1 => Ok(Weekday::Mon),
        2 => Ok(Weekday::Tue),
        3 => Ok(Weekday::Wed),
        4 => Ok(Weekday::Thu),
        5 => Ok(Weekday::Fri),
        6 => Ok(Weekday::Sat),
        7 => Ok(Weekday::Sun),
        other => Err(format!("invalid ISO weekday `{other}`, expected 1..=7")),
    }
}

pub fn weekday_to_iso(weekday: Weekday) -> i16 {
    match weekday {
        Weekday::Mon => 1,
        Weekday::Tue => 2,
        Weekday::Wed => 3,
        Weekday::Thu => 4,
        Weekday::Fri => 5,
        Weekday::Sat => 6,
        Weekday::Sun => 7,
    }
}

/// A recurring task's rule, its template, and how far it has already been
/// materialized.
///
/// The template (`title`, `description`, `start_time`/`duration_minutes`,
/// `blocks_availability`, `customer_id`/`customer_context_id`, `project_id`,
/// `assignee_member_ids`) is copied onto every occurrence's own `tasks` row
/// at materialization time. Editing the recurrence changes the template for
/// occurrences materialized afterwards; it never reaches back into ones
/// already created — those are ordinary tasks now, edited or deleted like
/// any other (see `TaskService::patch_task`'s detach-on-edit rule).
#[derive(Debug, Clone, PartialEq)]
pub struct TaskRecurrence {
    pub id: TaskRecurrenceId,
    pub organization_id: OrganizationId,
    pub rule: RecurrenceRule,
    pub starts_on: NaiveDate,
    pub ends_on: Option<NaiveDate>,
    /// Every occurrence up to and including this date already has a
    /// materialized `tasks` row. Moved forward in the same transaction as
    /// the rows it accounts for (see
    /// `service::TaskRecurrenceService::materialize_up_to`), so it is never
    /// ahead of what was actually persisted.
    pub horizon_filled_to: NaiveDate,
    /// The IANA zone `start_time` is interpreted in. "Every Tuesday at 9am"
    /// is a wall-clock claim: this is what keeps 9am local across a DST
    /// change, the same reason `domain::planning::Tz` exists.
    pub timezone: Tz,
    pub start_time: NaiveTime,
    pub duration_minutes: i32,
    pub all_day: bool,
    pub title: String,
    pub description: Option<String>,
    pub blocks_availability: bool,
    pub customer_id: Option<CustomerId>,
    pub customer_context_id: Option<CustomerContextId>,
    pub project_id: Option<ProjectId>,
    /// The complete assignee set applied to every materialized occurrence —
    /// same "always the full list, never a delta" contract as
    /// `PatchTaskCommand::assignees`.
    pub assignee_member_ids: Vec<MemberId>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn task_recurrence_id_parses_uuid() {
        let uuid = Uuid::new_v4();
        let parsed = TaskRecurrenceId::from_str(&uuid.to_string()).unwrap();

        assert_eq!(parsed.0, uuid);
    }

    #[test]
    fn task_recurrence_id_rejects_invalid_uuid() {
        assert!(TaskRecurrenceId::from_str("not-a-uuid").is_err());
    }

    #[test]
    fn recurrence_frequency_round_trips_through_its_string_form() {
        for frequency in [
            RecurrenceFrequency::Daily,
            RecurrenceFrequency::Weekly,
            RecurrenceFrequency::Monthly,
        ] {
            assert_eq!(
                frequency.as_str().parse::<RecurrenceFrequency>().unwrap(),
                frequency
            );
        }
    }

    #[test]
    fn recurrence_frequency_rejects_unknown_values() {
        assert!("YEARLY".parse::<RecurrenceFrequency>().is_err());
    }

    #[test]
    fn iso_weekday_round_trips_for_every_day() {
        for weekday in [
            Weekday::Mon,
            Weekday::Tue,
            Weekday::Wed,
            Weekday::Thu,
            Weekday::Fri,
            Weekday::Sat,
            Weekday::Sun,
        ] {
            let iso = weekday_to_iso(weekday);
            assert!((1..=7).contains(&iso));
            assert_eq!(weekday_from_iso(iso).unwrap(), weekday);
        }
    }

    #[test]
    fn iso_weekday_rejects_out_of_range_values() {
        assert!(weekday_from_iso(0).is_err());
        assert!(weekday_from_iso(8).is_err());
    }

    #[test]
    fn rule_frequency_matches_its_own_variant() {
        assert_eq!(
            RecurrenceRule::Daily.frequency(),
            RecurrenceFrequency::Daily
        );
        assert_eq!(
            RecurrenceRule::Weekly {
                weekdays: vec![Weekday::Tue]
            }
            .frequency(),
            RecurrenceFrequency::Weekly
        );
        assert_eq!(
            RecurrenceRule::Monthly { day_of_month: 1 }.frequency(),
            RecurrenceFrequency::Monthly
        );
    }
}
