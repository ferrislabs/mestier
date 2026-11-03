use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{AssignTicketCommand, MemberId};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::TicketAssigneePath, response::TicketResponse, ticket::require_manager};

#[derive(Debug, Deserialize, ToSchema)]
pub struct AssignTicketRequest {
    pub member_id: Option<MemberId>,
}

#[utoipa::path(
    put,
    path = "/api/v1/organizations/{organization_id}/support/tickets/{ticket_id}/assignee",
    operation_id = "assignSupportTicket",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("ticket_id" = mestier_core::TicketId, Path, description = "Ticket identifier"),
    ),
    request_body = AssignTicketRequest,
    responses(
        (status = 200, description = "The ticket with its new assignee, or none when member_id is null", body = inline(DataEnvelope<TicketResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Ticket or member not found in the organization"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    TicketAssigneePath {
        organization_id,
        ticket_id,
    }: TicketAssigneePath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<AssignTicketRequest>,
) -> Result<Response<TicketResponse>, ApiError> {
    let actor = require_manager(&state, &identity, organization_id).await?;

    let ticket = state
        .usecase
        .assign_support_ticket(AssignTicketCommand {
            actor,
            organization_id,
            ticket_id,
            assignee: payload.member_id,
        })
        .await?;

    Ok(Response::OK(ticket.into()))
}
