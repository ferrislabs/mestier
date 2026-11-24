//! DAF (demande d'agrément de fourniture): the sheet of one supply submitted
//! to the project manager for approval before it is installed.
//!
//! A project holds many DAF, numbered, and one dossier carrying the header
//! they share. Like a DOE supply, a DAF keeps copies (supplier, photo,
//! datasheets): changing the catalogue never rewrites a sheet already sent.
//!
//! Spec: `docs/chantiers/daf.md`.

use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, NaiveDate, Utc};
use common::CoreError;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{OrganizationId, ProductId, ProjectId, SupplierSnapshot};

pub mod commands;
pub mod document;
pub mod ports;
pub mod render;
pub mod service;

/// At most this many DAF per project, so the dossier export stays bounded.
pub const MAX_DAFS_PER_PROJECT: usize = 200;
/// At most this many attachments per DAF.
pub const MAX_ATTACHMENTS: usize = 20;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct DafId(pub Uuid);

impl FromStr for DafId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(DafId)
    }
}

impl Display for DafId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The DAF's number within its project, printed "D.A.F n°09".
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, ToSchema)]
#[serde(try_from = "u16", into = "u16")]
pub struct DafNumber(u16);

impl DafNumber {
    pub const MAX: u16 = 999;

    pub fn get(self) -> u16 {
        self.0
    }

    /// The number after this one, or `None` past `MAX`.
    pub fn next(self) -> Option<Self> {
        Self::try_from(self.0 + 1).ok()
    }

    /// "D.A.F n°09": two digits at least, as on paper.
    pub fn label(self) -> String {
        format!("D.A.F n°{:02}", self.0)
    }
}

impl TryFrom<u16> for DafNumber {
    type Error = CoreError;

    fn try_from(value: u16) -> Result<Self, Self::Error> {
        if value == 0 || value > Self::MAX {
            return Err(CoreError::Conflict(format!(
                "DAF number must be 1 to {}",
                Self::MAX
            )));
        }
        Ok(Self(value))
    }
}

impl From<DafNumber> for u16 {
    fn from(value: DafNumber) -> Self {
        value.0
    }
}

/// What is submitted: "Jeux - Ensemble table bois". Never blank.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(try_from = "String", into = "String")]
pub struct DafDesignation(String);

impl DafDesignation {
    pub const MAX_CHARS: usize = 200;

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for DafDesignation {
    type Error = CoreError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let trimmed = value.trim();
        let chars = trimmed.chars().count();
        if chars == 0 || chars > Self::MAX_CHARS {
            return Err(CoreError::Conflict(format!(
                "designation must be 1 to {} characters",
                Self::MAX_CHARS
            )));
        }
        Ok(Self(trimmed.to_owned()))
    }
}

impl From<DafDesignation> for String {
    fn from(value: DafDesignation) -> Self {
        value.0
    }
}

/// The project manager's answer. The VISAS box of the sheet stays blank on
/// paper; this is what the summary page reports.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum DafStatus {
    #[default]
    Pending,
    Accepted,
    RejectedWithComments,
    Rejected,
}

impl DafStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pending => "pending",
            Self::Accepted => "accepted",
            Self::RejectedWithComments => "rejected_with_comments",
            Self::Rejected => "rejected",
        }
    }

    pub fn french(self) -> &'static str {
        match self {
            Self::Pending => "En attente",
            Self::Accepted => "Acceptée",
            Self::RejectedWithComments => "Refus avec obs.",
            Self::Rejected => "Refus ferme",
        }
    }
}

impl FromStr for DafStatus {
    type Err = CoreError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "pending" => Ok(Self::Pending),
            "accepted" => Ok(Self::Accepted),
            "rejected_with_comments" => Ok(Self::RejectedWithComments),
            "rejected" => Ok(Self::Rejected),
            other => Err(CoreError::Internal(format!("unknown DAF status `{other}`"))),
        }
    }
}

/// A stored file inserted after the sheet, and listed by `name` in the
/// "Pièces jointes" box.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct DafAttachment {
    pub key: String,
    pub name: String,
}

