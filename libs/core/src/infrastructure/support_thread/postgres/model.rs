use chrono::{DateTime, Utc};
use common::CoreError;
use uuid::Uuid;

use crate::{
    Attachment, AttachmentId, EntryKind, LocatedAttachment, MemberId, OrganizationId, TicketEntry,
    TicketEntryId, TicketId,
};

#[derive(Debug, Clone)]
pub struct EntryRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub ticket_id: Uuid,
    pub kind: String,
    pub author_member_id: Option<Uuid>,
    pub body: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl EntryRow {
    pub fn into_entry(self, attachments: Vec<Attachment>) -> Result<TicketEntry, CoreError> {
        let kind = self.kind.parse::<EntryKind>().map_err(|error| {
            CoreError::Internal(format!(
                "stored ticket entry {} has an invalid kind: {error}",
                self.id
            ))
        })?;

        Ok(TicketEntry {
            id: TicketEntryId(self.id),
            organization_id: OrganizationId(self.org_id),
            ticket_id: TicketId(self.ticket_id),
            kind,
            author: self.author_member_id.map(MemberId),
            body: self.body,
            attachments,
            created_at: self.created_at,
            updated_at: self.updated_at,
        })
    }
}

#[derive(Debug, Clone)]
pub struct AttachmentRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub ticket_id: Uuid,
    pub entry_id: Uuid,
    pub file_key: String,
    pub file_name: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub created_at: DateTime<Utc>,
}

impl TryFrom<AttachmentRow> for Attachment {
    type Error = CoreError;

    fn try_from(row: AttachmentRow) -> Result<Self, Self::Error> {
        let size_bytes = u64::try_from(row.size_bytes).map_err(|error| {
            CoreError::Internal(format!(
                "stored attachment {} has an invalid size: {error}",
                row.id
            ))
        })?;

        Ok(Self {
            id: AttachmentId(row.id),
            organization_id: OrganizationId(row.org_id),
            ticket_id: TicketId(row.ticket_id),
            entry_id: TicketEntryId(row.entry_id),
            file_key: row.file_key,
            file_name: row.file_name,
            mime_type: row.mime_type,
            size_bytes,
            created_at: row.created_at,
        })
    }
}

#[derive(Debug, Clone)]
pub struct LocatedAttachmentRow {
    pub id: Uuid,
    pub org_id: Uuid,
    pub ticket_id: Uuid,
    pub entry_id: Uuid,
    pub file_key: String,
    pub file_name: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub created_at: DateTime<Utc>,
    pub entry_kind: String,
}

impl TryFrom<LocatedAttachmentRow> for LocatedAttachment {
    type Error = CoreError;

    fn try_from(row: LocatedAttachmentRow) -> Result<Self, Self::Error> {
        let entry_kind = row.entry_kind.parse::<EntryKind>().map_err(|error| {
            CoreError::Internal(format!(
                "stored ticket entry {} has an invalid kind: {error}",
                row.entry_id
            ))
        })?;
        let attachment = Attachment::try_from(AttachmentRow {
            id: row.id,
            org_id: row.org_id,
            ticket_id: row.ticket_id,
            entry_id: row.entry_id,
            file_key: row.file_key,
            file_name: row.file_name,
            mime_type: row.mime_type,
            size_bytes: row.size_bytes,
            created_at: row.created_at,
        })?;

        Ok(Self {
            attachment,
            entry_kind,
        })
    }
}
