use auth::Identity;
use axum::{
    Extension,
    body::Bytes,
    extract::State,
    http::{HeaderMap, header::CONTENT_TYPE},
};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{AttachFileCommand, UploadFileCommand};
use percent_encoding::percent_decode_str;

use crate::{paths::EntryAttachmentsPath, response::AttachmentResponse, ticket::require_manager};

const FILE_NAME_HEADER: &str = "x-file-name";
pub(crate) const UPLOAD_FOLDER: &str = "support-tickets";

pub(crate) fn file_name(headers: &HeaderMap) -> Result<String, ApiError> {
    let value = headers
        .get(FILE_NAME_HEADER)
        .ok_or_else(|| ApiError::Validation("the X-File-Name header is required".to_owned()))?;
    let raw = std::str::from_utf8(value.as_bytes())
        .map_err(|_| ApiError::Validation("the X-File-Name header must be UTF-8".to_owned()))?;
    let decoded = percent_decode_str(raw).decode_utf8().map_err(|_| {
        ApiError::Validation("the X-File-Name header must percent-encode UTF-8".to_owned())
    })?;
    let name = decoded.trim();

    if name.is_empty() {
        return Err(ApiError::Validation(
            "the X-File-Name header must not be blank".to_owned(),
        ));
    }

    Ok(name.to_owned())
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/support/entries/{entry_id}/attachments",
    operation_id = "attachSupportEntryFile",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("entry_id" = mestier_core::TicketEntryId, Path, description = "Entry identifier"),
        ("X-File-Name" = String, Header, description = "Original name of the file, percent-encoded UTF-8, shown to readers of the thread"),
    ),
    request_body(content = Vec<u8>, content_type = "application/octet-stream"),
    responses(
        (status = 201, description = "The file is stored and attached to the entry", body = inline(DataEnvelope<AttachmentResponse>)),
        (status = 400, description = "Missing or blank X-File-Name header, or empty body"),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Entry not found in the organization"),
        (status = 409, description = "The entry is a client message"),
        (status = 413, description = "Payload too large"),
        (status = 422, description = "File exceeds the maximum upload size"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    EntryAttachmentsPath {
        organization_id,
        entry_id,
    }: EntryAttachmentsPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response<AttachmentResponse>, ApiError> {
    let actor = require_manager(&state, &identity, organization_id).await?;
    let file_name = file_name(&headers)?;

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
            folder: Some(UPLOAD_FOLDER.to_owned()),
        })
        .await?;

    let attached = state
        .usecase
        .attach_to_support_entry(AttachFileCommand {
            actor,
            organization_id,
            entry_id,
            file_key: stored.key.clone(),
            file_name,
            mime_type: stored.mime_type,
            size_bytes: stored.size_bytes,
        })
        .await;

    match attached {
        Ok(attachment) => Ok(Response::Created(attachment.into())),
        Err(error) => {
            if let Err(cleanup) = state.file_storage.delete(&stored.key).await {
                tracing::warn!(key = %stored.key, error = %cleanup, "could not delete the object of a refused support attachment");
            }

            Err(error.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use axum::http::HeaderValue;

    use super::*;

    fn headers(value: Option<&[u8]>) -> HeaderMap {
        let mut headers = HeaderMap::new();
        if let Some(value) = value {
            headers.insert(FILE_NAME_HEADER, HeaderValue::from_bytes(value).unwrap());
        }

        headers
    }

    #[test]
    fn the_file_name_is_trimmed() {
        assert_eq!(
            file_name(&headers(Some(b"  quote.pdf "))).unwrap(),
            "quote.pdf"
        );
    }

    #[test]
    fn a_non_ascii_utf8_file_name_is_kept() {
        assert_eq!(
            file_name(&headers(Some("devis é.pdf".as_bytes()))).unwrap(),
            "devis é.pdf"
        );
    }

    #[test]
    fn a_missing_blank_or_non_utf8_file_name_is_refused() {
        for headers in [
            headers(None),
            headers(Some(b"   ")),
            headers(Some(&[0xff, 0xfe])),
        ] {
            assert!(matches!(file_name(&headers), Err(ApiError::Validation(_))));
        }
    }
}
