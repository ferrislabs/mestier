use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{MemberId, OrganizationId, RateBp, Surcharge, SurchargeId, WorkDay, WorkDayInput};

pub mod commands;
pub mod error;
pub mod ports;
pub mod service;

#[cfg(test)]
mod tests;

pub use commands::{ExtraPeriodDraft, ExtraPeriodFilter};
pub use error::{ExtraPeriodError, ExtraPeriodOutcome};
pub use service::ExtraPeriodService;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct ExtraPeriodId(pub Uuid);

impl FromStr for ExtraPeriodId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(ExtraPeriodId)
    }
}

impl Display for ExtraPeriodId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ExtraStatus {
    Pending,
    Approved,
    Rejected,
    Cancelled,
}

impl ExtraStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "PENDING",
            Self::Approved => "APPROVED",
            Self::Rejected => "REJECTED",
            Self::Cancelled => "CANCELLED",
        }
    }

    pub fn approve(self) -> ExtraPeriodOutcome<Self> {
        match self {
            Self::Pending => Ok(Self::Approved),
            _ => Err(ExtraPeriodError::AlreadyDecided),
        }
    }

    pub fn reject(self) -> ExtraPeriodOutcome<Self> {
        match self {
            Self::Pending => Ok(Self::Rejected),
            _ => Err(ExtraPeriodError::AlreadyDecided),
        }
    }

    pub fn cancel(self) -> ExtraPeriodOutcome<Self> {
        match self {
            Self::Pending | Self::Approved => Ok(Self::Cancelled),
            _ => Err(ExtraPeriodError::AlreadyDecided),
        }
    }
}

impl FromStr for ExtraStatus {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "PENDING" => Ok(Self::Pending),
            "APPROVED" => Ok(Self::Approved),
            "REJECTED" => Ok(Self::Rejected),
            "CANCELLED" => Ok(Self::Cancelled),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtraReason(String);

impl ExtraReason {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for ExtraReason {
    type Error = ExtraPeriodError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            Err(ExtraPeriodError::ReasonBlank)
        } else {
            Ok(Self(trimmed.to_owned()))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExtraPeriod {
    pub id: ExtraPeriodId,
    pub organization_id: OrganizationId,
    pub member_id: MemberId,
    pub date: NaiveDate,
    pub hours: WorkDay,
    pub reason: ExtraReason,
    pub surcharge_id: Option<SurchargeId>,
    pub rate: RateBp,
    pub status: ExtraStatus,
    pub created_by_member_id: Option<MemberId>,
    pub decided_by_member_id: Option<MemberId>,
    pub decided_at: Option<DateTime<Utc>>,
    pub decision_comment: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl ExtraPeriod {
    pub fn declare(
        id: ExtraPeriodId,
        organization_id: OrganizationId,
        draft: &ExtraPeriodDraft,
        surcharge: Option<&Surcharge>,
        creator: Option<MemberId>,
        now: DateTime<Utc>,
    ) -> ExtraPeriodOutcome<Self> {
        let hours = WorkDay::try_from(&WorkDayInput {
            start: draft.start.clone(),
            end: draft.end.clone(),
            lunch: draft.lunch.clone(),
        })?;
        let reason = ExtraReason::try_from(draft.reason.as_str())?;
        let rate = match surcharge {
            Some(surcharge) if surcharge.is_archived() => {
                return Err(ExtraPeriodError::SurchargeArchived);
            }
            Some(surcharge) => surcharge.rate,
            None => RateBp::try_from(draft.rate_bp.unwrap_or(0))?,
        };
        let mut period = Self {
            id,
            organization_id,
            member_id: draft.member_id,
            date: draft.date,
            hours,
            reason,
            surcharge_id: surcharge.map(|surcharge| surcharge.id),
            rate,
            status: ExtraStatus::Pending,
            created_by_member_id: creator,
            decided_by_member_id: None,
            decided_at: None,
            decision_comment: None,
            created_at: now,
            updated_at: now,
        };
        if draft.approve {
            period.approve(creator, now)?;
        }
        Ok(period)
    }

    pub fn approve(
        &mut self,
        decider: Option<MemberId>,
        at: DateTime<Utc>,
    ) -> ExtraPeriodOutcome<()> {
        let status = self.status.approve()?;
        self.decide(status, decider, at);
        Ok(())
    }

    pub fn reject(
        &mut self,
        comment: &str,
        decider: Option<MemberId>,
        at: DateTime<Utc>,
    ) -> ExtraPeriodOutcome<()> {
        let status = self.status.reject()?;
        let comment = comment.trim();
        if comment.is_empty() {
            return Err(ExtraPeriodError::CommentRequired);
        }
        self.decision_comment = Some(comment.to_owned());
        self.decide(status, decider, at);
        Ok(())
    }

    pub fn cancel(
        &mut self,
        decider: Option<MemberId>,
        at: DateTime<Utc>,
    ) -> ExtraPeriodOutcome<()> {
        let status = self.status.cancel()?;
        self.decide(status, decider, at);
        Ok(())
    }

    fn decide(&mut self, status: ExtraStatus, decider: Option<MemberId>, at: DateTime<Utc>) {
        self.status = status;
        self.decided_by_member_id = decider;
        self.decided_at = Some(at);
        self.updated_at = at;
    }
}
