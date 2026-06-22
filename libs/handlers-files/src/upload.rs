use axum::extract::Query;
use axum::{body::Bytes, extract::State, http::HeaderMap};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use http::header::CONTENT_TYPE;
use mestier_core::UploadFileCommand;

use crate::{paths::FilesPath, response::FileUploadResponse};

/// Optional `?folder=<slug>` query parameter for `POST /api/v1/files`.
/// Accepted slug pattern: `^[a-z0-9_-]+$` (no slashes, no dots, no traversal).
#[derive(Debug, serde::Deserialize)]
pub struct UploadFolderQuery {
    pub folder: Option<String>,
}

impl UploadFolderQuery {
    /// Returns `Ok(None)` when folder is absent; `Ok(Some(slug))` for a valid slug;
    /// `Err(ApiError::Validation)` for an invalid value.
    pub fn validated_folder(&self) -> Result<Option<String>, ApiError> {
        match &self.folder {
            None => Ok(None),
            Some(slug) => {
                let valid = !slug.is_empty()
                    && slug.chars().all(|c| {
                        c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_' || c == '-'
                    });
                if valid {
                    Ok(Some(slug.clone()))
                } else {
                    Err(ApiError::Validation(format!(
                        "folder must match ^[a-z0-9_-]+$ (got: {slug:?})"
                    )))
                }
            }
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/files",
    operation_id = "uploadFile",
    tag = super::TAG,
    params(
        ("folder" = Option<String>, Query, description = "Sub-folder for the stored object (e.g. \"attachments\")"),
    ),
    request_body(content = Vec<u8>, content_type = "application/octet-stream"),
    responses(
        (status = 201, description = "File uploaded", body = inline(DataEnvelope<FileUploadResponse>)),
        (status = 400, description = "Validation failed — empty body or invalid folder slug"),
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
    Query(folder_query): Query<UploadFolderQuery>,
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

    let folder = folder_query.validated_folder()?;

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
            folder,
        })
        .await?;

    Ok(Response::Created(stored.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folder_slug_with_slash_is_invalid() {
        let q = UploadFolderQuery {
            folder: Some("../../etc".to_owned()),
        };
        assert!(q.validated_folder().is_err());
    }

    #[test]
    fn folder_slug_attachments_is_valid() {
        let q = UploadFolderQuery {
            folder: Some("attachments".to_owned()),
        };
        assert!(q.validated_folder().is_ok());
    }

    #[test]
    fn folder_none_is_valid() {
        let q = UploadFolderQuery { folder: None };
        assert_eq!(q.validated_folder().unwrap(), None);
    }
}
