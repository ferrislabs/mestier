use std::str::FromStr;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

use super::error::{AbsenceError, AbsenceOutcome};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum AbsenceStatus {
    Pending,
    Approved,
    Rejected,
    Cancelled,
}

impl AbsenceStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "PENDING",
            Self::Approved => "APPROVED",
            Self::Rejected => "REJECTED",
            Self::Cancelled => "CANCELLED",
        }
    }

    pub fn approve(self) -> AbsenceOutcome<Self> {
        match self {
            Self::Pending => Ok(Self::Approved),
            _ => Err(AbsenceError::AlreadyDecided),
        }
    }

    pub fn reject(self) -> AbsenceOutcome<Self> {
        match self {
            Self::Pending => Ok(Self::Rejected),
            _ => Err(AbsenceError::AlreadyDecided),
        }
    }

    pub fn cancel(self) -> AbsenceOutcome<Self> {
        match self {
            Self::Pending | Self::Approved => Ok(Self::Cancelled),
            _ => Err(AbsenceError::AlreadyDecided),
        }
    }

    pub fn is_in_force(self) -> bool {
        self == Self::Approved
    }
}

impl FromStr for AbsenceStatus {
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
