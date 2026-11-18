//! The DAF of a project: the dossier header, the sheets, a library photo
//! placed on a sheet, and the PDF exports. See docs/chantiers/daf.md.

use axum::Router;
use axum_extra::routing::TypedPath;
use handlers::AppState;
use mestier_core::{DafId, OrganizationId, ProjectId};
use serde::Deserialize;

pub mod request;
pub mod response;

pub fn router(_state: &AppState) -> Router<AppState> {
    Router::new()
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/projects/{project_id}/dafs")]
pub struct ProjectDafsPath {
    pub organization_id: OrganizationId,
    pub project_id: ProjectId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/projects/{project_id}/daf-dossier")]
pub struct DafDossierPath {
    pub organization_id: OrganizationId,
    pub project_id: ProjectId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/projects/{project_id}/dafs/pdf")]
pub struct DafDossierPdfPath {
    pub organization_id: OrganizationId,
    pub project_id: ProjectId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/dafs/{daf_id}")]
pub struct DafPath {
    pub organization_id: OrganizationId,
    pub daf_id: DafId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/dafs/{daf_id}/photo/place")]
pub struct DafPlacePhotoPath {
    pub organization_id: OrganizationId,
    pub daf_id: DafId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/dafs/{daf_id}/pdf")]
pub struct DafPdfPath {
    pub organization_id: OrganizationId,
    pub daf_id: DafId,
}
