//! Maintenance contracts: a customer's site kept up for one season, the
//! lines of work done there (mowing, hedge trimming…) and the dated
//! interventions on each line.
//!
//! The next due date of a line is never stored. `due::due_for` computes it
//! from the cadence, the season and the validated interventions, so the grid
//! and the weekly recap cannot disagree (ADR 0006).
//!
//! Spec: `docs/chantiers/entretien.md`.

use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, Datelike, NaiveDate, Utc, Weekday};
use common::CoreError;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{CustomerContextId, CustomerId, MemberId, OrganizationId, ProjectId, QuoteId, TaskId};

pub mod commands;
pub mod contract_service;
pub mod due;
pub mod events;
pub mod overview;
pub mod plan;
pub mod ports;
pub mod visits;

macro_rules! uuid_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
        pub struct $name(pub Uuid);

        impl FromStr for $name {
            type Err = uuid::Error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                Uuid::from_str(s).map($name)
            }
        }

        impl Display for $name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                write!(f, "{}", self.0)
            }
        }
    };
}

uuid_id!(MaintenanceContractId);
uuid_id!(MaintenanceLineId);
uuid_id!(InterventionId);

/// An ISO week number, 1 to 53.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, ToSchema,
)]
#[serde(try_from = "u8", into = "u8")]
pub struct IsoWeekNumber(u8);

impl IsoWeekNumber {
    pub fn get(self) -> u8 {
        self.0
    }
}

impl TryFrom<u8> for IsoWeekNumber {
    type Error = CoreError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if !(1..=53).contains(&value) {
            return Err(CoreError::Conflict("week must be 1 to 53".to_owned()));
        }
        Ok(Self(value))
    }
}

impl From<IsoWeekNumber> for u8 {
    fn from(value: IsoWeekNumber) -> Self {
        value.0
    }
}

/// The year a contract covers and the weeks work happens in, S12 to S48 on
/// the paper sheet. No due date exists outside it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(try_from = "SeasonRaw", into = "SeasonRaw")]
pub struct Season {
    year: i32,
    first_week: IsoWeekNumber,
    last_week: IsoWeekNumber,
}

#[derive(Serialize, Deserialize)]
struct SeasonRaw {
    year: i32,
    first_week: IsoWeekNumber,
    last_week: IsoWeekNumber,
}

impl Season {
    pub const MIN_YEAR: i32 = 2000;
    pub const MAX_YEAR: i32 = 2100;

    pub fn new(
        year: i32,
        first_week: IsoWeekNumber,
        last_week: IsoWeekNumber,
    ) -> Result<Self, CoreError> {
        if !(Self::MIN_YEAR..=Self::MAX_YEAR).contains(&year) {
            return Err(CoreError::Conflict(format!(
                "season year must be {} to {}",
                Self::MIN_YEAR,
                Self::MAX_YEAR
            )));
        }
        if first_week > last_week {
            return Err(CoreError::Conflict(
                "season first week must not be after its last week".to_owned(),
            ));
        }
        if NaiveDate::from_isoywd_opt(year, u32::from(last_week.get()), Weekday::Sun).is_none() {
            return Err(CoreError::Conflict(format!(
                "{year} has no week {}",
                last_week.get()
            )));
        }
        Ok(Self {
            year,
            first_week,
            last_week,
        })
    }

    pub fn year(self) -> i32 {
        self.year
    }

    pub fn first_week(self) -> IsoWeekNumber {
        self.first_week
    }

    pub fn last_week(self) -> IsoWeekNumber {
        self.last_week
    }

    /// Monday of the first week.
    pub fn first_day(self) -> NaiveDate {
        self.monday_of(self.first_week)
    }

    /// Sunday of the last week.
    pub fn last_day(self) -> NaiveDate {
        NaiveDate::from_isoywd_opt(self.year, u32::from(self.last_week.get()), Weekday::Sun)
            .expect("checked by Season::new")
    }

    /// Monday of `week` in this season's ISO year. Week 53 of a year that
    /// has none falls back to the season's last day, which `new` rules out
    /// for the season's own bounds.
    pub fn monday_of(self, week: IsoWeekNumber) -> NaiveDate {
        NaiveDate::from_isoywd_opt(self.year, u32::from(week.get()), Weekday::Mon)
            .unwrap_or_else(|| self.last_day())
    }

