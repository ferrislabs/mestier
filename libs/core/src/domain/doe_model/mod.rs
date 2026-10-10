//! DOE models: the sections a new DOE starts with, per organization.
//!
//! A DOE is the same document in every trade: cover, parties, numbered
//! themes, photos, plans. What differs is which themes, so that is data,
//! not code. Mestier ships starter models (`StarterModel`) an organization
//! copies once and then owns; editing its copy never touches the starter.
//!
//! Spec: `docs/chantiers/doe.md`.

use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, Utc};
use common::CoreError;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    OrganizationId,
    domain::doe::{BlankPageCount, MakerLabel, PhotosPerPage},
};

pub mod commands;
pub mod ports;
pub mod service;
pub mod starter;
pub mod style;

pub use style::{DoeFont, DoeLabel, DoeStyle, DoeTemplate, LogoPosition, LogoSize, TemplateValues};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct DoeModelId(pub Uuid);

impl FromStr for DoeModelId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(DoeModelId)
    }
}

impl Display for DoeModelId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A `#rrggbb` color, lowercased.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(try_from = "String", into = "String")]
pub struct HexColor(String);

impl HexColor {
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// Mestier's brand green, used until an organization picks its own.
    pub fn mestier_default() -> Self {
        Self("#0f3d36".to_owned())
    }

    pub fn white() -> Self {
        Self("#ffffff".to_owned())
    }
}

impl TryFrom<String> for HexColor {
    type Error = CoreError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        let lower = value.trim().to_ascii_lowercase();
        let valid = lower.len() == 7
            && lower.starts_with('#')
            && lower[1..].chars().all(|c| c.is_ascii_hexdigit());
        if !valid {
            return Err(CoreError::Conflict("color must be #rrggbb".to_owned()));
        }
        Ok(Self(lower))
    }
}

impl From<HexColor> for String {
    fn from(value: HexColor) -> Self {
        value.0
    }
}

/// A section's kind and settings, without content: a model holds no supply,
/// photo or plan.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum DoeModelSectionKind {
    Supplies { maker_label: MakerLabel },
    Photos { per_page: PhotosPerPage },
    Plans,
    BlankPages { count: BlankPageCount },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
pub struct DoeModelSection {
    pub title: String,
    /// A disabled section is offered in the DOE but starts unticked, like
    /// "Jeux" for a job without a playground.
    pub enabled: bool,
    pub kind: DoeModelSectionKind,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DoeModel {
    pub id: DoeModelId,
    pub organization_id: OrganizationId,
    pub name: String,
    pub description: Option<String>,
    pub sections: Vec<DoeModelSection>,
    /// Replaces the organization's style for the DOEs made from this model.
    /// `None` keeps the organization's.
    pub style: Option<DoeStyle>,
    pub archived_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl DoeModel {
    pub fn is_archived(&self) -> bool {
        self.archived_at.is_some()
    }
}

/// Models shipped with Mestier. Copied into an organization on request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum StarterModel {
    Landscaping,
}

/// How an organization's documents look. One row per organization; an
/// organization without one gets `DoeSettings::default_for`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DoeSettings {
    pub organization_id: OrganizationId,
    pub style: DoeStyle,
}

impl DoeSettings {
    pub fn default_for(organization_id: OrganizationId) -> Self {
        Self {
            organization_id,
            style: DoeStyle::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hex_color_is_lowercased() {
        assert_eq!(
            HexColor::try_from("#0F3D36".to_owned()).unwrap().as_str(),
            "#0f3d36"
        );
    }

    #[test]
    fn hex_color_rejects_malformed() {
        for bad in ["0f3d36", "#0f3d3", "#0f3d366", "#0g3d36", ""] {
            assert!(HexColor::try_from(bad.to_owned()).is_err(), "{bad}");
        }
    }
}
