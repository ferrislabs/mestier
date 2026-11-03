use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::TransitionTicketCommand;
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{
    paths::TicketStatusPath,
    response::TicketResponse,
    ticket::{parse_status, require_manager},
};

#[derive(Debug, Deserialize, ToSchema)]
pub struct TransitionTicketRequest {
    #[schema(example = "in_progress")]
    pub status: String,
}

#[utoipa::path(
    patch,
    path = "/api/v1/organizations/{organization_id}/support/tickets/{ticket_id}/status",
    operation_id = "transitionSupportTicket",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("ticket_id" = mestier_core::TicketId, Path, description = "Ticket identifier"),
    ),
    request_body = TransitionTicketRequest,
    responses(
        (status = 200, description = "The ticket in its new status", body = inline(DataEnvelope<TicketResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Ticket not found in the organization"),
        (status = 422, description = "Unknown status; the stored values are new, in_progress, waiting_customer and resolved"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    TicketStatusPath {
        organization_id,
        ticket_id,
    }: TicketStatusPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<TransitionTicketRequest>,
) -> Result<Response<TicketResponse>, ApiError> {
    let actor = require_manager(&state, &identity, organization_id).await?;
    let status = parse_status(&payload.status)?;

    let ticket = state
        .usecase
        .transition_support_ticket(TransitionTicketCommand {
            actor,
            organization_id,
            ticket_id,
            status,
        })
        .await?;

    Ok(Response::OK(ticket.into()))
}
