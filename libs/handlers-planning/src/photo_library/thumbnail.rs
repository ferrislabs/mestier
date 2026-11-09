use auth::Identity;
use axum::{Extension, body::Body, extract::State, response::IntoResponse};
use handlers::{ApiError, AppState, require_org_membership, resolve_actor};
use http::{
    StatusCode,
    header::{CACHE_CONTROL, CONTENT_TYPE},
};
use mestier_core::GetLibraryPhotoCommand;

use crate::photo_library::ThumbnailPath;

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/photo-library/photos/{photo_id}/thumbnail",
    operation_id = "getLibraryPhotoThumbnail",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("photo_id" = mestier_core::LibraryPhotoId, Path, description = "Photo identifier"),
    ),
    responses(
        (status = 200, description = "The thumbnail, a JPEG whose longest side is 400 px", content_type = "image/jpeg"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Photo not found, or held by another organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    ThumbnailPath {
        organization_id,
        photo_id,
    }: ThumbnailPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<impl IntoResponse, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (_, actor) = resolve_actor(&state, &identity).await?;

    let photo = state
        .usecase
        .get_library_photo(GetLibraryPhotoCommand {
            actor,
            organization_id,
            id: photo_id,
        })
        .await?;
    let thumbnail = state.file_storage.get(&photo.thumbnail_key).await?;

    Ok((
        StatusCode::OK,
        [
            (CONTENT_TYPE, "image/jpeg"),
            (CACHE_CONTROL, "private, max-age=3600"),
        ],
        Body::from(thumbnail.bytes),
    ))
}
