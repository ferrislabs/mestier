use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::RemoveBadgeCommand;

use crate::{paths::TicketBadgePath, response::TicketResponse, ticket::require_manager};

#[utoipa::path(
    delete,
    path = "/api/v1/organizations/{organization_id}/support/tickets/{ticket_id}/badges/{label}",
    operation_id = "removeSupportTicketBadge",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("ticket_id" = mestier_core::TicketId, Path, description = "Ticket identifier"),
        ("label" = String, Path, description = "Badge label, percent-encoded"),
    ),
    responses(
        (status = 200, description = "The ticket without the badge; unchanged when it carried none with that label", body = inline(DataEnvelope<TicketResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Ticket not found in the organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    TicketBadgePath {
        organization_id,
        ticket_id,
        label,
    }: TicketBadgePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<TicketResponse>, ApiError> {
    let actor = require_manager(&state, &identity, organization_id).await?;

    let ticket = state
        .usecase
        .remove_support_ticket_badge(RemoveBadgeCommand {
            actor,
            organization_id,
            ticket_id,
            label,
        })
        .await?;

    Ok(Response::OK(ticket.into()))
}
