use chrono::{DateTime, Utc};
use common::CoreError;
use uuid::Uuid;

use crate::{LibraryPhoto, LibraryPhotoId, OrganizationId, ProjectId};

#[derive(Debug, Clone)]
pub struct LibraryPhotoRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub file_key: String,
    pub thumbnail_key: String,
    pub mime_type: String,
    pub width: i32,
    pub height: i32,
    pub size_bytes: i32,
    pub content_hash: String,
    pub caption: Option<String>,
    pub tags: Vec<String>,
    pub project_id: Option<Uuid>,
    pub taken_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

fn unsigned(value: i32, column: &str) -> Result<u32, CoreError> {
    u32::try_from(value)
        .map_err(|_| CoreError::Internal(format!("negative {column} in library_photos")))
}

impl TryFrom<LibraryPhotoRow> for LibraryPhoto {
    type Error = CoreError;

    fn try_from(row: LibraryPhotoRow) -> Result<Self, Self::Error> {
        Ok(Self {
            id: LibraryPhotoId(row.id),
            organization_id: OrganizationId(row.org_id),
            file_key: row.file_key,
            thumbnail_key: row.thumbnail_key,
            mime_type: row.mime_type,
            width: unsigned(row.width, "width")?,
            height: unsigned(row.height, "height")?,
            size_bytes: unsigned(row.size_bytes, "size_bytes")?,
            content_hash: row.content_hash,
            caption: row.caption,
            tags: row.tags,
            project_id: row.project_id.map(ProjectId),
            taken_at: row.taken_at,
            created_at: row.created_at,
            updated_at: row.updated_at,
        })
    }
}
