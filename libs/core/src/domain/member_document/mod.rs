use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{MemberId, OrganizationId};

pub mod commands;
pub mod error;
pub mod ports;
pub mod service;

#[cfg(test)]
mod tests;

pub use error::{MemberDocumentError, MemberDocumentOutcome};
pub use service::MemberDocumentService;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct MemberDocumentId(pub Uuid);

impl FromStr for MemberDocumentId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(MemberDocumentId)
    }
}

impl Display for MemberDocumentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum DocumentKind {
    Contract,
    Identity,
    Diploma,
    Medical,
    Other,
}

impl DocumentKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Contract => "CONTRACT",
            Self::Identity => "IDENTITY",
            Self::Diploma => "DIPLOMA",
            Self::Medical => "MEDICAL",
            Self::Other => "OTHER",
        }
    }
}

impl FromStr for DocumentKind {
    type Err = ();

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "CONTRACT" => Ok(Self::Contract),
            "IDENTITY" => Ok(Self::Identity),
            "DIPLOMA" => Ok(Self::Diploma),
            "MEDICAL" => Ok(Self::Medical),
            "OTHER" => Ok(Self::Other),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DocumentName(String);

impl DocumentName {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for DocumentName {
    type Error = MemberDocumentError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let trimmed = value.trim();
        if trimmed.is_empty() {
            Err(MemberDocumentError::NameEmpty)
        } else {
            Ok(Self(trimmed.to_owned()))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FileKey(String);

impl FileKey {
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<&str> for FileKey {
    type Error = MemberDocumentError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        let invalid = value.trim().is_empty()
            || value != value.trim()
            || value.starts_with('/')
            || value.contains("..")
            || value.chars().any(char::is_control);
        if invalid {
            Err(MemberDocumentError::FileKeyInvalid)
        } else {
            Ok(Self(value.to_owned()))
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MemberDocument {
    pub id: MemberDocumentId,
    pub organization_id: OrganizationId,
    pub member_id: MemberId,
    pub name: DocumentName,
    pub kind: DocumentKind,
    pub file_key: FileKey,
    pub added_at: DateTime<Utc>,
}

impl MemberDocument {
    pub fn new(
        id: MemberDocumentId,
        organization_id: OrganizationId,
        member_id: MemberId,
        name: &str,
        kind: DocumentKind,
        file_key: &str,
        added_at: DateTime<Utc>,
    ) -> MemberDocumentOutcome<Self> {
        Ok(Self {
            id,
            organization_id,
            member_id,
            name: DocumentName::try_from(name)?,
            kind,
            file_key: FileKey::try_from(file_key)?,
            added_at,
        })
    }
}
