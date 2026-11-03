use chrono::{DateTime, Utc};
use mestier_core::{
    Attachment, AttachmentId, Badge, BadgeTone, EntryKind, MemberId, PresignedUrl, Ticket,
    TicketEntry, TicketEntryId, TicketId, TicketNumber, TicketStatus,
};
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

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct AttachmentResponse {
    pub id: AttachmentId,
    pub file_name: String,
    pub mime_type: String,
    pub size_bytes: u64,
    pub created_at: DateTime<Utc>,
}

impl From<Attachment> for AttachmentResponse {
    fn from(attachment: Attachment) -> Self {
        Self {
            id: attachment.id,
            file_name: attachment.file_name,
            mime_type: attachment.mime_type,
            size_bytes: attachment.size_bytes,
            created_at: attachment.created_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct TicketEntryResponse {
    pub id: TicketEntryId,
    pub ticket_id: TicketId,
    pub kind: EntryKind,
    pub author_member_id: Option<MemberId>,
    pub body: String,
    pub attachments: Vec<AttachmentResponse>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl From<TicketEntry> for TicketEntryResponse {
    fn from(entry: TicketEntry) -> Self {
        Self {
            id: entry.id,
            ticket_id: entry.ticket_id,
            kind: entry.kind,
            author_member_id: entry.author,
            body: entry.body,
            attachments: entry
                .attachments
                .into_iter()
                .map(AttachmentResponse::from)
                .collect(),
            created_at: entry.created_at,
            updated_at: entry.updated_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct AttachmentUrlResponse {
    pub url: String,
    pub expires_at: DateTime<Utc>,
}

impl From<PresignedUrl> for AttachmentUrlResponse {
    fn from(presigned: PresignedUrl) -> Self {
        Self {
            url: presigned.url,
            expires_at: presigned.expires_at,
        }
    }
}

#[derive(Debug, PartialEq, Serialize)]
pub struct EmptyResponse;
