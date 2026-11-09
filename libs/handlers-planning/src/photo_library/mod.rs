use axum::{
    Router,
    extract::DefaultBodyLimit,
    routing::{get, post},
};
use axum_extra::routing::{RouterExt, TypedPath};
use handlers::{ApiError, AppState};
use mestier_core::{DoeId, LibraryPhotoId, OrganizationId, PhotoLibraryError};
use serde::Deserialize;

pub mod delete;
pub mod get_one;
pub mod list;
pub mod place;
pub mod response;
pub mod thumbnail;
pub mod update;
pub mod upload;

pub fn router(state: &AppState) -> Router<AppState> {
    let upload_limit = usize::try_from(state.args.file_storage.max_upload_bytes)
        .unwrap_or(usize::MAX)
        .saturating_add(1);

    Router::new()
        .route(
            PhotosPath::PATH,
            get(list::handler)
                .merge(post(upload::handler).layer(DefaultBodyLimit::max(upload_limit))),
        )
        .typed_get(get_one::handler)
        .typed_get(thumbnail::handler)
        .typed_patch(update::handler)
        .typed_delete(delete::handler)
        .typed_post(place::handler)
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/photo-library/photos")]
pub struct PhotosPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/photo-library/photos/{photo_id}")]
pub struct PhotoPath {
    pub organization_id: OrganizationId,
    pub photo_id: LibraryPhotoId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/photo-library/photos/{photo_id}/thumbnail")]
pub struct ThumbnailPath {
    pub organization_id: OrganizationId,
    pub photo_id: LibraryPhotoId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/does/{doe_id}/photos/place")]
pub struct PlacePhotosPath {
    pub organization_id: OrganizationId,
    pub doe_id: DoeId,
}

pub(crate) fn unprocessable(error: PhotoLibraryError) -> ApiError {
    ApiError::UnprocessableEntity(error.to_string())
}
