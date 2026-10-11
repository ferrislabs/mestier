//! What a renderer is handed, and the port it implements.
//!
//! `DoeDocument` is the DOE as it will be printed: disabled sections gone,
//! sections and supplies numbered, labels resolved. Everything a renderer
//! could get wrong about the business is decided before it is called, in
//! `DoeDocument::from_doe` (see `document.rs`), so the renderer only lays out.

use std::collections::HashMap;

use chrono::NaiveDate;
use serde::Serialize;

use crate::domain::{
    doe::{DoeFile, DoeParty, DoeSupply, PhotosPerPage},
    doe_model::{DoeStyle, HexColor, LogoPosition, LogoSize},
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DoeDocument {
    pub title: String,
    pub place: Option<String>,
    pub lot: Option<String>,
    pub issued_on: NaiveDate,
    pub author: Option<String>,
    pub revision: String,
    pub parties: Vec<DoeParty>,
    pub sections: Vec<NumberedSection>,
    pub style: DocumentStyle,
    /// The running header's text, variables already replaced.
    pub header: String,
    pub footer: String,
    /// Set only when the DOE has a cover photo and the style prints it.
    pub cover_photo_key: Option<String>,
}

/// A `DoeStyle` as the renderer reads it: fonts as family names, labels as
/// plain text.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct DocumentStyle {
    pub primary: HexColor,
    pub secondary: HexColor,
    pub on_primary: HexColor,
    pub heading_font: String,
    pub body_font: String,
    pub logo_key: Option<String>,
    pub cover_logo_position: LogoPosition,
    pub cover_logo_size: LogoSize,
    pub header_logo: bool,
    pub cover_title: String,
    pub parties_title: String,
    pub contents_title: String,
    pub supply_table_title: String,
}

impl From<&DoeStyle> for DocumentStyle {
    fn from(style: &DoeStyle) -> Self {
        Self {
            primary: style.primary.clone(),
            secondary: style.secondary.clone(),
            on_primary: style.on_primary.clone(),
            heading_font: style.heading_font.family().to_owned(),
            body_font: style.body_font.family().to_owned(),
            logo_key: style.logo_key.clone(),
            cover_logo_position: style.cover_logo_position,
            cover_logo_size: style.cover_logo_size,
            header_logo: style.header_logo,
            cover_title: style.cover_title.as_str().to_owned(),
            parties_title: style.parties_title.as_str().to_owned(),
            contents_title: style.contents_title.as_str().to_owned(),
            supply_table_title: style.supply_table_title.as_str().to_owned(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NumberedSection {
    /// 1-based, counting enabled sections only.
    pub number: u16,
    pub title: String,
    pub content: NumberedContent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum NumberedContent {
    Supplies {
        /// Already in French: "Fabricant" or "Pépinière".
        maker_label: String,
        supplies: Vec<NumberedSupply>,
    },
    Photos {
        per_page: PhotosPerPage,
        photos: Vec<DoeFile>,
    },
    Plans {
        plans: Vec<DoeFile>,
    },
    BlankPages {
        count: u8,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct NumberedSupply {
    /// "1.2": section number, then 1-based position in the section.
    pub number: String,
    /// The supply's title, or its designation when it has none.
    pub heading: String,
    pub supply: DoeSupply,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BundledFile {
    pub mime_type: String,
    pub bytes: Vec<u8>,
}

/// Every file a document references, by storage key. The renderer reads
/// nothing else: no disk, no network.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FileBundle(pub HashMap<String, BundledFile>);

impl FileBundle {
    pub fn get(&self, key: &str) -> Option<&BundledFile> {
        self.0.get(key)
    }

    pub fn insert(&mut self, key: String, file: BundledFile) {
        self.0.insert(key, file);
    }
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DoeRenderError {
    #[error("file `{key}` is referenced but was not provided")]
    MissingFile { key: String },
    #[error("file `{key}` has type `{mime_type}`, which cannot be placed in a document")]
    UnsupportedFile { key: String, mime_type: String },
    /// The document would exceed what one render may cost (pages, bytes).
    #[error("document too large: {0}")]
    TooLarge(String),
    #[error("typesetting failed: {0}")]
    Typeset(String),
}

/// Lays a document out as a PDF. CPU-bound and synchronous: callers run it
/// off the async executor.
pub trait DoeRenderer: Send + Sync {
    fn render(&self, document: &DoeDocument, files: &FileBundle)
    -> Result<Vec<u8>, DoeRenderError>;

    /// The same layout as `render`, as one SVG per requested page (0-based),
    /// for an on-screen preview. Pages past the end are skipped.
    fn render_svg(
        &self,
        document: &DoeDocument,
        files: &FileBundle,
        pages: &[usize],
    ) -> Result<Vec<String>, DoeRenderError>;
}
