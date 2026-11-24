use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response, resolve_actor};
use mestier_core::PlaceDafPhotoCommand;

use crate::daf::{
    DafPlacePhotoPath, request::PlaceDafPhotoRequest, require_daf, response::DafResponse,
};

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/dafs/{daf_id}/photo/place",
    operation_id = "placeDafPhoto",
    tag = super::super::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("daf_id" = mestier_core::DafId, Path, description = "DAF identifier"),
    ),
    request_body = PlaceDafPhotoRequest,
    responses(
        (status = 200, description = "The DAF, with a copy of the library photo as its image", body = inline(DataEnvelope<DafResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "DAF or photo not found, or the photo file is missing"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    DafPlacePhotoPath {
        organization_id,
        daf_id,
    }: DafPlacePhotoPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<PlaceDafPhotoRequest>,
) -> Result<Response<DafResponse>, ApiError> {
    require_daf(&state, &identity, organization_id, daf_id).await?;
    let (_, actor) = resolve_actor(&state, &identity).await?;

    let daf = state
        .usecase
        .place_daf_photo(
            PlaceDafPhotoCommand {
                actor,
                organization_id,
                id: daf_id,
                photo_id: payload.photo_id,
            },
            &state.file_storage,
        )
        .await?;

    Ok(Response::OK(daf.into()))
}
