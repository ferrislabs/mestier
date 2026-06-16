use axum::{body::Bytes, extract::State, http::HeaderMap};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use http::header::CONTENT_TYPE;
use mestier_core::UploadFileCommand;

use crate::{paths::FilesPath, response::FileUploadResponse};

#[utoipa::path(
    post,
    path = "/api/v1/files",
    operation_id = "uploadFile",
    tag = super::TAG,
    request_body(content = Vec<u8>, content_type = "application/octet-stream"),
    responses(
        (status = 201, description = "File uploaded", body = inline(DataEnvelope<FileUploadResponse>)),
        (status = 400, description = "Validation failed"),
        (status = 401, description = "Unauthorized"),
        (status = 413, description = "Payload too large"),
        (status = 500, description = "File storage error"),
    ),
    security(("bearer_auth" = []))
)]
#[tracing::instrument(skip_all, err)]
pub async fn handler(
    _: FilesPath,
    State(state): State<AppState>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response<FileUploadResponse>, ApiError> {
    if body.is_empty() {
        return Err(ApiError::Validation("file body cannot be empty".to_owned()));
    }

    if body.len() as u64 > state.args.file_storage.max_upload_bytes {
        return Err(ApiError::UnprocessableEntity(format!(
            "file exceeds max upload size of {} bytes",
            state.args.file_storage.max_upload_bytes
        )));
    }

    let mime_type = headers
        .get(CONTENT_TYPE)
        .and_then(|value| value.to_str().ok())
        .unwrap_or("application/octet-stream")
        .to_owned();

    let stored = state
        .file_storage
        .upload(UploadFileCommand {
            mime_type,
            bytes: body.to_vec(),
        })
        .await?;

    Ok(Response::Created(stored.into()))
}
