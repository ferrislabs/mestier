use axum_extra::routing::TypedPath;
use mestier_core::{AttachmentId, OrganizationId, SupportApiKeyId, TicketEntryId, TicketId};
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

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/support/api-keys")]
pub struct ApiKeysPath {
    pub organization_id: OrganizationId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/organizations/{organization_id}/support/api-keys/{key_id}")]
pub struct ApiKeyPath {
    pub organization_id: OrganizationId,
    pub key_id: SupportApiKeyId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/support/intake/tickets")]
pub struct IntakeTicketsPath;

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/support/intake/tickets/{ticket_id}")]
pub struct IntakeTicketPath {
    pub ticket_id: TicketId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/support/intake/tickets/{ticket_id}/messages")]
pub struct IntakeMessagesPath {
    pub ticket_id: TicketId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/support/intake/tickets/{ticket_id}/attachments")]
pub struct IntakeAttachmentsPath {
    pub ticket_id: TicketId,
}

#[derive(TypedPath, Deserialize)]
#[typed_path("/api/v1/support/intake/tickets/{ticket_id}/attachments/{attachment_id}/url")]
pub struct IntakeAttachmentUrlPath {
    pub ticket_id: TicketId,
    pub attachment_id: AttachmentId,
}