    pub fn contains(self, date: NaiveDate) -> bool {
        (self.first_day()..=self.last_day()).contains(&date)
    }

    /// The same weeks one year on, for duplicating a contract. Week 53
    /// becomes 52 when the next year has no week 53.
    pub fn next(self) -> Result<Self, CoreError> {
        let year = self.year + 1;
        let clamp = |week: IsoWeekNumber| {
            if NaiveDate::from_isoywd_opt(year, u32::from(week.get()), Weekday::Mon).is_some() {
                week
            } else {
                IsoWeekNumber(52)
            }
        };
        Self::new(year, clamp(self.first_week), clamp(self.last_week))
    }
}

impl TryFrom<SeasonRaw> for Season {
    type Error = CoreError;

    fn try_from(raw: SeasonRaw) -> Result<Self, Self::Error> {
        Self::new(raw.year, raw.first_week, raw.last_week)
    }
}

impl From<Season> for SeasonRaw {
    fn from(season: Season) -> Self {
        Self {
            year: season.year,
            first_week: season.first_week,
            last_week: season.last_week,
        }
    }
}

/// Days between two interventions of a rolling cadence, 1 to 365.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(try_from = "u16", into = "u16")]
pub struct IntervalDays(u16);

impl IntervalDays {
    pub const MAX: u16 = 365;

    pub fn get(self) -> u16 {
        self.0
    }
}

impl TryFrom<u16> for IntervalDays {
    type Error = CoreError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        if value == 0 || value > Self::MAX {
            return Err(CoreError::Conflict(format!(
                "interval must be 1 to {} days",
                Self::MAX
            )));
        }
        Ok(Self(value))
    }
}

impl From<IntervalDays> for u16 {
    fn from(value: IntervalDays) -> Self {
        value.0
    }
}

/// A span of weeks in which one intervention is expected: "taille de
/// printemps, S14 à S17".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(try_from = "WindowRaw", into = "WindowRaw")]
pub struct Window {
    from_week: IsoWeekNumber,
    to_week: IsoWeekNumber,
}

#[derive(Serialize, Deserialize)]
struct WindowRaw {
    from_week: IsoWeekNumber,
    to_week: IsoWeekNumber,
}

impl Window {
    pub fn new(from_week: IsoWeekNumber, to_week: IsoWeekNumber) -> Result<Self, CoreError> {
        if from_week > to_week {
            return Err(CoreError::Conflict(
                "a window must not end before it starts".to_owned(),
            ));
        }
        Ok(Self { from_week, to_week })
    }

    pub fn from_week(self) -> IsoWeekNumber {
        self.from_week
    }

    pub fn to_week(self) -> IsoWeekNumber {
        self.to_week
    }
}

impl TryFrom<WindowRaw> for Window {
    type Error = CoreError;

    fn try_from(raw: WindowRaw) -> Result<Self, Self::Error> {
        Self::new(raw.from_week, raw.to_week)
    }
}

impl From<Window> for WindowRaw {
    fn from(window: Window) -> Self {
        Self {
            from_week: window.from_week,
            to_week: window.to_week,
        }
    }
}

/// 1 to 12 windows, in order, none overlapping the next.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(try_from = "Vec<Window>", into = "Vec<Window>")]
pub struct Windows(Vec<Window>);

impl Windows {
    pub const MAX: usize = 12;

    pub fn as_slice(&self) -> &[Window] {
        &self.0
    }
}

impl TryFrom<Vec<Window>> for Windows {
    type Error = CoreError;

    fn try_from(mut windows: Vec<Window>) -> Result<Self, Self::Error> {
        if windows.is_empty() || windows.len() > Self::MAX {
            return Err(CoreError::Conflict(format!(
                "a cadence needs 1 to {} windows",
                Self::MAX
            )));
        }
        windows.sort_by_key(|window| window.from_week);
        if windows
            .windows(2)
            .any(|pair| pair[0].to_week >= pair[1].from_week)
        {
            return Err(CoreError::Conflict("windows must not overlap".to_owned()));
        }
        Ok(Self(windows))
    }
}

impl From<Windows> for Vec<Window> {
    fn from(value: Windows) -> Self {
        value.0
    }
}

