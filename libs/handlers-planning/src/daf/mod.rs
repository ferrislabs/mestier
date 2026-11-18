//! The DAF of a project: the dossier header, the sheets, a library photo
//! placed on a sheet, and the PDF exports. See docs/chantiers/daf.md.

use auth::Identity;
use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use handlers::{ApiError, AppState};
use mestier_core::{Daf, DafId, OrganizationId, ProjectId};
use serde::Deserialize;

use crate::require_org_membership;

pub mod create;
pub mod delete;
pub mod dossier_pdf;
pub mod get_one;
pub mod list;
pub mod pdf;
pub mod place_photo;
pub mod request;
pub mod response;
pub mod save_dossier;
pub mod update;

pub fn router(_state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_get(list::handler)
        .typed_put(save_dossier::handler)
        .typed_post(create::handler)
        .typed_get(get_one::handler)
        .typed_put(update::handler)
        .typed_delete(delete::handler)
        .typed_post(place_photo::handler)
        .typed_get(pdf::handler)
        .typed_get(dossier_pdf::handler)
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

pub(crate) fn belongs_to(daf: Daf, organization_id: OrganizationId) -> Result<Daf, ApiError> {
    if daf.organization_id != organization_id {
        return Err(ApiError::NotFound);
    }
    Ok(daf)
}

pub(crate) async fn require_daf(
    state: &AppState,
    identity: &Identity,
    organization_id: OrganizationId,
    daf_id: DafId,
) -> Result<Daf, ApiError> {
    require_org_membership(state, identity, organization_id).await?;
    belongs_to(
        state.usecase.get_daf(organization_id, daf_id).await?,
        organization_id,
    )
}
