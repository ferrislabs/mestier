use axum::extract::State;
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::GetTicketCommand;

use crate::{
    intake::{IntakeOrganization, client_visible_thread, types::IntakeTicketResponse},
    paths::IntakeTicketPath,
};

#[utoipa::path(
    get,
    path = "/api/v1/support/intake/tickets/{ticket_id}",
    operation_id = "getIntakeTicket",
    tag = crate::TAG,
    description = "The ticket and its client-visible entries (client messages and replies) in creation order. Internal notes and drafts are never part of it.",
    params(
        ("ticket_id" = mestier_core::TicketId, Path, description = "Ticket identifier"),
    ),
    responses(
        (status = 200, description = "The ticket with its entries", body = inline(DataEnvelope<IntakeTicketResponse>)),
        (status = 401, description = "Missing, malformed, unknown or revoked API key"),
        (status = 404, description = "No such ticket in the organization of the key"),
        (status = 429, description = "Too many requests"),
    ),
    security(("intake_key" = []))
)]
pub async fn handler(
    IntakeTicketPath { ticket_id }: IntakeTicketPath,
    IntakeOrganization(organization_id): IntakeOrganization,
    State(state): State<AppState>,
) -> Result<Response<IntakeTicketResponse>, ApiError> {
    let ticket = state
        .usecase
        .get_support_ticket(GetTicketCommand {
            organization_id,
            ticket_id,
        })
        .await?;
    let entries = client_visible_thread(&state, organization_id, ticket_id).await?;

    Ok(Response::OK(
        IntakeTicketResponse::from(ticket).with_entries(entries),
    ))
}
