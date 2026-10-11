use auth::Identity;
use axum::Router;
use axum_extra::routing::{RouterExt, TypedPath};
use handlers::{ApiError, AppState};
use mestier_core::{DoeModel, DoeModelId};
use serde::Deserialize;

use crate::require_org_membership;

pub mod archive;
pub mod create;
pub mod from_starter;
pub mod get_one;
pub mod list;
pub mod preview;
pub mod response;
pub mod restore;
pub mod settings;
pub mod starters;
pub mod update;

pub fn router(_state: &AppState) -> Router<AppState> {
    Router::new()
        .typed_get(list::handler)
        .typed_post(create::handler)
        .typed_post(from_starter::handler)
        .typed_get(get_one::handler)
        .typed_put(update::handler)
        .typed_delete(archive::handler)
        .typed_post(restore::handler)
        .typed_get(starters::handler)
        .typed_post(preview::handler)
        .typed_post(preview::pages_handler)
        .typed_get(settings::get_handler)
        .typed_put(settings::put_handler)
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/doe-models")]
pub struct DoeModelsPath {
    pub organization_id: mestier_core::OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/doe-models/from-starter")]
pub struct DoeModelFromStarterPath {
    pub organization_id: mestier_core::OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/doe-models/{doe_model_id}")]
pub struct DoeModelPath {
    pub organization_id: mestier_core::OrganizationId,
    pub doe_model_id: DoeModelId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/doe-models/{doe_model_id}/restore")]
pub struct DoeModelRestorePath {
    pub organization_id: mestier_core::OrganizationId,
    pub doe_model_id: DoeModelId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/doe-model-starters")]
pub struct DoeModelStartersPath {
    pub organization_id: mestier_core::OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/doe-style/preview")]
pub struct DoeStylePreviewPath {
    pub organization_id: mestier_core::OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/doe-style/preview/pages")]
pub struct DoeStylePreviewPagesPath {
    pub organization_id: mestier_core::OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/doe-settings")]
pub struct DoeSettingsPath {
    pub organization_id: mestier_core::OrganizationId,
}

pub(crate) async fn require_doe_model(
    state: &AppState,
    identity: &Identity,
    organization_id: mestier_core::OrganizationId,
    doe_model_id: DoeModelId,
) -> Result<DoeModel, ApiError> {
    require_org_membership(state, identity, organization_id).await?;

    let model = state.usecase.get_doe_model(doe_model_id).await?;
    if model.organization_id != organization_id {
        return Err(ApiError::NotFound);
    }

    Ok(model)
}
