use axum::{
    body::Bytes,
    extract::{State, rejection::PathRejection},
};
use handlers::{ApiError, AppState, Response};

use crate::response::{
    photo::{PhotoError, UploadedSurveyPhoto, Uploader, checked_then_normalized, within_size},
    public::{PublicResponsePhotosPath, hidden, locate},
};

#[utoipa::path(
    post,
    path = "/api/v1/public/surveys/{slug}/responses/{resume_token}/photos",
    operation_id = "uploadPublicSurveyPhoto",
    tag = super::super::super::TAG,
    description = "Uploads one photo for a public response in progress, without an account. The body is the raw image: JPEG, PNG or WebP, detected from its content (HEIC is converted in the browser), at most the configured upload size. The image is normalized like the photo library (EXIF rotation applied, at most 2400 px on its longest side, metadata and GPS stripped, JPEG quality 85) and stored under a key of this response; the key is returned and is what a photo answer lists. A response accepts at most 20 uploads. An image over 50 megapixels or 12000 px on a side is refused with 422 before it is decoded. Rate limited per client address (30 uploads a minute) and per survey (300 uploads a minute).",
    params(
        ("slug" = String, Path, description = "The public slug of the survey"),
        ("resume_token" = String, Path, description = "The resume token returned when the response was started"),
    ),
    request_body(content = Vec<u8>, content_type = "image/jpeg"),
    responses(
        (status = 201, description = "The photo, normalized and stored; its key goes in the photo answer", body = inline(handlers::DataEnvelope<UploadedSurveyPhoto>)),
        (status = 404, description = "No such survey or response"),
        (status = 409, description = "Closed, outside its campaign window, already complete, or 20 photos already uploaded (details.reason = response_photo_cap_reached)"),
        (status = 413, description = "The file exceeds the maximum upload size"),
        (status = 422, description = "Not a JPEG, PNG or WebP image, or empty, or corrupt"),
        (status = 429, description = "Too many requests"),
        (status = 503, description = "Too many photos are being processed; retry after the Retry-After delay"),
    ),
)]
pub async fn handler(
    path: Result<PublicResponsePhotosPath, PathRejection>,
    State(state): State<AppState>,
    body: Bytes,
) -> Result<Response<UploadedSurveyPhoto>, PhotoError> {
    let PublicResponsePhotosPath { slug, resume_token } = path.map_err(|_| ApiError::NotFound)?;
    let (slug, token) = locate(slug, resume_token)?;
    within_size(&body, state.args.file_storage.max_upload_bytes)?;
    let check = async {
        state
            .usecase
            .check_public_response_photo(&slug, token.clone())
            .await
            .map_err(|error| PhotoError::from(hidden(error)))
    };
    let image = checked_then_normalized(check, body, Uploader::Public).await?;
    let key = Box::pin(state.usecase.upload_public_response_photo(
        &slug,
        token,
        image,
        &state.file_storage,
    ))
    .await
    .map_err(hidden)?;

    Ok(Response::Created(UploadedSurveyPhoto { key }))
}
