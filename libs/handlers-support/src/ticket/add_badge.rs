use std::str::FromStr;

use auth::Identity;
use axum::{Extension, Json, extract::State};
use handlers::{ApiError, AppState, DataEnvelope, Response};
use mestier_core::{AddBadgeCommand, Badge, BadgeTone};
use serde::Deserialize;
use utoipa::ToSchema;

use crate::{paths::TicketBadgesPath, response::TicketResponse, ticket::require_manager};

#[derive(Debug, Deserialize, ToSchema)]
pub struct AddTicketBadgeRequest {
    #[schema(example = "VIP")]
    pub label: String,
    #[serde(default)]
    #[schema(example = "brand")]
    pub tone: Option<String>,
}

fn parse_tone(value: Option<&str>) -> Result<BadgeTone, ApiError> {
    match value {
        None => Ok(BadgeTone::default()),
        Some(value) => BadgeTone::from_str(value)
            .map_err(|_| ApiError::UnprocessableEntity(format!("unknown badge tone `{value}`"))),
    }
}

#[utoipa::path(
    post,
    path = "/api/v1/organizations/{organization_id}/support/tickets/{ticket_id}/badges",
    operation_id = "addSupportTicketBadge",
    tag = crate::TAG,
    params(
        ("organization_id" = mestier_core::OrganizationId, Path, description = "Organization identifier"),
        ("ticket_id" = mestier_core::TicketId, Path, description = "Ticket identifier"),
    ),
    request_body = AddTicketBadgeRequest,
    responses(
        (status = 200, description = "The ticket with the badge; a badge with the same label has its tone replaced", body = inline(DataEnvelope<TicketResponse>)),
        (status = 401, description = "Unauthorized"),
        (status = 403, description = "Forbidden"),
        (status = 404, description = "Ticket not found in the organization"),
        (status = 409, description = "Blank or too long label"),
        (status = 422, description = "Unknown tone; the values are neutral, brand, success, warning and error"),
    ),
    security(("bearer_auth" = []))
)]
pub async fn handler(
    TicketBadgesPath {
        organization_id,
        ticket_id,
    }: TicketBadgesPath,
    State(state): State<AppState>,
    Extension(identity): Extension<Identity>,
    Json(payload): Json<AddTicketBadgeRequest>,
) -> Result<Response<TicketResponse>, ApiError> {
    let actor = require_manager(&state, &identity, organization_id).await?;
    let tone = parse_tone(payload.tone.as_deref())?;
    let badge = Badge::new(&payload.label, tone)?;

    let ticket = state
        .usecase
        .add_support_ticket_badge(AddBadgeCommand {
            actor,
            organization_id,
            ticket_id,
            badge,
        })
        .await?;

    Ok(Response::OK(ticket.into()))
}
