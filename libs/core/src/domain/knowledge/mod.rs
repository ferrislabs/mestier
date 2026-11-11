use std::{fmt::Display, str::FromStr};

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use uuid::Uuid;

use crate::OrganizationId;

pub mod commands;
pub mod error;
pub mod folders;
pub mod ports;
pub mod service;
pub mod tables;
pub mod text;

pub use commands::*;
pub use error::{KnowledgeError, KnowledgeOutcome};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct KnowledgeArticleId(pub Uuid);

impl FromStr for KnowledgeArticleId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(KnowledgeArticleId)
    }
}

impl Display for KnowledgeArticleId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct KnowledgeFolderId(pub Uuid);

impl FromStr for KnowledgeFolderId {
    type Err = uuid::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Uuid::from_str(s).map(KnowledgeFolderId)
    }
}

impl Display for KnowledgeFolderId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// A folder of the drive. `parent_id` is `None` at the root.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeFolder {
    pub id: KnowledgeFolderId,
    pub organization_id: OrganizationId,
    pub parent_id: Option<KnowledgeFolderId>,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Where the articles of a listing are taken from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KnowledgeFolderScope {
    /// Only the articles outside any folder.
    Root,
    /// Only the articles placed directly in this folder.
    Directly(KnowledgeFolderId),
    /// The articles of this folder and of every folder below it.
    Within(KnowledgeFolderId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeKind {
    File,
    Note,
}

impl KnowledgeKind {
    pub fn as_str(self) -> &'static str {
        match self {
            KnowledgeKind::File => "file",
            KnowledgeKind::Note => "note",
        }
    }
}

impl FromStr for KnowledgeKind {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "file" => Ok(KnowledgeKind::File),
            "note" => Ok(KnowledgeKind::Note),
            _ => Err(()),
        }
    }
}

/// The stored file behind an article of kind `file`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeFile {
    pub key: String,
    pub mime_type: String,
    pub size_bytes: u64,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeArticle {
    pub id: KnowledgeArticleId,
    pub organization_id: OrganizationId,
    /// `None` when the article sits at the root of the drive.
    pub folder_id: Option<KnowledgeFolderId>,
    pub kind: KnowledgeKind,
    pub title: String,
    /// The text of a note. `None` for a file.
    pub body: Option<String>,
    /// `None` for a note.
    pub file: Option<KnowledgeFile>,
    /// Searchable text read from the file. Left out of list results.
    pub extracted_text: Option<String>,
    pub tags: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum KnowledgeResourceType {
    Project,
    Customer,
    Quote,
}

impl KnowledgeResourceType {
    pub fn as_str(self) -> &'static str {
        match self {
            KnowledgeResourceType::Project => "project",
            KnowledgeResourceType::Customer => "customer",
            KnowledgeResourceType::Quote => "quote",
        }
    }
}

impl Display for KnowledgeResourceType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

impl FromStr for KnowledgeResourceType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "project" => Ok(KnowledgeResourceType::Project),
            "customer" => Ok(KnowledgeResourceType::Customer),
            "quote" => Ok(KnowledgeResourceType::Quote),
            _ => Err(()),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct KnowledgeResource {
    pub resource_type: KnowledgeResourceType,
    pub id: Uuid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KnowledgeLink {
    pub article_id: KnowledgeArticleId,
    pub resource: KnowledgeResource,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Default)]
pub struct KnowledgeFilter {
    /// Full-text query over title, tags, note and extracted text.
    pub query: Option<String>,
    /// Every tag listed must be on the article.
    pub tags: Vec<String>,
    /// Only the articles linked to this resource.
    pub resource: Option<KnowledgeResource>,
    /// `None` searches the whole drive.
    pub folder: Option<KnowledgeFolderScope>,
}
