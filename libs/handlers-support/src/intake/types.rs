use chrono::{DateTime, Utc};
use mestier_core::{
    Attachment, AttachmentId, EntryKind, PresignedUrl, Ticket, TicketEntry, TicketEntryId,
    TicketId, TicketNumber, TicketStatus,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use utoipa::ToSchema;

use crate::response::BadgeResponse;

#[derive(Debug, Deserialize, ToSchema)]
pub struct CreateIntakeTicketRequest {
    #[schema(example = "Le déploiement échoue")]
    pub subject: String,
    #[schema(example = "Le déploiement dpl_8f2a91 échoue depuis ce matin.")]
    pub body: String,
    #[schema(value_type = Object, example = json!({ "plan": "pro", "seats": 12 }))]
    pub custom_fields: Value,
    #[schema(example = "autharie-ticket-4821", max_length = 255)]
    pub external_id: Option<String>,
}

#[derive(Debug, Deserialize, ToSchema)]
pub struct PostIntakeMessageRequest {
    #[schema(example = "Toujours en panne ce matin")]
    pub body: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct IntakeAttachmentResponse {
    pub id: AttachmentId,
    pub file_name: String,
    pub mime_type: String,
    pub size_bytes: u64,
    pub created_at: DateTime<Utc>,
}

impl From<Attachment> for IntakeAttachmentResponse {
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, ToSchema)]
#[serde(rename_all = "snake_case")]
pub enum IntakeEntryKind {
    ClientMessage,
    Reply,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct NotClientVisible;

impl TryFrom<EntryKind> for IntakeEntryKind {
    type Error = NotClientVisible;

    fn try_from(kind: EntryKind) -> Result<Self, Self::Error> {
        match kind {
            EntryKind::ClientMessage => Ok(Self::ClientMessage),
            EntryKind::Reply => Ok(Self::Reply),
            EntryKind::InternalNote | EntryKind::Draft => Err(NotClientVisible),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct IntakeEntryResponse {
    id: TicketEntryId,
    kind: IntakeEntryKind,
    body: String,
    attachments: Vec<IntakeAttachmentResponse>,
    created_at: DateTime<Utc>,
}

impl TryFrom<TicketEntry> for IntakeEntryResponse {
    type Error = NotClientVisible;

    fn try_from(entry: TicketEntry) -> Result<Self, Self::Error> {
        Ok(Self {
            id: entry.id,
            kind: IntakeEntryKind::try_from(entry.kind)?,
            body: entry.body,
            attachments: entry
                .attachments
                .into_iter()
                .map(IntakeAttachmentResponse::from)
                .collect(),
            created_at: entry.created_at,
        })
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct IntakeTicketResponse {
    pub id: TicketId,
    pub number: TicketNumber,
    pub subject: String,
    pub body: String,
    pub status: TicketStatus,
    #[schema(value_type = Object)]
    pub custom_fields: Value,
    pub badges: Vec<BadgeResponse>,
    pub external_id: Option<String>,
    pub created_at: DateTime<Utc>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entries: Option<Vec<IntakeEntryResponse>>,
}

impl From<Ticket> for IntakeTicketResponse {
    fn from(ticket: Ticket) -> Self {
        Self {
            id: ticket.id,
            number: ticket.number,
            subject: ticket.subject,
            body: ticket.body,
            status: ticket.status,
            custom_fields: ticket.custom_fields.into_value(),
            badges: ticket.badges.iter().map(BadgeResponse::from).collect(),
            external_id: ticket.external_id,
            created_at: ticket.created_at,
            entries: None,
        }
    }
}

impl IntakeTicketResponse {
    pub fn with_entries(mut self, entries: Vec<TicketEntry>) -> Self {
        self.entries = Some(
            entries
                .into_iter()
                .filter_map(|entry| IntakeEntryResponse::try_from(entry).ok())
                .collect(),
        );

        self
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, ToSchema)]
pub struct IntakeAttachmentUrlResponse {
    pub url: String,
    pub expires_at: DateTime<Utc>,
}

impl From<PresignedUrl> for IntakeAttachmentUrlResponse {
    fn from(presigned: PresignedUrl) -> Self {
        Self {
            url: presigned.url,
            expires_at: presigned.expires_at,
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;
    use mestier_core::{Badge, BadgeTone, CustomFields, MemberId, OrganizationId};
    use serde_json::json;
    use uuid::Uuid;

    use super::*;

    fn sorted_keys(value: &Value) -> Vec<&str> {
        let mut keys: Vec<&str> = value
            .as_object()
            .unwrap()
            .keys()
            .map(String::as_str)
            .collect();
        keys.sort_unstable();

        keys
    }

    fn moment() -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 11, 4, 9, 30, 0).unwrap()
    }

    fn attachment() -> Attachment {
        Attachment {
            id: AttachmentId(Uuid::now_v7()),
            organization_id: OrganizationId(Uuid::now_v7()),
            ticket_id: TicketId(Uuid::now_v7()),
            entry_id: TicketEntryId(Uuid::now_v7()),
            file_key: "uploads/support-tickets/secret-key".to_owned(),
            file_name: "photo.png".to_owned(),
            mime_type: "image/png".to_owned(),
            size_bytes: 12,
            created_at: moment(),
        }
    }

    fn entry(kind: EntryKind, body: &str, author: Option<MemberId>) -> TicketEntry {
        TicketEntry {
            id: TicketEntryId(Uuid::now_v7()),
            organization_id: OrganizationId(Uuid::now_v7()),
            ticket_id: TicketId(Uuid::now_v7()),
            kind,
            author,
            body: body.to_owned(),
            attachments: vec![attachment()],
            created_at: moment(),
            updated_at: moment(),
        }
    }

    fn ticket() -> Ticket {
        Ticket {
            id: TicketId(Uuid::now_v7()),
            organization_id: OrganizationId(Uuid::now_v7()),
            number: TicketNumber(7),
            subject: "Printer on fire".to_owned(),
            body: "It smokes".to_owned(),
            status: TicketStatus::InProgress,
            assignee: Some(MemberId(Uuid::now_v7())),
            assigned_at: Some(moment()),
            custom_fields: CustomFields::try_from(json!({ "plan": "pro" })).unwrap(),
            badges: vec![Badge::new("VIP", BadgeTone::Brand).unwrap()],
            external_id: Some("autharie-1".to_owned()),
            created_at: moment(),
            updated_at: moment(),
        }
    }

    #[test]
    fn a_ticket_response_carries_exactly_the_client_visible_fields() {
        let assignee = ticket().assignee.unwrap().0.to_string();
        let response = IntakeTicketResponse::from(ticket());

        let value = serde_json::to_value(&response).unwrap();
        let text = value.to_string();

        assert_eq!(
            sorted_keys(&value),
            [
                "badges",
                "body",
                "created_at",
                "custom_fields",
                "external_id",
                "id",
                "number",
                "status",
                "subject"
            ]
        );
        assert_eq!(
            value["badges"],
            json!([{ "label": "VIP", "tone": "brand" }])
        );
        assert_eq!(value["status"], "in_progress");
        assert!(!text.contains("assignee"));
        assert!(!text.contains("assigned_at"));
        assert!(!text.contains(&assignee));
    }

    #[test]
    fn a_ticket_view_lists_the_client_visible_entries_and_drops_the_rest() {
        let entries = vec![
            entry(EntryKind::ClientMessage, "from the client", None),
            entry(
                EntryKind::InternalNote,
                "SECRET NOTE",
                Some(MemberId(Uuid::now_v7())),
            ),
            entry(
                EntryKind::Reply,
                "from the team",
                Some(MemberId(Uuid::now_v7())),
            ),
            entry(
                EntryKind::Draft,
                "SECRET DRAFT",
                Some(MemberId(Uuid::now_v7())),
            ),
        ];

        let value =
            serde_json::to_value(IntakeTicketResponse::from(ticket()).with_entries(entries))
                .unwrap();
        let text = value.to_string();

        let kinds: Vec<&str> = value["entries"]
            .as_array()
            .unwrap()
            .iter()
            .map(|entry| entry["kind"].as_str().unwrap())
            .collect();
        assert_eq!(kinds, ["client_message", "reply"]);
        assert!(!text.contains("SECRET"));
        assert!(!text.contains("author"));
        assert!(!text.contains("secret-key"));
        assert!(!text.contains("file_key"));
    }

    #[test]
    fn a_ticket_without_a_view_has_no_entries_key() {
        let value = serde_json::to_value(IntakeTicketResponse::from(ticket())).unwrap();

        assert!(value.get("entries").is_none());
    }

    #[test]
    fn an_entry_response_has_neither_author_nor_update_time_nor_ticket() {
        let value = serde_json::to_value(
            IntakeEntryResponse::try_from(entry(
                EntryKind::Reply,
                "from the team",
                Some(MemberId(Uuid::now_v7())),
            ))
            .unwrap(),
        )
        .unwrap();

        assert_eq!(
            sorted_keys(&value),
            ["attachments", "body", "created_at", "id", "kind"]
        );
        assert_eq!(
            sorted_keys(&value["attachments"][0]),
            ["created_at", "file_name", "id", "mime_type", "size_bytes"]
        );
    }

    #[test]
    fn only_a_client_message_and_a_reply_have_an_intake_kind() {
        assert_eq!(
            IntakeEntryKind::try_from(EntryKind::ClientMessage),
            Ok(IntakeEntryKind::ClientMessage)
        );
        assert_eq!(
            IntakeEntryKind::try_from(EntryKind::Reply),
            Ok(IntakeEntryKind::Reply)
        );
        assert_eq!(
            IntakeEntryKind::try_from(EntryKind::InternalNote),
            Err(NotClientVisible)
        );
        assert_eq!(
            IntakeEntryKind::try_from(EntryKind::Draft),
            Err(NotClientVisible)
        );
    }

    #[test]
    fn an_entry_converts_only_when_a_client_may_see_it() {
        let author = Some(MemberId(Uuid::now_v7()));

        let client_message =
            IntakeEntryResponse::try_from(entry(EntryKind::ClientMessage, "from the client", None))
                .unwrap();
        let reply = IntakeEntryResponse::try_from(entry(EntryKind::Reply, "from the team", author))
            .unwrap();
        let note = IntakeEntryResponse::try_from(entry(EntryKind::InternalNote, "SECRET", author));
        let draft = IntakeEntryResponse::try_from(entry(EntryKind::Draft, "SECRET", author));

        assert_eq!(client_message.kind, IntakeEntryKind::ClientMessage);
        assert_eq!(client_message.body, "from the client");
        assert_eq!(reply.kind, IntakeEntryKind::Reply);
        assert_eq!(reply.body, "from the team");
        assert_eq!(note, Err(NotClientVisible));
        assert_eq!(draft, Err(NotClientVisible));
    }

    #[test]
    fn the_intake_kind_serializes_in_snake_case() {
        assert_eq!(
            serde_json::to_value(IntakeEntryKind::ClientMessage).unwrap(),
            "client_message"
        );
        assert_eq!(
            serde_json::to_value(IntakeEntryKind::Reply).unwrap(),
            "reply"
        );
    }

    #[test]
    fn a_ticket_response_carries_the_creation_time_and_never_the_update_time() {
        let mut moved = ticket();
        moved.updated_at = moment() + chrono::Duration::days(3);

        let text = serde_json::to_value(IntakeTicketResponse::from(moved))
            .unwrap()
            .to_string();

        assert!(text.contains("created_at"));
        assert!(!text.contains("updated_at"));
        assert!(!text.contains("2026-11-07"));
    }

    #[test]
    fn the_url_response_has_the_url_and_its_expiry_only() {
        let value = serde_json::to_value(IntakeAttachmentUrlResponse::from(PresignedUrl {
            url: "http://storage/object".to_owned(),
            expires_at: moment(),
        }))
        .unwrap();

        assert_eq!(sorted_keys(&value), ["expires_at", "url"]);
    }
}
