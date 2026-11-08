use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{AbsenceKind, OrganizationId};

pub mod commands;
pub mod error;
pub mod ports;
pub mod service;

#[cfg(test)]
mod tests;

pub use error::{AbsenceTypeError, AbsenceTypeOutcome};
pub use service::AbsenceTypeService;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct AbsenceTypeId(pub Uuid);

impl FromStr for AbsenceTypeId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(AbsenceTypeId)
    }
}

impl Display for AbsenceTypeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CountMode {
    Working,
    Workable,
    Calendar,
}

impl CountMode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Working => "WORKING",
            Self::Workable => "WORKABLE",
            Self::Calendar => "CALENDAR",
        }
    }
}

impl FromStr for CountMode {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "WORKING" => Ok(Self::Working),
            "WORKABLE" => Ok(Self::Workable),
            "CALENDAR" => Ok(Self::Calendar),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbsenceTypeName(String);

impl AbsenceTypeName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for AbsenceTypeName {
    type Error = AbsenceTypeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            Err(AbsenceTypeError::NameEmpty)
        } else {
            Ok(Self(trimmed.to_owned()))
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hue(u16);

impl Hue {
    pub fn get(self) -> u16 {
        self.0
    }
}

impl TryFrom<u16> for Hue {
    type Error = AbsenceTypeError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        if value > 360 {
            Err(AbsenceTypeError::HueOutOfRange)
        } else {
            Ok(Self(value))
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct AbsenceTypeInput<'a> {
    pub name: &'a str,
    pub hue: u16,
    pub count_mode: CountMode,
    pub account_id: Option<Uuid>,
    pub paid: bool,
    pub needs_approval: bool,
    pub needs_proof: bool,
    pub kind: AbsenceKind,
}

impl AbsenceTypeInput<'_> {
    pub fn validate(&self) -> AbsenceTypeOutcome<(AbsenceTypeName, Hue)> {
        Ok((
            AbsenceTypeName::try_from(self.name)?,
            Hue::try_from(self.hue)?,
        ))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AbsenceType {
    pub id: AbsenceTypeId,
    pub organization_id: OrganizationId,
    pub name: AbsenceTypeName,
    pub hue: Hue,
    pub count_mode: CountMode,
    pub account_id: Option<Uuid>,
    pub paid: bool,
    pub needs_approval: bool,
    pub needs_proof: bool,
    pub kind: AbsenceKind,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl AbsenceType {
    pub fn new(
        id: AbsenceTypeId,
        organization_id: OrganizationId,
        input: AbsenceTypeInput<'_>,
        created_at: DateTime<Utc>,
    ) -> AbsenceTypeOutcome<Self> {
        let (name, hue) = input.validate()?;
        Ok(Self {
            id,
            organization_id,
            name,
            hue,
            count_mode: input.count_mode,
            account_id: input.account_id,
            paid: input.paid,
            needs_approval: input.needs_approval,
            needs_proof: input.needs_proof,
            kind: input.kind,
            archived_at: None,
            created_at,
        })
    }

    pub fn revise(&mut self, input: AbsenceTypeInput<'_>) -> AbsenceTypeOutcome<()> {
        let (name, hue) = input.validate()?;
        self.name = name;
        self.hue = hue;
        self.count_mode = input.count_mode;
        self.account_id = input.account_id;
        self.paid = input.paid;
        self.needs_approval = input.needs_approval;
        self.needs_proof = input.needs_proof;
        self.kind = input.kind;
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

#[derive(Debug, Clone, Copy)]
pub struct DefaultAbsenceType {
    pub name: &'static str,
    pub account_name: Option<&'static str>,
    pub hue: u16,
    pub count_mode: CountMode,
    pub paid: bool,
    pub needs_approval: bool,
    pub needs_proof: bool,
    pub kind: AbsenceKind,
}

impl From<&DefaultAbsenceType> for AbsenceTypeInput<'static> {
    fn from(value: &DefaultAbsenceType) -> Self {
        Self {
            name: value.name,
            hue: value.hue,
            count_mode: value.count_mode,
            account_id: None,
            paid: value.paid,
            needs_approval: value.needs_approval,
            needs_proof: value.needs_proof,
            kind: value.kind,
        }
    }
}

const DEFAULT_ABSENCE_TYPES: [DefaultAbsenceType; 7] = [
    DefaultAbsenceType {
        name: "Congés payés",
        account_name: Some("Congés payés"),
        hue: 155,
        count_mode: CountMode::Working,
        paid: true,
        needs_approval: true,
        needs_proof: false,
        kind: AbsenceKind::Leave,
    },
    DefaultAbsenceType {
        name: "RTT",
        account_name: Some("RTT"),
        hue: 250,
        count_mode: CountMode::Working,
        paid: true,
        needs_approval: true,
        needs_proof: false,
        kind: AbsenceKind::Leave,
    },
    DefaultAbsenceType {
        name: "Arrêt maladie",
        account_name: None,
        hue: 25,
        count_mode: CountMode::Calendar,
        paid: true,
        needs_approval: false,
        needs_proof: true,
        kind: AbsenceKind::Sick,
    },
    DefaultAbsenceType {
        name: "Intempéries",
        account_name: None,
        hue: 215,
        count_mode: CountMode::Working,
        paid: true,
        needs_approval: false,
        needs_proof: false,
        kind: AbsenceKind::Unavailable,
    },
    DefaultAbsenceType {
        name: "Événement familial",
        account_name: None,
        hue: 320,
        count_mode: CountMode::Working,
        paid: true,
        needs_approval: true,
        needs_proof: true,
        kind: AbsenceKind::Leave,
    },
    DefaultAbsenceType {
        name: "Formation",
        account_name: None,
        hue: 190,
        count_mode: CountMode::Working,
        paid: true,
        needs_approval: true,
        needs_proof: false,
        kind: AbsenceKind::Unavailable,
    },
    DefaultAbsenceType {
        name: "Congé sans solde",
        account_name: None,
        hue: 60,
        count_mode: CountMode::Working,
        paid: false,
        needs_approval: true,
        needs_proof: false,
        kind: AbsenceKind::Leave,
    },
];

pub fn default_absence_types() -> &'static [DefaultAbsenceType] {
    &DEFAULT_ABSENCE_TYPES
}

pub(crate) fn staggered(base: DateTime<Utc>, index: usize) -> DateTime<Utc> {
    base + Duration::milliseconds(index as i64)
}
