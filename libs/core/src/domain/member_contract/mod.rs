use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{MemberId, OrganizationId};

pub mod commands;
pub mod error;
pub mod ports;
pub mod schedule;
pub mod service;

#[cfg(test)]
mod tests;

pub use commands::MemberContractInput;
pub use error::{MemberContractError, MemberContractOutcome};
pub use schedule::{
    BlockInput, ClockTime, ContractSchedule, DayRefusal, LunchBreak, LunchInput, ScheduleBlock,
    ScheduleError, ScheduleInput, ScheduleRefusal, ScheduleUnit, WeekSchedule, WorkDay,
    WorkDayInput,
};
pub use service::MemberContractService;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct MemberContractId(pub Uuid);

impl FromStr for MemberContractId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(MemberContractId)
    }
}

impl Display for MemberContractId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ContractType {
    Cdi,
    Cdd,
    Apprenticeship,
    Interim,
    Manager,
}

impl ContractType {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Cdi => "CDI",
            Self::Cdd => "CDD",
            Self::Apprenticeship => "APPRENTICESHIP",
            Self::Interim => "INTERIM",
            Self::Manager => "MANAGER",
        }
    }

    pub fn requires_end_date(self) -> bool {
        matches!(self, Self::Cdd | Self::Apprenticeship | Self::Interim)
    }
}

impl FromStr for ContractType {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "CDI" => Ok(Self::Cdi),
            "CDD" => Ok(Self::Cdd),
            "APPRENTICESHIP" => Ok(Self::Apprenticeship),
            "INTERIM" => Ok(Self::Interim),
            "MANAGER" => Ok(Self::Manager),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ContractPeriod {
    start: NaiveDate,
    end: Option<NaiveDate>,
    trial_end: Option<NaiveDate>,
}

impl ContractPeriod {
    pub fn new(
        start: NaiveDate,
        end: Option<NaiveDate>,
        trial_end: Option<NaiveDate>,
    ) -> MemberContractOutcome<Self> {
        if end.is_some_and(|end| end < start) {
            return Err(MemberContractError::EndBeforeStart);
        }
        let trial_inside =
            trial_end.is_none_or(|trial| trial >= start && end.is_none_or(|end| trial <= end));
        if !trial_inside {
            return Err(MemberContractError::TrialOutsideContract);
        }
        Ok(Self {
            start,
            end,
            trial_end,
        })
    }

    pub fn start(&self) -> NaiveDate {
        self.start
    }

    pub fn end(&self) -> Option<NaiveDate> {
        self.end
    }

    pub fn trial_end(&self) -> Option<NaiveDate> {
        self.trial_end
    }

    pub fn overlaps(&self, other: &Self) -> bool {
        let starts_before_other_ends = other.end.is_none_or(|end| self.start <= end);
        let other_starts_before_end = self.end.is_none_or(|end| other.start <= end);
        starts_before_other_ends && other_starts_before_end
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemberContract {
    pub id: MemberContractId,
    pub organization_id: OrganizationId,
    pub member_id: MemberId,
    pub contract_type: ContractType,
    pub period: ContractPeriod,
    pub schedule: ContractSchedule,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

struct Validated {
    contract_type: ContractType,
    period: ContractPeriod,
    schedule: ContractSchedule,
}

fn validated(input: MemberContractInput) -> MemberContractOutcome<Validated> {
    if input.contract_type.requires_end_date() && input.end_date.is_none() {
        return Err(MemberContractError::FixedTermNeedsEndDate);
    }
    Ok(Validated {
        contract_type: input.contract_type,
        period: ContractPeriod::new(input.start_date, input.end_date, input.trial_end_date)?,
        schedule: ContractSchedule::try_from(input.schedule)?,
    })
}

impl MemberContract {
    pub fn draft(
        id: MemberContractId,
        organization_id: OrganizationId,
        member_id: MemberId,
        input: MemberContractInput,
        now: DateTime<Utc>,
    ) -> MemberContractOutcome<Self> {
        let Validated {
            contract_type,
            period,
            schedule,
        } = validated(input)?;
        Ok(Self {
            id,
            organization_id,
            member_id,
            contract_type,
            period,
            schedule,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn replaced_with(
        &self,
        input: MemberContractInput,
        now: DateTime<Utc>,
    ) -> MemberContractOutcome<Self> {
        let Validated {
            contract_type,
            period,
            schedule,
        } = validated(input)?;
        Ok(Self {
            contract_type,
            period,
            schedule,
            updated_at: now,
            ..self.clone()
        })
    }
}
