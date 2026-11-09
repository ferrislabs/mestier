use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{OrganizationId, ProjectId};

pub mod commands;
pub mod error;
pub mod normalize;
pub mod ports;
pub mod service;
pub mod tags;

pub use error::{PhotoLibraryError, PhotoLibraryOutcome};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct LibraryPhotoId(pub Uuid);

impl FromStr for LibraryPhotoId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(LibraryPhotoId)
    }
}

impl Display for LibraryPhotoId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LibraryPhoto {
    pub id: LibraryPhotoId,
    pub organization_id: OrganizationId,
    pub file_key: String,
    pub thumbnail_key: String,
    pub mime_type: String,
    pub width: u32,
    pub height: u32,
    pub size_bytes: u32,
    pub content_hash: String,
    pub caption: Option<String>,
    pub tags: Vec<String>,
    pub project_id: Option<ProjectId>,
    pub taken_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl LibraryPhoto {
    pub fn is_to_sort(&self) -> bool {
        self.caption.is_none() && self.tags.is_empty()
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct PhotoFilter {
    pub query: Option<String>,
    pub tags: Vec<String>,
    pub project_id: Option<ProjectId>,
    pub to_sort: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InsertOutcome {
    Inserted(LibraryPhoto),
    Existing(LibraryPhoto),
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BackfillReport {
    pub imported: u32,
    pub already_present: u32,
    pub skipped: u32,
    pub failed: u32,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UploadedPhoto {
    pub photo: LibraryPhoto,
    pub created: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlacementStatus {
    Placed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlacementResult {
    pub photo_id: LibraryPhotoId,
    pub status: PlacementStatus,
    pub key: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct PlacementReport {
    pub doe: crate::Doe,
    pub results: Vec<PlacementResult>,
}
