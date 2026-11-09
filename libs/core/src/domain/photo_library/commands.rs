use uuid::Uuid;

use crate::{
    DoeId, LibraryPhotoId, OrganizationId, ProjectId,
    domain::photo_library::normalize::NormalizedImage,
};

pub const MAX_PLACEMENTS: usize = 50;

#[derive(Debug, Clone)]
pub struct UploadLibraryPhotoCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub project_id: Option<ProjectId>,
    pub image: NormalizedImage,
}

#[derive(Debug, Clone)]
pub struct ListLibraryPhotosCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub query: Option<String>,
    pub tags: Vec<String>,
    pub project_id: Option<ProjectId>,
    pub to_sort: bool,
    pub limit: u64,
    pub offset: u64,
}

#[derive(Debug, Clone)]
pub struct GetLibraryPhotoCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub id: LibraryPhotoId,
}

#[derive(Debug, Clone)]
pub struct DeleteLibraryPhotoCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub id: LibraryPhotoId,
}

#[derive(Debug, Clone)]
pub struct UpdateLibraryPhotoCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub id: LibraryPhotoId,
    pub caption: Option<String>,
    pub tags: Option<Vec<String>>,
    pub project_id: Option<Option<ProjectId>>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PlacementTargetKind {
    Section,
    Supply,
    Cover,
}

#[derive(Debug, Clone)]
pub struct PlacementRequest {
    pub photo_id: LibraryPhotoId,
    pub target: PlacementTargetKind,
    pub section_id: Option<Uuid>,
    pub section_title: Option<String>,
    pub supply_id: Option<Uuid>,
    pub caption: Option<String>,
}

#[derive(Debug, Clone)]
pub struct PlaceLibraryPhotosCommand {
    pub actor: authz::Subject,
    pub organization_id: OrganizationId,
    pub doe_id: DoeId,
    pub placements: Vec<PlacementRequest>,
}