/// The rule that gives a line its next due date.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(tag = "kind", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Cadence {
    /// Due `every_days` after the last validated intervention: mowed on the
    /// 10th, due on the 24th.
    Rolling { every_days: IntervalDays },
    /// Due once in each window, at its first Monday.
    Windows { windows: Windows },
    /// Done when asked, never due ("ND" on the paper sheet).
    OnDemand,
}

impl Cadence {
    /// Windows outside the season would never come due.
    pub fn check_fits(&self, season: Season) -> Result<(), CoreError> {
        let Self::Windows { windows } = self else {
            return Ok(());
        };
        let fits = windows.as_slice().iter().all(|window| {
            window.from_week >= season.first_week && window.to_week <= season.last_week
        });
        if !fits {
            return Err(CoreError::Conflict(
                "every window must lie within the season".to_owned(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ContractStatus {
    /// Created by an import, not yet reviewed. Never due.
    Draft,
    Active,
}

impl ContractStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Draft => "DRAFT",
            Self::Active => "ACTIVE",
        }
    }
}

impl FromStr for ContractStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "DRAFT" => Ok(Self::Draft),
            "ACTIVE" => Ok(Self::Active),
            other => Err(format!("invalid maintenance contract status `{other}`")),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct MaintenanceLine {
    pub id: MaintenanceLineId,
    pub label: String,
    pub cadence: Cadence,
    pub minutes_per_intervention: Option<u32>,
}

/// The contract and its lines form one aggregate: the form saves them
/// together, in order.
#[derive(Debug, Clone, PartialEq)]
pub struct MaintenanceContract {
    pub id: MaintenanceContractId,
    pub organization_id: OrganizationId,
    pub customer_id: CustomerId,
    pub customer_context_id: CustomerContextId,
    pub project_id: ProjectId,
    /// The "code affaire", C25029 on the paper sheet.
    pub reference: Option<String>,
    pub season: Season,
    pub manager_member_id: Option<MemberId>,
    pub amount_cents: Option<i64>,
    pub quote_id: Option<QuoteId>,
    pub notes: Option<String>,
    pub status: ContractStatus,
    pub lines: Vec<MaintenanceLine>,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum InterventionStatus {
    /// Entered by the field, waiting for the manager.
    Proposed,
    Validated,
    Rejected,
}

impl InterventionStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Proposed => "PROPOSED",
            Self::Validated => "VALIDATED",
            Self::Rejected => "REJECTED",
        }
    }
}

impl FromStr for InterventionStatus {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "PROPOSED" => Ok(Self::Proposed),
            "VALIDATED" => Ok(Self::Validated),
            "REJECTED" => Ok(Self::Rejected),
            other => Err(format!("invalid intervention status `{other}`")),
        }
    }
}

/// A dated visit on a line. Only a `Validated` one moves the line's due
/// date. `resolved_by`/`resolved_at` are `None` exactly while `Proposed`,
/// mirroring the database CHECK.
#[derive(Debug, Clone, PartialEq)]
pub struct Intervention {
    pub id: InterventionId,
    pub organization_id: OrganizationId,
    pub line_id: MaintenanceLineId,
    pub done_on: NaiveDate,
    pub status: InterventionStatus,
    pub task_id: Option<TaskId>,
    pub proposed_by: MemberId,
    pub resolved_by: Option<MemberId>,
    pub resolved_at: Option<DateTime<Utc>>,
    pub note: Option<String>,
    pub created_at: DateTime<Utc>,
}

/// An intervention date must fall in the contract's season and not in the
/// future.
pub fn check_intervention_date(
    season: Season,
    done_on: NaiveDate,
    today: NaiveDate,
) -> Result<(), CoreError> {
    if done_on > today {
        return Err(CoreError::Conflict(
            "an intervention cannot be dated in the future".to_owned(),
        ));
    }
    if !season.contains(done_on) {
        return Err(CoreError::Conflict(format!(
            "an intervention must be dated within the {} season",
            season.year
        )));
    }
    Ok(())
}

