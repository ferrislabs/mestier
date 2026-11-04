use axum::extract::State;
use handlers::{ApiError, AppState, DataEnvelope, Response};

use crate::{
    intake::{IntakeOrganization, client_visible_thread, types::IntakeAttachmentUrlResponse},
    paths::IntakeAttachmentUrlPath,
};

#[utoipa::path(
    get,
    path = "/api/v1/support/intake/tickets/{ticket_id}/attachments/{attachment_id}/url",
    operation_id = "getIntakeAttachmentUrl",
    tag = crate::TAG,
    description = "A short-lived presigned read url for an attachment of a client message or of a reply of that ticket.",
    params(
        ("ticket_id" = mestier_core::TicketId, Path, description = "Ticket identifier"),
        ("attachment_id" = mestier_core::AttachmentId, Path, description = "Attachment identifier"),
    ),
    responses(
        (status = 200, description = "A short-lived presigned read url", body = inline(DataEnvelope<IntakeAttachmentUrlResponse>)),
        (status = 401, description = "Missing, malformed, unknown or revoked API key"),
        (status = 404, description = "No such attachment among the client-visible entries of that ticket"),
        (status = 429, description = "Too many requests"),
    ),
    security(("intake_key" = []))
)]
pub async fn handler(
    IntakeAttachmentUrlPath {
        ticket_id,
        attachment_id,
    }: IntakeAttachmentUrlPath,
    IntakeOrganization(organization_id): IntakeOrganization,
    State(state): State<AppState>,
) -> Result<Response<IntakeAttachmentUrlResponse>, ApiError> {
    let entries = client_visible_thread(&state, organization_id, ticket_id).await?;
    let attachment = entries
        .iter()
        .flat_map(|entry| entry.attachments.iter())
        .find(|attachment| attachment.id == attachment_id)
        .ok_or(ApiError::NotFound)?;

    let presigned = state
        .file_storage
        .presigned_get_url(&attachment.file_key)
        .await?;

    Ok(Response::OK(presigned.into()))
}
