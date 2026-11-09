use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, require_org_membership, resolve_actor};
use mestier_core::GetLibraryPhotoCommand;

use crate::photo_library::{PhotoPath, response::LibraryPhotoResponse};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/photo-library/photos/{photo_id}",
    operation_id = "getLibraryPhoto",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("photo_id" = mestier_core::LibraryPhotoId, Path, description = "Photo identifier"),
    ),
    responses(
        (status = 200, description = "The photo", body = inline(DataEnvelope<LibraryPhotoResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Photo not found, or held by another organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    PhotoPath {
        organization_id,
        photo_id,
    }: PhotoPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<LibraryPhotoResponse>, ApiError> {
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

    Ok(Response::OK(photo.into()))
}
