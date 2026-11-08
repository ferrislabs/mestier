use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, NaiveDate, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{AbsenceType, AbsenceTypeId, FileKey, MemberId, OrganizationId};

pub mod commands;
pub mod error;
pub mod ports;
pub mod service;
pub mod span;
pub mod status;

#[cfg(test)]
mod tests;

pub use error::{AbsenceError, AbsenceOutcome};
pub use span::{AbsenceSpan, DayPart};
pub use status::AbsenceStatus;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct AbsenceId(pub Uuid);

impl FromStr for AbsenceId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(AbsenceId)
    }
}

impl Display for AbsenceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// Congé, arrêt ou indisponibilité: the same business fact — the person is
/// not available on this slot — with a different qualification. One column,
/// not one table per kind (see the planning module design doc).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AbsenceKind {
    Leave,
    Sick,
    Unavailable,
}

impl AbsenceKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Leave => "LEAVE",
            Self::Sick => "SICK",
            Self::Unavailable => "UNAVAILABLE",
        }
    }
}

impl Display for AbsenceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl FromStr for AbsenceKind {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "LEAVE" => Ok(Self::Leave),
            "SICK" => Ok(Self::Sick),
            "UNAVAILABLE" => Ok(Self::Unavailable),
            other => Err(format!("invalid absence kind `{other}`")),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Absence {
    pub id: AbsenceId,
    pub organization_id: OrganizationId,
    pub member_id: MemberId,
    pub kind: AbsenceKind,
    pub type_id: AbsenceTypeId,
    pub starts_at: DateTime<Utc>,
    pub ends_at: DateTime<Utc>,
    pub all_day: bool,
    pub start_date: NaiveDate,
    pub start_part: DayPart,
    pub end_date: NaiveDate,
    pub end_part: DayPart,
    pub status: AbsenceStatus,
    pub note: Option<String>,
    pub proof_file_key: Option<String>,
    pub created_by_member_id: Option<MemberId>,
    pub decided_by_member_id: Option<MemberId>,
    pub decided_at: Option<DateTime<Utc>>,
    pub decision_comment: Option<String>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

pub(crate) fn clean_note(note: Option<&str>) -> AbsenceOutcome<Option<String>> {
    match note {
        Some(value) if value.trim().is_empty() => Err(AbsenceError::NoteBlank),
        other => Ok(other.map(str::to_owned)),
    }
}

pub(crate) fn clean_proof(proof: Option<&str>) -> AbsenceOutcome<Option<String>> {
    proof
        .map(|key| {
            FileKey::try_from(key)
                .map(|valid| valid.as_str().to_owned())
                .map_err(|_| AbsenceError::ProofKeyInvalid)
        })
        .transpose()
}

impl Absence {
    #[allow(clippy::too_many_arguments)]
    pub fn request(
        id: AbsenceId,
        organization_id: OrganizationId,
        member_id: MemberId,
        absence_type: &AbsenceType,
        span: AbsenceSpan,
        window: (DateTime<Utc>, DateTime<Utc>),
        needs_approval: bool,
        note: Option<&str>,
        proof_file_key: Option<&str>,
        created_by_member_id: Option<MemberId>,
        now: DateTime<Utc>,
    ) -> AbsenceOutcome<Self> {
        if absence_type.is_archived() {
            return Err(AbsenceError::TypeArchived);
        }
        let note = clean_note(note)?;
        let proof_file_key = clean_proof(proof_file_key)?;
        Ok(Self {
            id,
            organization_id,
            member_id,
            kind: absence_type.kind,
            type_id: absence_type.id,
            starts_at: window.0,
            ends_at: window.1,
            all_day: span.is_full_days(),
            start_date: span.start_date(),
            start_part: span.start_part(),
            end_date: span.end_date(),
            end_part: span.end_part(),
            status: if needs_approval || (absence_type.needs_proof && proof_file_key.is_none()) {
                AbsenceStatus::Pending
            } else {
                AbsenceStatus::Approved
            },
            note,
            proof_file_key,
            created_by_member_id,
            decided_by_member_id: None,
            decided_at: None,
            decision_comment: None,
            deleted_at: None,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn approve(
        &mut self,
        absence_type: &AbsenceType,
        decider: Option<MemberId>,
        at: DateTime<Utc>,
    ) -> AbsenceOutcome<()> {
        let status = self.status.approve()?;
        if absence_type.needs_proof && self.proof_file_key.is_none() {
            return Err(AbsenceError::ProofRequired);
        }
        self.decide(status, decider, at);
        Ok(())
    }

    pub fn reject(
        &mut self,
        comment: &str,
        decider: Option<MemberId>,
        at: DateTime<Utc>,
    ) -> AbsenceOutcome<()> {
        let status = self.status.reject()?;
        let comment = comment.trim();
        if comment.is_empty() {
            return Err(AbsenceError::CommentRequired);
        }
        self.decision_comment = Some(comment.to_owned());
        self.decide(status, decider, at);
        Ok(())
    }

    pub fn cancel(&mut self, decider: Option<MemberId>, at: DateTime<Utc>) -> AbsenceOutcome<()> {
        let status = self.status.cancel()?;
        self.decide(status, decider, at);
        Ok(())
    }

    fn decide(&mut self, status: AbsenceStatus, decider: Option<MemberId>, at: DateTime<Utc>) {
        self.status = status;
        self.decided_by_member_id = decider;
        self.decided_at = Some(at);
        self.updated_at = at;
    }
}
