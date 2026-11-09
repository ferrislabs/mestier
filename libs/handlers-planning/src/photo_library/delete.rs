use auth::Identity;
use axum::{Extension, extract::State, http::StatusCode};
use handlers::{ApiError, AppState, require_org_membership, resolve_actor};
use mestier_core::DeleteLibraryPhotoCommand;

use crate::photo_library::PhotoPath;

#[utoipa::path(
    delete,
    path = "/api/v1/organizations/{organization_id}/photo-library/photos/{photo_id}",
    operation_id = "deleteLibraryPhoto",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("photo_id" = mestier_core::LibraryPhotoId, Path, description = "Photo identifier"),
    ),
    responses(
        (status = 204, description = "Photo and its two stored objects deleted; no DOE is touched"),
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
) -> Result<StatusCode, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let removed = state
        .usecase
        .acting_as(user_id)
        .delete_library_photo(DeleteLibraryPhotoCommand {
            actor,
            organization_id,
            id: photo_id,
        })
        .await?;

    for key in [&removed.file_key, &removed.thumbnail_key] {
        if let Err(error) = state.file_storage.delete(key).await {
            tracing::warn!(key, error = %error, "could not remove a deleted library object");
        }
    }

    Ok(StatusCode::NO_CONTENT)
}
