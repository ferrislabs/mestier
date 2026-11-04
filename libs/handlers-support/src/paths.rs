use axum_extra::routing::TypedPath;
use mestier_core::{AttachmentId, OrganizationId, TicketEntryId, TicketId};
use serde::Deserialize;

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/support/tickets")]
pub struct TicketsPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/support/tickets/{ticket_id}")]
pub struct TicketPath {
    pub organization_id: OrganizationId,
    pub ticket_id: TicketId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/support/tickets/{ticket_id}/status")]
pub struct TicketStatusPath {
    pub organization_id: OrganizationId,
    pub ticket_id: TicketId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/support/tickets/{ticket_id}/assignee")]
pub struct TicketAssigneePath {
    pub organization_id: OrganizationId,
    pub ticket_id: TicketId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/support/tickets/{ticket_id}/badges")]
pub struct TicketBadgesPath {
    pub organization_id: OrganizationId,
    pub ticket_id: TicketId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/support/tickets/{ticket_id}/badges/{label}")]
pub struct TicketBadgePath {
    pub organization_id: OrganizationId,
    pub ticket_id: TicketId,
    pub label: String,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/support/tickets/{ticket_id}/entries")]
pub struct TicketEntriesPath {
    pub organization_id: OrganizationId,
    pub ticket_id: TicketId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/support/entries/{entry_id}")]
pub struct EntryPath {
    pub organization_id: OrganizationId,
    pub entry_id: TicketEntryId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/support/entries/{entry_id}/send")]
pub struct EntrySendPath {
    pub organization_id: OrganizationId,
    pub entry_id: TicketEntryId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/support/entries/{entry_id}/attachments")]
pub struct EntryAttachmentsPath {
    pub organization_id: OrganizationId,
    pub entry_id: TicketEntryId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/support/attachments/{attachment_id}/url")]
pub struct AttachmentUrlPath {
    pub organization_id: OrganizationId,
    pub attachment_id: AttachmentId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/support/settings")]
pub struct SettingsPath {
    pub organization_id: OrganizationId,
}
