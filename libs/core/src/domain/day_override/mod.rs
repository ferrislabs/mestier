use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{MemberId, OrganizationId, WorkDay, WorkDayInput, domain::member_contract::LunchBreak};

pub mod commands;
pub mod error;
pub mod feed;
pub mod ports;
pub mod service;

#[cfg(test)]
mod tests;

pub use commands::DayOverrideInput;
pub use error::{DayOverrideError, DayOverrideOutcome, PlanningPeriodError};
pub use feed::{FeedMember, HrPlanningFeed};
pub use service::DayOverrideService;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct DayOverrideId(pub Uuid);

impl FromStr for DayOverrideId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(DayOverrideId)
    }
}

impl Display for DayOverrideId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum OverrideKind {
    Work,
    Off,
}

impl OverrideKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Work => "WORK",
            Self::Off => "OFF",
        }
    }
}

impl FromStr for OverrideKind {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "WORK" => Ok(Self::Work),
            "OFF" => Ok(Self::Off),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PlanningPeriod {
    from: NaiveDate,
    to: NaiveDate,
}

impl PlanningPeriod {
    pub fn new(from: NaiveDate, to: NaiveDate, max_days: i64) -> Result<Self, PlanningPeriodError> {
        if to < from {
            return Err(PlanningPeriodError::Inverted);
        }
        if (to - from).num_days() + 1 > max_days {
            return Err(PlanningPeriodError::TooLong(max_days));
        }
        Ok(Self { from, to })
    }

    pub fn from(&self) -> NaiveDate {
        self.from
    }

    pub fn to(&self) -> NaiveDate {
        self.to
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DayShape {
    Work(WorkDay),
    Off,
}

impl DayShape {
    pub fn kind(&self) -> OverrideKind {
        match self {
            Self::Work(_) => OverrideKind::Work,
            Self::Off => OverrideKind::Off,
        }
    }

    pub fn work(&self) -> Option<&WorkDay> {
        match self {
            Self::Work(day) => Some(day),
            Self::Off => None,
        }
    }

    pub fn lunch(&self) -> Option<LunchBreak> {
        self.work().and_then(WorkDay::lunch)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DayOverride {
    pub id: DayOverrideId,
    pub organization_id: OrganizationId,
    pub member_id: MemberId,
    pub date: NaiveDate,
    pub shape: DayShape,
    pub note: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl DayOverride {
    pub fn declare(
        id: DayOverrideId,
        organization_id: OrganizationId,
        member_id: MemberId,
        date: NaiveDate,
        input: &DayOverrideInput,
        now: DateTime<Utc>,
    ) -> DayOverrideOutcome<Self> {
        let shape = match input.kind {
            OverrideKind::Off => {
                if input.start.is_some() || input.end.is_some() || input.lunch.is_some() {
                    return Err(DayOverrideError::OffHasHours);
                }
                DayShape::Off
            }
            OverrideKind::Work => {
                let start = input.start.clone().ok_or(DayOverrideError::StartMissing)?;
                let end = input.end.clone().ok_or(DayOverrideError::EndMissing)?;
                let hours = WorkDayInput {
                    start,
                    end,
                    lunch: input.lunch.clone(),
                };
                DayShape::Work(WorkDay::try_from(&hours)?)
            }
        };
        Ok(Self {
            id,
            organization_id,
            member_id,
            date,
            shape,
            note: input.note.trim().to_owned(),
            created_at: now,
            updated_at: now,
        })
    }
}
