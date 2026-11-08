use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{LeaveAccount, LeaveAccountId, LeaveDays, MemberId, OrganizationId};

pub mod commands;
pub mod error;
pub mod ports;
pub mod service;

#[cfg(test)]
mod tests;

pub use commands::LeaveAdjustmentDraft;
pub use error::{LeaveAdjustmentError, LeaveAdjustmentOutcome};
pub use service::LeaveAdjustmentService;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct LeaveAdjustmentId(pub Uuid);

impl FromStr for LeaveAdjustmentId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(LeaveAdjustmentId)
    }
}

impl Display for LeaveAdjustmentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AdjustmentKind {
    Adjustment,
    CarryOver,
}

impl AdjustmentKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Adjustment => "ADJUSTMENT",
            Self::CarryOver => "CARRY_OVER",
        }
    }
}

impl FromStr for AdjustmentKind {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "ADJUSTMENT" => Ok(Self::Adjustment),
            "CARRY_OVER" => Ok(Self::CarryOver),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AdjustmentDays(LeaveDays);

impl AdjustmentDays {
    pub fn days(self) -> LeaveDays {
        self.0
    }

    pub fn from_hundredths(hundredths: i32) -> Option<Self> {
        LeaveDays::from_hundredths(hundredths)
            .filter(|days| !days.is_zero())
            .map(Self)
    }
}

impl TryFrom<f64> for AdjustmentDays {
    type Error = LeaveAdjustmentError;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        LeaveDays::from_decimal(value)
            .filter(|days| !days.is_zero())
            .map(Self)
            .ok_or(LeaveAdjustmentError::DaysInvalid)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AdjustmentReason(String);

impl AdjustmentReason {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for AdjustmentReason {
    type Error = LeaveAdjustmentError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            Err(LeaveAdjustmentError::ReasonEmpty)
        } else {
            Ok(Self(trimmed.to_owned()))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LeaveAdjustment {
    pub id: LeaveAdjustmentId,
    pub organization_id: OrganizationId,
    pub member_id: MemberId,
    pub account_id: LeaveAccountId,
    pub date: NaiveDate,
    pub days: AdjustmentDays,
    pub kind: AdjustmentKind,
    pub reason: AdjustmentReason,
    pub author_member_id: Option<MemberId>,
    pub created_at: DateTime<Utc>,
}

impl LeaveAdjustment {
    pub fn record(
        id: LeaveAdjustmentId,
        organization_id: OrganizationId,
        draft: &LeaveAdjustmentDraft,
        account: &LeaveAccount,
        author_member_id: Option<MemberId>,
        created_at: DateTime<Utc>,
    ) -> LeaveAdjustmentOutcome<Self> {
        if account.is_archived() {
            return Err(LeaveAdjustmentError::AccountArchived);
        }
        let days = AdjustmentDays::try_from(draft.days)?;
        let reason = AdjustmentReason::try_from(draft.reason.as_str())?;
        if draft.kind == AdjustmentKind::CarryOver {
            if days.days().is_negative() {
                return Err(LeaveAdjustmentError::CarryOverNegative);
            }
            if days.days() > account.carry_over_max.days() {
                return Err(LeaveAdjustmentError::CarryOverTooLarge);
            }
        }
        Ok(Self {
            id,
            organization_id,
            member_id: draft.member_id,
            account_id: draft.account_id,
            date: draft.date,
            days,
            kind: draft.kind,
            reason,
            author_member_id,
            created_at,
        })
    }
}
