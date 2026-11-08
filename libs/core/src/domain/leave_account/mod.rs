use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::OrganizationId;

pub mod commands;
pub mod defaults;
pub mod error;
pub mod ports;
pub mod service;

#[cfg(test)]
mod tests;

pub use defaults::{DefaultLeaveAccount, default_leave_accounts};
pub use error::{LeaveAccountError, LeaveAccountOutcome};
pub use service::LeaveAccountService;

const MAX_HUNDREDTHS: i64 = 100_000_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct LeaveAccountId(pub Uuid);

impl FromStr for LeaveAccountId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(LeaveAccountId)
    }
}

impl Display for LeaveAccountId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct LeaveDays(i32);

impl LeaveDays {
    pub const ZERO: Self = Self(0);

    pub fn from_hundredths(hundredths: i32) -> Option<Self> {
        (i64::from(hundredths).abs() <= MAX_HUNDREDTHS).then_some(Self(hundredths))
    }

    pub fn from_decimal(value: f64) -> Option<Self> {
        if !value.is_finite() {
            return None;
        }
        let scaled = value * 100.0;
        let rounded = scaled.round();
        if (scaled - rounded).abs() > 1e-6 || rounded.abs() > MAX_HUNDREDTHS as f64 {
            return None;
        }
        Some(Self(rounded as i32))
    }

    pub fn hundredths(self) -> i32 {
        self.0
    }

    pub fn as_decimal(self) -> f64 {
        f64::from(self.0) / 100.0
    }

    pub fn is_zero(self) -> bool {
        self.0 == 0
    }

    pub fn is_negative(self) -> bool {
        self.0 < 0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PositiveDays(LeaveDays);

impl PositiveDays {
    pub fn days(self) -> LeaveDays {
        self.0
    }

    pub fn from_hundredths(hundredths: i32) -> Option<Self> {
        LeaveDays::from_hundredths(hundredths)
            .filter(|days| days.hundredths() > 0)
            .map(Self)
    }
}

impl TryFrom<f64> for PositiveDays {
    type Error = LeaveAccountError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        LeaveDays::from_decimal(value)
            .filter(|days| days.hundredths() > 0)
            .map(Self)
            .ok_or(LeaveAccountError::AcquisitionInvalid)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CarryOverMax(LeaveDays);

impl CarryOverMax {
    pub fn days(self) -> LeaveDays {
        self.0
    }

    pub fn from_hundredths(hundredths: i32) -> Option<Self> {
        LeaveDays::from_hundredths(hundredths)
            .filter(|days| days.hundredths() >= 0)
            .map(Self)
    }
}

impl TryFrom<f64> for CarryOverMax {
    type Error = LeaveAccountError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        LeaveDays::from_decimal(value)
            .filter(|days| days.hundredths() >= 0)
            .map(Self)
            .ok_or(LeaveAccountError::CarryOverMaxInvalid)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeaveAccountName(String);

impl LeaveAccountName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for LeaveAccountName {
    type Error = LeaveAccountError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            Err(LeaveAccountError::NameEmpty)
        } else {
            Ok(Self(trimmed.to_owned()))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PeriodStartMonth(u8);

impl PeriodStartMonth {
    pub fn get(self) -> u8 {
        self.0
    }
}

impl TryFrom<u8> for PeriodStartMonth {
    type Error = LeaveAccountError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if (1..=12).contains(&value) {
            Ok(Self(value))
        } else {
            Err(LeaveAccountError::PeriodStartMonthOutOfRange)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Acquisition {
    Monthly(PositiveDays),
    Annual(PositiveDays),
}

impl Acquisition {
    pub fn kind_str(self) -> &'static str {
        match self {
            Self::Monthly(_) => "MONTHLY",
            Self::Annual(_) => "ANNUAL",
        }
    }

    pub fn amount(self) -> PositiveDays {
        match self {
            Self::Monthly(days) | Self::Annual(days) => days,
        }
    }

    pub fn from_parts(kind: &str, amount: PositiveDays) -> Option<Self> {
        match kind {
            "MONTHLY" => Some(Self::Monthly(amount)),
            "ANNUAL" => Some(Self::Annual(amount)),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum AcquisitionDraft {
    Monthly { per_month: f64 },
    Annual { days: f64 },
}

impl TryFrom<AcquisitionDraft> for Acquisition {
    type Error = LeaveAccountError;

    fn try_from(value: AcquisitionDraft) -> Result<Self, Self::Error> {
        match value {
            AcquisitionDraft::Monthly { per_month } => {
                PositiveDays::try_from(per_month).map(Self::Monthly)
            }
            AcquisitionDraft::Annual { days } => PositiveDays::try_from(days).map(Self::Annual),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct LeaveAccountInput<'a> {
    pub name: &'a str,
    pub acquisition: AcquisitionDraft,
    pub period_start_month: u8,
    pub carry_over_max: f64,
}

type ValidatedAccount = (
    LeaveAccountName,
    Acquisition,
    PeriodStartMonth,
    CarryOverMax,
);

impl LeaveAccountInput<'_> {
    pub fn validate(&self) -> LeaveAccountOutcome<ValidatedAccount> {
        Ok((
            LeaveAccountName::try_from(self.name)?,
            Acquisition::try_from(self.acquisition)?,
            PeriodStartMonth::try_from(self.period_start_month)?,
            CarryOverMax::try_from(self.carry_over_max)?,
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeaveAccount {
    pub id: LeaveAccountId,
    pub organization_id: OrganizationId,
    pub name: LeaveAccountName,
    pub acquisition: Acquisition,
    pub period_start_month: PeriodStartMonth,
    pub carry_over_max: CarryOverMax,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl LeaveAccount {
    pub fn new(
        id: LeaveAccountId,
        organization_id: OrganizationId,
        input: LeaveAccountInput<'_>,
        created_at: DateTime<Utc>,
    ) -> LeaveAccountOutcome<Self> {
        let (name, acquisition, period_start_month, carry_over_max) = input.validate()?;
        Ok(Self {
            id,
            organization_id,
            name,
            acquisition,
            period_start_month,
            carry_over_max,
            archived_at: None,
            created_at,
        })
    }

    pub fn revise(&mut self, input: LeaveAccountInput<'_>) -> LeaveAccountOutcome<()> {
        let (name, acquisition, period_start_month, carry_over_max) = input.validate()?;
        self.name = name;
        self.acquisition = acquisition;
        self.period_start_month = period_start_month;
        self.carry_over_max = carry_over_max;
        Ok(())
    }

    pub fn archive(&mut self, at: DateTime<Utc>) {
        self.archived_at.get_or_insert(at);
    }

    pub fn restore(&mut self) {
        self.archived_at = None;
    }

    pub fn is_archived(&self) -> bool {
        self.archived_at.is_some()
    }
}
