use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, require_org_membership, resolve_actor};
use mestier_core::GetLibraryPhotoCommand;
use serde::Serialize;
use utoipa::ToSchema;

use crate::photo_library::CopyPhotoPath;

#[derive(Debug, PartialEq, Serialize, ToSchema)]
pub struct CopiedPhotoResponse {
    /// Storage key of the copy, usable as an image of a DOE style.
    pub key: String,
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/photo-library/photos/{photo_id}/copy",
    operation_id = "copyLibraryPhoto",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("photo_id" = mestier_core::LibraryPhotoId, Path, description = "Photo identifier"),
    ),
    responses(
        (status = 200, description = "A copy of the photo file, independent of the library entry", body = inline(DataEnvelope<CopiedPhotoResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Photo not found, held by another organization, or its file is missing"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    CopyPhotoPath {
        organization_id,
        photo_id,
    }: CopyPhotoPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<CopiedPhotoResponse>, ApiError> {
    require_org_membership(&state, &identity, organization_id).await?;
    let (_, actor) = resolve_actor(&state, &identity).await?;

    let key = state
        .usecase
        .copy_library_photo(
            GetLibraryPhotoCommand {
                actor,
                organization_id,
                id: photo_id,
            },
            &state.file_storage,
        )
        .await?;

    Ok(Response::OK(CopiedPhotoResponse { key }))
}
