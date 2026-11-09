use chrono::{DateTime, Utc};
use mestier_core::{
    LibraryPhoto, LibraryPhotoId, PlacementReport, PlacementResult, PlacementStatus, ProjectId,
};
use serde::Serialize;
use utoipa::ToSchema;

use crate::doe::response::DoeResponse;

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct LibraryPhotoResponse {
    pub id: LibraryPhotoId,
    pub file_key: String,
    pub thumbnail_key: String,
    pub mime_type: String,
    pub width: u32,
    pub height: u32,
    pub size_bytes: u32,
    pub caption: Option<String>,
    pub tags: Vec<String>,
    pub project_id: Option<ProjectId>,
    pub taken_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<LibraryPhoto> for LibraryPhotoResponse {
    fn from(photo: LibraryPhoto) -> Self {
        Self {
            id: photo.id,
            file_key: photo.file_key,
            thumbnail_key: photo.thumbnail_key,
            mime_type: photo.mime_type,
            width: photo.width,
            height: photo.height,
            size_bytes: photo.size_bytes,
            caption: photo.caption,
            tags: photo.tags,
            project_id: photo.project_id,
            taken_at: photo.taken_at,
            created_at: photo.created_at,
            updated_at: photo.updated_at,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, PartialEq, Eq, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PlacementStatusResponse {
    Placed,
    Failed,
}

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct PlacementResultResponse {
    pub photo_id: LibraryPhotoId,
    pub status: PlacementStatusResponse,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

impl From<PlacementResult> for PlacementResultResponse {
    fn from(result: PlacementResult) -> Self {
        Self {
            photo_id: result.photo_id,
            status: match result.status {
                PlacementStatus::Placed => PlacementStatusResponse::Placed,
                PlacementStatus::Failed => PlacementStatusResponse::Failed,
            },
            key: result.key,
            error: result.error,
        }
    }
}

#[derive(Debug, Serialize, PartialEq, ToSchema)]
pub struct PlacementReportResponse {
    pub doe: DoeResponse,
    pub results: Vec<PlacementResultResponse>,
}

impl From<PlacementReport> for PlacementReportResponse {
    fn from(report: PlacementReport) -> Self {
        Self {
            doe: report.doe.into(),
            results: report.results.into_iter().map(Into::into).collect(),
        }
    }
}
