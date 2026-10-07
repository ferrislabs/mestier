//! DOE (dossier des ouvrages exécutés): the file handed to the customer at
//! reception, listing what was installed on the job.
//!
//! A DOE is a document, not a view over the catalogue. Every supply in it is
//! a copy of a product's technical sheet taken when it was added, so editing
//! a product or a supplier afterwards cannot change a file already handed
//! over. `DoeSupply::source_product_id` only says where the copy came from;
//! nothing reads through it.
//!
//! Spec: `docs/chantiers/doe.md`.

use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, NaiveDate, Utc};
use common::CoreError;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{OrganizationId, ProductId, ProjectId};

pub mod commands;
pub mod document;
pub mod ports;
pub mod render;
pub mod service;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct DoeId(pub Uuid);

impl FromStr for DoeId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(DoeId)
    }
}

impl Display for DoeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// The DOE's revision mark, `A`, `B`, `C`… as printed on the cover. Free
/// text because some customers number them, never blank.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(try_from = "String", into = "String")]
pub struct Revision(String);

impl Revision {
    pub const MAX_CHARS: usize = 8;

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl TryFrom<String> for Revision {
    type Error = CoreError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let trimmed = value.trim();
        let chars = trimmed.chars().count();
        if chars == 0 || chars > Self::MAX_CHARS {
            return Err(CoreError::Conflict(format!(
                "revision must be 1 to {} characters",
                Self::MAX_CHARS
            )));
        }
        Ok(Self(trimmed.to_owned()))
    }
}

impl From<Revision> for String {
    fn from(value: Revision) -> Self {
        value.0
    }
}

/// How many blank pages a `BlankPages` section prints.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(try_from = "u8", into = "u8")]
pub struct BlankPageCount(u8);

impl BlankPageCount {
    pub const MAX: u8 = 20;

    pub fn get(self) -> u8 {
        self.0
    }
}

impl TryFrom<u8> for BlankPageCount {
    type Error = CoreError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        if value == 0 || value > Self::MAX {
            return Err(CoreError::Conflict(format!(
                "blank page count must be 1 to {}",
                Self::MAX
            )));
        }
        Ok(Self(value))
    }
}

impl From<BlankPageCount> for u8 {
    fn from(value: BlankPageCount) -> Self {
        value.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PhotosPerPage {
    One,
    Two,
    Four,
}

impl PhotosPerPage {
    pub fn get(self) -> usize {
        match self {
            Self::One => 1,
            Self::Two => 2,
            Self::Four => 4,
        }
    }
}

/// What the "made by" row of a supply sheet is called. A plant comes from a
/// nursery, a bench from a manufacturer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum MakerLabel {
    Manufacturer,
    Nursery,
}

impl MakerLabel {
    pub fn french(self) -> &'static str {
        match self {
            Self::Manufacturer => "Fabricant",
            Self::Nursery => "Pépinière",
        }
    }
}

/// Maîtrise d'ouvrage, maîtrise d'œuvre, and the contractor issuing the DOE.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PartyRole {
    ProjectOwner,
    ProjectManager,
    Contractor,
}

impl PartyRole {
    pub fn french(self) -> &'static str {
        match self {
            Self::ProjectOwner => "Maîtrise d’ouvrage",
            Self::ProjectManager => "Maîtrise d’œuvre",
            Self::Contractor => "Entreprise",
        }
    }
}

/// A party as printed in the DOE. Copied, like supplies: the customer moving
/// offices does not rewrite a file already handed over.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct DoeParty {
    pub role: PartyRole,
    pub name: String,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
    pub logo_key: Option<String>,
}

/// A stored file placed in the DOE: a photo, a plan. `caption` is the photo
/// legend or the plan's designation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct DoeFile {
    pub key: String,
    pub caption: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct SupplierSnapshot {
    pub name: String,
    pub address: Option<String>,
    pub phone: Option<String>,
    pub email: Option<String>,
}

/// One supply sheet: "1.1 Plantation d'arbres" and its identification table.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct DoeSupply {
    pub id: Uuid,
    pub source_product_id: Option<ProductId>,
    /// The heading, when it differs from the designation ("Plantation
    /// d'arbres" for "Arbres"). `None` prints the designation.
    pub title: Option<String>,
    pub designation: String,
    /// The line of the DPGF (price breakdown of a public works contract)
    /// this supply answers to.
    pub dpgf_ref: Option<String>,
    pub destination: Option<String>,
    pub supplier: Option<SupplierSnapshot>,
    pub maker: Option<String>,
    pub specs: Option<String>,
    pub photos: Vec<DoeFile>,
    /// The supplier's own documents, inserted page by page after the sheet.
    pub datasheet_keys: Vec<String>,
}

/// What a section holds. The content lives in the variant, so a photo in a
/// supplies section cannot be represented.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DoeSectionContent {
    Supplies {
        maker_label: MakerLabel,
        supplies: Vec<DoeSupply>,
    },
    Photos {
        per_page: PhotosPerPage,
        photos: Vec<DoeFile>,
    },
    Plans {
        plans: Vec<DoeFile>,
    },
    BlankPages {
        count: BlankPageCount,
    },
}

/// A numbered theme of the DOE. A disabled section is kept, so unticking
/// "Jeux" and ticking it back loses nothing, but it is neither printed nor
/// numbered.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct DoeSection {
    pub id: Uuid,
    pub title: String,
    pub enabled: bool,
    pub content: DoeSectionContent,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Doe {
    pub id: DoeId,
    pub organization_id: OrganizationId,
    pub project_id: ProjectId,
    pub title: String,
    pub place: Option<String>,
    pub lot: Option<String>,
    pub issued_on: NaiveDate,
    pub author: Option<String>,
    pub revision: Revision,
    pub parties: Vec<DoeParty>,
    pub sections: Vec<DoeSection>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn revision_is_trimmed() {
        assert_eq!(Revision::try_from(" B ".to_owned()).unwrap().as_str(), "B");
    }

    #[test]
    fn revision_rejects_blank_and_too_long() {
        assert!(Revision::try_from("  ".to_owned()).is_err());
        assert!(Revision::try_from("ABCDEFGHI".to_owned()).is_err());
        assert!(Revision::try_from("ABCDEFGH".to_owned()).is_ok());
    }

    #[test]
    fn blank_page_count_bounds() {
        assert!(BlankPageCount::try_from(0).is_err());
        assert!(BlankPageCount::try_from(1).is_ok());
        assert!(BlankPageCount::try_from(20).is_ok());
        assert!(BlankPageCount::try_from(21).is_err());
    }

    #[test]
    fn section_content_rejects_invalid_count_from_json() {
        let json = r#"{"kind":"blank_pages","count":0}"#;
        assert!(serde_json::from_str::<DoeSectionContent>(json).is_err());
    }

    #[test]
    fn section_content_round_trips() {
        let content = DoeSectionContent::Photos {
            per_page: PhotosPerPage::Two,
            photos: vec![DoeFile {
                key: "doe/photo".to_owned(),
                caption: Some("Massif".to_owned()),
            }],
        };
        let json = serde_json::to_string(&content).unwrap();
        assert!(json.contains(r#""kind":"photos""#));
        assert_eq!(
            serde_json::from_str::<DoeSectionContent>(&json).unwrap(),
            content
        );
    }
}
