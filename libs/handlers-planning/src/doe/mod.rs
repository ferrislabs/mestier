//! The DOE of a project: read, create from a model, replace, draft a supply
//! from a product, and export as PDF. See docs/chantiers/doe.md.

use auth::Identity;
use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use handlers::{ApiError, AppState};
use mestier_core::{Doe, DoeId, OrganizationId, ProjectId};
use serde::Deserialize;

use crate::require_org_membership;

pub mod create;
pub mod delete;
pub mod draft_supply;
pub mod get_one;
pub mod get_project;
pub mod list;
pub mod pdf;
pub mod response;
pub mod update;

pub fn router(_state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_get(list::handler)
        .typed_get(get_project::handler)
        .typed_post(create::handler)
        .typed_get(get_one::handler)
        .typed_put(update::handler)
        .typed_delete(delete::handler)
        .typed_post(draft_supply::handler)
        .typed_get(pdf::handler)
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/projects/{project_id}/doe")]
pub struct ProjectDoePath {
    pub organization_id: OrganizationId,
    pub project_id: ProjectId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/does")]
pub struct DoesPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/does/{doe_id}")]
pub struct DoePath {
    pub organization_id: OrganizationId,
    pub doe_id: DoeId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/does/{doe_id}/supplies/draft")]
pub struct DoeDraftSupplyPath {
    pub organization_id: OrganizationId,
    pub doe_id: DoeId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/does/{doe_id}/pdf")]
pub struct DoePdfPath {
    pub organization_id: OrganizationId,
    pub doe_id: DoeId,
}

pub(crate) fn belongs_to(doe: Doe, organization_id: OrganizationId) -> Result<Doe, ApiError> {
    if doe.organization_id != organization_id {
        return Err(ApiError::NotFound);
    }
    Ok(doe)
}

pub(crate) async fn require_doe(
    state: &AppState,
    identity: &Identity,
    organization_id: OrganizationId,
    doe_id: DoeId,
) -> Result<Doe, ApiError> {
    require_org_membership(state, identity, organization_id).await?;
    belongs_to(state.usecase.get_doe(doe_id).await?, organization_id)
}