/// The header every DAF of a project shares.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DafDossier {
    pub organization_id: OrganizationId,
    pub project_id: ProjectId,
    /// "Aménagement des espaces paysagers du parc Montgroux".
    pub title: String,
    /// The company sending the DAF: "Treyve Paysages".
    pub issuer: String,
    /// Maîtrise d'ouvrage.
    pub project_owner: Option<String>,
    /// Maîtrise d'œuvre.
    pub project_manager: Option<String>,
    /// `None` while the dossier is only prefilled, not yet saved.
    pub updated_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Daf {
    pub id: DafId,
    pub organization_id: OrganizationId,
    pub project_id: ProjectId,
    pub number: DafNumber,
    pub designation: DafDesignation,
    /// "Correspondance": the price line of the contract, "N° PRIX : 6.5.5".
    pub price_ref: Option<String>,
    /// Where the copy came from. Informative: nothing reads through it.
    pub source_product_id: Option<ProductId>,
    pub supplier: Option<SupplierSnapshot>,
    /// "Assemblage et pose": who installs, several lines.
    pub installer: Option<String>,
    /// "Caractéristiques techniques".
    pub specs: Option<String>,
    /// The free "OBSERVATIONS" block of the sheet.
    pub observations: Option<String>,
    pub photo_key: Option<String>,
    pub attachments: Vec<DafAttachment>,
    /// `None` prints the dossier's.
    pub project_owner: Option<String>,
    /// `None` prints the dossier's.
    pub project_manager: Option<String>,
    pub status: DafStatus,
    /// "Envoyé".
    pub sent_on: Option<NaiveDate>,
    /// "Date de diffusion".
    pub issued_on: NaiveDate,
    /// "Retour": when the answer came back.
    pub returned_on: Option<NaiveDate>,
    /// The project manager's remark, the summary's "Observation" column.
    pub review_note: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// A DAF as the summary lists it.
#[derive(Debug, Clone, PartialEq)]
pub struct DafSummary {
    pub id: DafId,
    pub number: DafNumber,
    pub designation: DafDesignation,
    pub price_ref: Option<String>,
    pub status: DafStatus,
    pub sent_on: Option<NaiveDate>,
    pub issued_on: NaiveDate,
    pub returned_on: Option<NaiveDate>,
    pub review_note: Option<String>,
    pub updated_at: DateTime<Utc>,
}

impl From<&Daf> for DafSummary {
    fn from(daf: &Daf) -> Self {
        Self {
            id: daf.id,
            number: daf.number,
            designation: daf.designation.clone(),
            price_ref: daf.price_ref.clone(),
            status: daf.status,
            sent_on: daf.sent_on,
            issued_on: daf.issued_on,
            returned_on: daf.returned_on,
            review_note: daf.review_note.clone(),
            updated_at: daf.updated_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn number_is_bounded() {
        assert!(DafNumber::try_from(0).is_err());
        assert!(DafNumber::try_from(1000).is_err());
        assert_eq!(DafNumber::try_from(999).unwrap().get(), 999);
        assert_eq!(DafNumber::try_from(999).unwrap().next(), None);
    }

    #[test]
    fn number_label_has_two_digits() {
        assert_eq!(DafNumber::try_from(9).unwrap().label(), "D.A.F n°09");
        assert_eq!(DafNumber::try_from(120).unwrap().label(), "D.A.F n°120");
    }

    #[test]
    fn designation_is_trimmed_and_never_blank() {
        assert!(DafDesignation::try_from("  ".to_owned()).is_err());
        assert!(DafDesignation::try_from("x".repeat(201)).is_err());
        assert_eq!(
            DafDesignation::try_from(" Ganivelle ".to_owned())
                .unwrap()
                .as_str(),
            "Ganivelle"
        );
    }

    #[test]
    fn status_round_trips() {
        for status in [
            DafStatus::Pending,
            DafStatus::Accepted,
            DafStatus::RejectedWithComments,
            DafStatus::Rejected,
        ] {
            assert_eq!(status.as_str().parse::<DafStatus>().unwrap(), status);
        }
    }
}
