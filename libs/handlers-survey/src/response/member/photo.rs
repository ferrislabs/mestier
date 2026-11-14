use auth::Identity;
use axum::{Extension, body::Bytes, extract::State};
use handlers::{AppState, Response, resolve_actor};
use mestier_core::UploadMemberResponsePhotoCommand;

use crate::{
    require_org_membership,
    response::{
        member::MemberResponsePhotosPath,
        photo::{PhotoError, UploadedSurveyPhoto, normalized, within_size},
    },
};

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/member-survey-responses/{response_id}/photos",
    operation_id = "uploadMemberSurveyPhoto",
    tag = super::super::super::TAG,
    description = "Uploads one photo for the member's response in progress. The body is the raw image: JPEG, PNG or WebP, detected from its content, at most the configured upload size. The image is normalized like the photo library (EXIF rotation applied, at most 2400 px on its longest side, metadata and GPS stripped, JPEG quality 85) and stored under a key of this response; the key is returned and is what a photo answer lists. A response accepts at most 20 uploads.",
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("response_id" = mestier_core::ResponseId, Path, description = "Response identifier"),
    ),
    request_body(content = Vec<u8>, content_type = "image/jpeg"),
    responses(
        (status = 201, description = "The photo, normalized and stored; its key goes in the photo answer", body = inline(handlers::DataEnvelope<UploadedSurveyPhoto>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Not a member of the organization"),
        (status = 404, description = "No response of this member with this id, or surveys not enabled"),
        (status = 409, description = "Closed, outside its campaign window, already complete, or 20 photos already uploaded (details.reason = response_photo_cap_reached)"),
        (status = 413, description = "The file exceeds the maximum upload size"),
        (status = 422, description = "Not a JPEG, PNG or WebP image, or empty, or corrupt"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    MemberResponsePhotosPath {
        organization_id,
        response_id,
    }: MemberResponsePhotosPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    body: Bytes,
) -> Result<Response<UploadedSurveyPhoto>, PhotoError> {
    require_org_membership(&state, &identity, organization_id).await?;
    within_size(&body, state.args.file_storage.max_upload_bytes)?;
    let image = normalized(body).await?;
    let (user_id, actor) = resolve_actor(&state, &identity).await?;

    let usecase = state.usecase.acting_as(user_id);
    let key = Box::pin(usecase.upload_member_response_photo(
        UploadMemberResponsePhotoCommand {
            actor,
            organization_id,
            id: response_id,
            image,
        },
        &state.file_storage,
    ))
    .await?;

    Ok(Response::Created(UploadedSurveyPhoto { key }))
}
