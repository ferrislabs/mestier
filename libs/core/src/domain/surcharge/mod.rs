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

pub use defaults::{DefaultSurcharge, default_surcharges};
pub use error::{SurchargeError, SurchargeOutcome};
pub use service::SurchargeService;

pub const MAX_RATE_BP: i32 = 20_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct SurchargeId(pub Uuid);

impl FromStr for SurchargeId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(SurchargeId)
    }
}

impl Display for SurchargeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SurchargeKind {
    Overtime,
    Sunday,
    Holiday,
    Night,
    Other,
}

impl SurchargeKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Overtime => "OVERTIME",
            Self::Sunday => "SUNDAY",
            Self::Holiday => "HOLIDAY",
            Self::Night => "NIGHT",
            Self::Other => "OTHER",
        }
    }
}

impl FromStr for SurchargeKind {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "OVERTIME" => Ok(Self::Overtime),
            "SUNDAY" => Ok(Self::Sunday),
            "HOLIDAY" => Ok(Self::Holiday),
            "NIGHT" => Ok(Self::Night),
            "OTHER" => Ok(Self::Other),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct RateBp(i32);

impl RateBp {
    pub const ZERO: Self = Self(0);

    pub fn get(self) -> i32 {
        self.0
    }
}

impl TryFrom<i32> for RateBp {
    type Error = SurchargeError;

    fn try_from(value: i32) -> Result<Self, Self::Error> {
        if (0..=MAX_RATE_BP).contains(&value) {
            Ok(Self(value))
        } else {
            Err(SurchargeError::RateInvalid)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurchargeName(String);

impl SurchargeName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for SurchargeName {
    type Error = SurchargeError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            Err(SurchargeError::NameEmpty)
        } else {
            Ok(Self(trimmed.to_owned()))
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct SurchargeInput<'a> {
    pub name: &'a str,
    pub rate_bp: i32,
    pub kind: SurchargeKind,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Surcharge {
    pub id: SurchargeId,
    pub organization_id: OrganizationId,
    pub name: SurchargeName,
    pub rate: RateBp,
    pub kind: SurchargeKind,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
}

impl Surcharge {
    pub fn new(
        id: SurchargeId,
        organization_id: OrganizationId,
        input: SurchargeInput<'_>,
        created_at: DateTime<Utc>,
    ) -> SurchargeOutcome<Self> {
        Ok(Self {
            id,
            organization_id,
            name: SurchargeName::try_from(input.name)?,
            rate: RateBp::try_from(input.rate_bp)?,
            kind: input.kind,
            archived_at: None,
            created_at,
        })
    }

    pub fn revise(&mut self, input: SurchargeInput<'_>) -> SurchargeOutcome<()> {
        let name = SurchargeName::try_from(input.name)?;
        let rate = RateBp::try_from(input.rate_bp)?;
        self.name = name;
        self.rate = rate;
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
