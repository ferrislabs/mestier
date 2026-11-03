use auth::Identity;
use axum::{Extension, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::GetTicketCommand;

use crate::{paths::TicketPath, require_view_support, response::TicketResponse};

#[utoipa::path(
    get,
    path = "/api/v1/organizations/{organization_id}/support/tickets/{ticket_id}",
    operation_id = "getSupportTicket",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("ticket_id" = mestier_core::TicketId, Path, description = "Ticket identifier"),
    ),
    responses(
        (status = 200, description = "The ticket", body = inline(DataEnvelope<TicketResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Ticket not found in the organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    TicketPath {
        organization_id,
        ticket_id,
    }: TicketPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
) -> Result<Response<TicketResponse>, ApiError> {
    require_view_support(&state, &identity, organization_id).await?;

    let ticket = state
        .usecase
        .get_support_ticket(GetTicketCommand {
            organization_id,
            ticket_id,
        })
        .await?;

    Ok(Response::OK(ticket.into()))
}
