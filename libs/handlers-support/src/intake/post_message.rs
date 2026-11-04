use axum::{Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::PostClientMessageCommand;

use crate::{
    intake::{
        IntakeOrganization, storable,
        types::{IntakeEntryResponse, PostIntakeMessageRequest},
        validation,
    },
    paths::IntakeMessagesPath,
};

#[utoipa::path(
    post,
    path = "/api/v1/support/intake/tickets/{ticket_id}/messages",
    operation_id = "postIntakeMessage",
    tag = crate::TAG,
    description = "Posts a message from the customer. A ticket that was waiting for the customer or resolved is reopened as in_progress.",
    params(
        ("ticket_id" = mestier_core::TicketId, Path, description = "Ticket identifier"),
    ),
    request_body = PostIntakeMessageRequest,
    responses(
        (status = 201, description = "The new client message; its id is the entry_id that attachIntakeFile expects", body = inline(DataEnvelope<IntakeEntryResponse>)),
        (status = 401, description = "Missing, malformed, unknown or revoked API key"),
        (status = 404, description = "No such ticket in the organization of the key"),
        (status = 422, description = "Blank body"),
        (status = 429, description = "Too many requests"),
    ),
    security(("intake_key" = []))
)]
pub async fn handler(
    IntakeMessagesPath { ticket_id }: IntakeMessagesPath,
    IntakeOrganization(organization_id): IntakeOrganization,
    State(state): State<AppState>,
    Json(payload): Json<PostIntakeMessageRequest>,
) -> Result<Response<IntakeEntryResponse>, ApiError> {
    storable(&payload.body, "body")?;

    let entry = state
        .usecase
        .post_support_client_message(PostClientMessageCommand {
            organization_id,
            ticket_id,
            body: payload.body,
        })
        .await
        .map_err(validation)?;

    let response = IntakeEntryResponse::try_from(entry).map_err(|_| ApiError::Internal)?;

    Ok(Response::Created(response))
}
