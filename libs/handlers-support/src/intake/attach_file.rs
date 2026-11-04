use axum::{
    body::Bytes,
    extract::{Query, State},
    http::{HeaderMap, Uri, header::CONTENT_TYPE},
};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{
    EntryKind, NewAttachment, OrganizationId, TicketEntryId, TicketId, UploadFileCommand,
};
use serde::Deserialize;
use utoipa::IntoParams;

use crate::{
    intake::{
        IntakeOrganization, client_visible_thread, storable,
        types::IntakeAttachmentResponse,
        upload::{displayable_file_name, intake_mime_type},
    },
    paths::IntakeAttachmentsPath,
    thread::attach::{UPLOAD_FOLDER, file_name},
};

#[derive(Debug, Deserialize, IntoParams)]
#[into_params(parameter_in = Query)]
pub struct AttachQuery {
    pub entry_id: TicketEntryId,
}

fn entry_id(uri: &Uri) -> Result<TicketEntryId, ApiError> {
    Query::<AttachQuery>::try_from_uri(uri)
        .map(|Query(query)| query.entry_id)
        .map_err(|_| {
            ApiError::UnprocessableEntity(
                "the entry_id query parameter is required and must be a ticket entry id".to_owned(),
            )
        })
}

async fn require_client_message(
    state: &AppState,
    organization_id: OrganizationId,
    ticket_id: TicketId,
    entry_id: TicketEntryId,
) -> Result<(), ApiError> {
    client_visible_thread(state, organization_id, ticket_id)
        .await?
        .iter()
        .find(|entry| entry.id == entry_id && entry.kind == EntryKind::ClientMessage)
        .map(|_| ())
        .ok_or(ApiError::NotFound)
}

#[utoipa::path(
    post,
    path = "/api/v1/support/intake/tickets/{ticket_id}/attachments",
    operation_id = "attachIntakeFile",
    tag = crate::TAG,
    description = "Attaches a file, sent as the raw request body, to a client message of the ticket. The entry_id is the id returned by postIntakeMessage. The stored mime type is the Content-Type when it is one of image/png, image/jpeg, image/gif, image/webp, application/pdf, text/plain, text/csv or application/json, and application/octet-stream otherwise.",
    params(
        ("ticket_id" = mestier_core::TicketId, Path, description = "Ticket identifier"),
        AttachQuery,
        ("X-File-Name" = String, Header, description = "Original name of the file, percent-encoded UTF-8"),
    ),
    request_body(content = Vec<u8>, content_type = "application/octet-stream"),
    responses(
        (status = 201, description = "The file is stored and attached to the client message", body = inline(DataEnvelope<IntakeAttachmentResponse>)),
        (status = 400, description = "Missing or blank X-File-Name header, or empty body"),
        (status = 401, description = "Missing, malformed, unknown or revoked API key"),
        (status = 404, description = "The entry is not a client message of that ticket in the organization of the key"),
        (status = 413, description = "Payload too large"),
        (status = 422, description = "Missing or malformed entry_id, file name longer than 255 characters or holding a control or bidirectional formatting character, or file exceeds the maximum upload size"),
        (status = 429, description = "Too many requests"),
    ),
    security(("intake_key" = []))
)]
pub async fn handler(
    IntakeAttachmentsPath { ticket_id }: IntakeAttachmentsPath,
    IntakeOrganization(organization_id): IntakeOrganization,
    State(state): State<AppState>,
    uri: Uri,
    headers: HeaderMap,
    body: Bytes,
) -> Result<Response<IntakeAttachmentResponse>, ApiError> {
    let entry_id = entry_id(&uri)?;
    let file_name = file_name(&headers)?;
    storable(&file_name, "X-File-Name")?;
    displayable_file_name(&file_name)?;

    if body.is_empty() {
        return Err(ApiError::Validation("file body cannot be empty".to_owned()));
    }

    if body.len() as u64 > state.args.file_storage.max_upload_bytes {
        return Err(ApiError::UnprocessableEntity(format!(
            "file exceeds max upload size of {} bytes",
            state.args.file_storage.max_upload_bytes
        )));
    }

    require_client_message(&state, organization_id, ticket_id, entry_id).await?;

    let mime_type = intake_mime_type(
        headers
            .get(CONTENT_TYPE)
            .and_then(|value| value.to_str().ok()),
    )
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
        .attach_to_support_client_message(
            organization_id,
            entry_id,
            NewAttachment {
                file_key: stored.key.clone(),
                file_name,
                mime_type: stored.mime_type,
                size_bytes: stored.size_bytes,
            },
        )
        .await;

    match attached {
        Ok(attachment) => Ok(Response::Created(attachment.into())),
        Err(error) => {
            if state.file_storage.delete(&stored.key).await.is_err() {
                tracing::warn!("could not delete the object of a refused intake attachment");
            }

            Err(error.into())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const ENTRY: &str = "0193c8a0-7b1e-7c5a-9d3e-2f4a6b8c0d1e";

    fn uri(query: &str) -> Uri {
        format!("/api/v1/support/intake/tickets/x/attachments{query}")
            .parse()
            .unwrap()
    }

    #[test]
    fn a_valid_entry_id_is_read() {
        let parsed = entry_id(&uri(&format!("?entry_id={ENTRY}"))).unwrap();

        assert_eq!(parsed.to_string(), ENTRY);
    }

    #[test]
    fn a_missing_blank_malformed_or_repeated_entry_id_is_unprocessable() {
        for query in [
            String::new(),
            "?other=1".to_owned(),
            "?entry_id=".to_owned(),
            "?entry_id=not-a-uuid".to_owned(),
            format!("?entry_id={ENTRY}&entry_id={ENTRY}"),
        ] {
            assert!(
                matches!(
                    entry_id(&uri(&query)),
                    Err(ApiError::UnprocessableEntity(_))
                ),
                "{query}"
            );
        }
    }
}
