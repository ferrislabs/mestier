use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::{
    LibraryPhotoId, PlaceLibraryPhotosCommand, PlacementRequest, PlacementTargetKind,
};
use serde::Deserialize;
use utoipa::ToSchema;
use uuid::Uuid;

use crate::{
    doe::require_doe,
    photo_library::{PlacePhotosPath, response::PlacementReportResponse, unprocessable},
};

#[derive(Debug, Clone, Copy, Deserialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum PlacementTarget {
    Section,
    Supply,
    Cover,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PlacementItem {
    pub photo_id: LibraryPhotoId,
    pub target: PlacementTarget,
    /// For `section`: the photos section to fill. Optional when the DOE has a
    /// single photos section.
    #[serde(default)]
    pub section_id: Option<Uuid>,
    #[serde(default)]
    pub section_title: Option<String>,
    /// Required for `supply`.
    #[serde(default)]
    pub supply_id: Option<Uuid>,
    /// Defaults to the library caption.
    #[serde(default)]
    pub caption: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PlacePhotosRequest {
    #[schema(max_items = 50)]
    pub placements: Vec<PlacementItem>,
}

impl From<PlacementItem> for PlacementRequest {
    fn from(item: PlacementItem) -> Self {
        Self {
            photo_id: item.photo_id,
            target: match item.target {
                PlacementTarget::Section => PlacementTargetKind::Section,
                PlacementTarget::Supply => PlacementTargetKind::Supply,
                PlacementTarget::Cover => PlacementTargetKind::Cover,
            },
            section_id: item.section_id,
            section_title: item.section_title,
            supply_id: item.supply_id,
            caption: item.caption,
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/does/{doe_id}/photos/place",
    operation_id = "placeLibraryPhotosInDoe",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("doe_id" = mestier_core::DoeId, Path, description = "DOE identifier"),
    ),
    request_body = PlacePhotosRequest,
    responses(
        (status = 200, description = "The updated DOE and one result per placement; invalid elements are reported `failed` and the valid ones written in one go", body = inline(DataEnvelope<PlacementReportResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "DOE not found, or held by another organization"),
        (status = 422, description = "More than 50 placements"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    PlacePhotosPath {
        organization_id,
        doe_id,
    }: PlacePhotosPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<PlacePhotosRequest>,
) -> Result<Response<PlacementReportResponse>, ApiError> {
    require_doe(&state, &identity, organization_id, doe_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let report = state
        .usecase
        .acting_as(user_id)
        .place_library_photos(
            PlaceLibraryPhotosCommand {
                actor,
                organization_id,
                doe_id,
                placements: payload.placements.into_iter().map(Into::into).collect(),
            },
            &state.file_storage,
        )
        .await?
        .map_err(unprocessable)?;

    Ok(Response::OK(report.into()))
}
