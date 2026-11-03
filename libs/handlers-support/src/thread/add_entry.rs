use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::AddEntryCommand;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::TicketEntriesPath, response::TicketEntryResponse, ticket::require_manager};

#[derive(Debug, Deserialize, ToSchema)]
pub struct AddTicketEntryRequest {
    #[schema(example = "note")]
    pub kind: String,
    #[schema(example = "Called the vendor, a technician comes on Monday")]
    pub body: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum NewEntryKind {
    Note,
    Draft,
    Reply,
}

impl NewEntryKind {
    fn parse(value: &str) -> Result<Self, ApiError> {
        match value {
            "note" => Ok(Self::Note),
            "draft" => Ok(Self::Draft),
            "reply" => Ok(Self::Reply),
            _ => Err(ApiError::UnprocessableEntity(format!(
                "unknown entry kind `{value}`"
            ))),
        }
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/support/tickets/{ticket_id}/entries",
    operation_id = "addSupportTicketEntry",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("ticket_id" = mestier_core::TicketId, Path, description = "Ticket identifier"),
    ),
    request_body = AddTicketEntryRequest,
    responses(
        (status = 201, description = "The new entry: an internal note, a draft, or a reply visible to the client", body = inline(DataEnvelope<TicketEntryResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Ticket not found in the organization"),
        (status = 409, description = "Blank body"),
        (status = 422, description = "Unknown kind; the values are note, draft and reply"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    TicketEntriesPath {
        organization_id,
        ticket_id,
    }: TicketEntriesPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<AddTicketEntryRequest>,
) -> Result<Response<TicketEntryResponse>, ApiError> {
    let actor = require_manager(&state, &identity, organization_id).await?;
    let kind = NewEntryKind::parse(&payload.kind)?;

    let command = AddEntryCommand {
        actor,
        organization_id,
        ticket_id,
        body: payload.body,
    };
    let entry = match kind {
        NewEntryKind::Note => state.usecase.add_support_internal_note(command).await?,
        NewEntryKind::Draft => state.usecase.save_support_draft(command).await?,
        NewEntryKind::Reply => state.usecase.send_support_reply(command).await?,
    };

    Ok(Response::Created(entry.into()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_three_writable_kinds_are_accepted() {
        assert_eq!(NewEntryKind::parse("note").unwrap(), NewEntryKind::Note);
        assert_eq!(NewEntryKind::parse("draft").unwrap(), NewEntryKind::Draft);
        assert_eq!(NewEntryKind::parse("reply").unwrap(), NewEntryKind::Reply);
    }

    #[test]
    fn stored_kinds_other_than_the_three_are_refused() {
        for value in ["client_message", "internal_note", "Note", ""] {
            assert!(matches!(
                NewEntryKind::parse(value),
                Err(ApiError::UnprocessableEntity(_))
            ));
        }
    }
}
