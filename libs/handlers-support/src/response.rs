use chrono::{DateTime, Utc};
use mestier_core::{Badge, BadgeTone, MemberId, Ticket, TicketId, TicketNumber, TicketStatus};
use serde::Serialize;
use serde_json::Value;
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct BadgeResponse {
    pub label: String,
    pub tone: BadgeTone,
}

impl From<&Badge> for BadgeResponse {
    fn from(badge: &Badge) -> Self {
        Self {
            label: badge.label().to_owned(),
            tone: badge.tone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct TicketResponse {
    pub id: TicketId,
    pub number: TicketNumber,
    pub subject: String,
    pub body: String,
    pub status: TicketStatus,
    pub assignee_member_id: Option<MemberId>,
    pub assigned_at: Option<DateTime<Utc>>,
    #[schema(value_type = Object)]
    pub custom_fields: Value,
    pub badges: Vec<BadgeResponse>,
    pub external_id: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<Ticket> for TicketResponse {
    fn from(ticket: Ticket) -> Self {
        Self {
            id: ticket.id,
            number: ticket.number,
            subject: ticket.subject,
            body: ticket.body,
            status: ticket.status,
            assignee_member_id: ticket.assignee,
            assigned_at: ticket.assigned_at,
            custom_fields: ticket.custom_fields.into_value(),
            badges: ticket.badges.iter().map(BadgeResponse::from).collect(),
            external_id: ticket.external_id,
            created_at: ticket.created_at,
            updated_at: ticket.updated_at,
        }
    }
}