/// The ISO week of `date`, as the grid and the recap number them.
pub fn iso_week_of(date: NaiveDate) -> (i32, IsoWeekNumber) {
    let week = date.iso_week();
    (week.year(), IsoWeekNumber(week.week() as u8))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn week(n: u8) -> IsoWeekNumber {
        IsoWeekNumber::try_from(n).unwrap()
    }

    fn window(from: u8, to: u8) -> Window {
        Window::new(week(from), week(to)).unwrap()
    }

    fn date(y: i32, m: u32, d: u32) -> NaiveDate {
        NaiveDate::from_ymd_opt(y, m, d).unwrap()
    }

    #[test]
    fn week_zero_and_fifty_four_are_rejected() {
        assert!(IsoWeekNumber::try_from(0).is_err());
        assert!(IsoWeekNumber::try_from(54).is_err());
    }

    #[test]
    fn a_season_runs_from_the_monday_of_its_first_week_to_the_sunday_of_its_last() {
        let season = Season::new(2026, week(12), week(48)).unwrap();

        assert_eq!(season.first_day(), date(2026, 3, 16));
        assert_eq!(season.last_day(), date(2026, 11, 29));
    }

    #[test]
    fn a_season_cannot_end_in_a_week_its_year_lacks() {
        assert!(Season::new(2026, week(12), week(53)).is_ok());
        assert!(Season::new(2027, week(12), week(53)).is_err());
    }

    #[test]
    fn a_season_cannot_end_before_it_starts() {
        assert!(Season::new(2026, week(30), week(12)).is_err());
    }

    #[test]
    fn the_next_season_clamps_week_fifty_three() {
        let next = Season::new(2026, week(12), week(53))
            .unwrap()
            .next()
            .unwrap();

        assert_eq!((next.year(), next.last_week()), (2027, week(52)));
    }

    #[test]
    fn interval_is_one_to_three_hundred_sixty_five_days() {
        assert!(IntervalDays::try_from(0).is_err());
        assert!(IntervalDays::try_from(1).is_ok());
        assert!(IntervalDays::try_from(365).is_ok());
        assert!(IntervalDays::try_from(366).is_err());
    }

    #[test]
    fn windows_are_sorted_and_must_not_overlap() {
        let sorted = Windows::try_from(vec![window(27, 30), window(14, 17)]).unwrap();
        assert_eq!(sorted.as_slice()[0], window(14, 17));

        assert!(Windows::try_from(vec![window(14, 17), window(17, 20)]).is_err());
        assert!(Windows::try_from(vec![]).is_err());
        assert!(Windows::try_from(vec![window(1, 1); 13]).is_err());
    }

    #[test]
    fn a_window_cannot_end_before_it_starts() {
        assert!(Window::new(week(17), week(14)).is_err());
    }

    #[test]
    fn windows_must_lie_within_the_season() {
        let season = Season::new(2026, week(12), week(48)).unwrap();
        let inside = Cadence::Windows {
            windows: Windows::try_from(vec![window(14, 17)]).unwrap(),
        };
        let outside = Cadence::Windows {
            windows: Windows::try_from(vec![window(46, 50)]).unwrap(),
        };

        assert!(inside.check_fits(season).is_ok());
        assert!(outside.check_fits(season).is_err());
    }

    #[test]
    fn a_cadence_round_trips_through_json() {
        let cadence = Cadence::Rolling {
            every_days: IntervalDays::try_from(14).unwrap(),
        };
        let json = serde_json::to_value(&cadence).unwrap();

        assert_eq!(
            json,
            serde_json::json!({ "kind": "ROLLING", "every_days": 14 })
        );
        assert_eq!(serde_json::from_value::<Cadence>(json).unwrap(), cadence);
    }

    #[test]
    fn an_invalid_cadence_is_refused_when_read() {
        let json = serde_json::json!({ "kind": "ROLLING", "every_days": 0 });

        assert!(serde_json::from_value::<Cadence>(json).is_err());
    }

    #[test]
    fn an_intervention_is_dated_in_the_season_and_not_in_the_future() {
        let season = Season::new(2026, week(12), week(48)).unwrap();
        let today = date(2026, 10, 11);

        assert!(check_intervention_date(season, date(2026, 10, 10), today).is_ok());
        assert!(check_intervention_date(season, date(2026, 10, 12), today).is_err());
        assert!(check_intervention_date(season, date(2026, 3, 1), today).is_err());
    }
}
